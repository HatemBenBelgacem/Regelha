#[cfg(feature = "ssr")]
use chrono::Local;
use chrono::NaiveDate;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Eine Rückzahlung an einen Kontakt.
/// Beziehung N:1: Viele Rückzahlungen gehören zu einem Kontakt.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rueckzahlung {
    pub id: Uuid,
    pub datum: NaiveDate,
    pub betrag: f64,
    pub kontakt_id: Uuid,
    /// Anzeigename des Kontakts, z.B. "Max Muster". Wird beim Lesen aus dem Kontakt geholt.
    pub kontakt: String,
    pub benutzer_id: Uuid,
}

#[cfg(feature = "ssr")]
pub(crate) mod store {
    use super::*;
    use crate::backend::auth::default_admin_uuid;
    use crate::backend::kontakte::store::MAX_MUSTER_ID;
    use std::sync::{LazyLock, Mutex};

    pub static RUECKZAHLUNGEN_MEMORY: LazyLock<Mutex<Vec<Rueckzahlung>>> = LazyLock::new(|| {
        let today = Local::now().date_naive();
        let admin_id = default_admin_uuid();
        Mutex::new(vec![Rueckzahlung {
            id: Uuid::new_v4(),
            datum: today,
            betrag: 100.00,
            kontakt_id: MAX_MUSTER_ID,
            kontakt: String::new(),
            benutzer_id: admin_id,
        }])
    });
}

/// Prüft den Betrag: Er muss grösser als 0 sein.
#[cfg(feature = "ssr")]
fn pruefe_betrag(betrag: f64) -> Result<(), ServerFnError> {
    if betrag <= 0.0 || !betrag.is_finite() {
        return Err(ServerFnError::new(
            "Bitte einen gültigen positiven Betrag eingeben.",
        ));
    }
    Ok(())
}

#[server(
    GetRueckzahlungen,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "get_rueckzahlungen"
)]
pub async fn get_rueckzahlungen() -> Result<Vec<Rueckzahlung>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use crate::backend::kontakte::anzeigename;
        use sqlx::{PgPool, Row};

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            let rows = sqlx::query(
                r#"
                SELECT r.id, r.datum, r.betrag, r.kontakt_id, r.benutzer_id,
                       k.vorname, k.nachname
                FROM rueckzahlungen r
                INNER JOIN kontakte k ON r.kontakt_id = k.id
                WHERE r.benutzer_id = $1
                ORDER BY r.datum DESC, k.nachname ASC, k.vorname ASC
                "#,
            )
            .bind(current_user.id)
            .fetch_all(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Fehler (rueckzahlungen): {e}")))?;

            let list = rows
                .into_iter()
                .map(|row| {
                    let vorname: String = row.get("vorname");
                    let nachname: String = row.get("nachname");
                    Rueckzahlung {
                        id: row.get("id"),
                        datum: row.get("datum"),
                        betrag: row.get("betrag"),
                        kontakt_id: row.get("kontakt_id"),
                        kontakt: anzeigename(&vorname, &nachname),
                        benutzer_id: row.get("benutzer_id"),
                    }
                })
                .collect();
            return Ok(list);
        }

        let kontakte = crate::backend::kontakte::store::KONTAKTE_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?
            .to_vec();
        let guard = store::RUECKZAHLUNGEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        let mut liste: Vec<Rueckzahlung> = guard
            .iter()
            .filter(|r| r.benutzer_id == current_user.id)
            .map(|r| {
                let mut r = r.clone();
                r.kontakt = kontakte
                    .iter()
                    .find(|k| k.id == r.kontakt_id && k.benutzer_id == current_user.id)
                    .map(|k| anzeigename(&k.vorname, &k.nachname))
                    .unwrap_or_default();
                r
            })
            .collect();
        liste.sort_by(|a, b| b.datum.cmp(&a.datum).then_with(|| a.kontakt.cmp(&b.kontakt)));
        Ok(liste)
    }
}

#[server(
    AddRueckzahlung,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "add_rueckzahlung"
)]
pub async fn add_rueckzahlung(
    datum: String,
    betrag: f64,
    kontakt_id: Uuid,
) -> Result<Rueckzahlung, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use crate::backend::schulden::{kontakt_name_db, kontakt_name_memory, parse_datum};
        use sqlx::PgPool;

        let datum = parse_datum(&datum)?;
        pruefe_betrag(betrag)?;
        let current_user = require_current_user().await?;

        let kontakt = match use_context::<PgPool>() {
            Some(pool) => kontakt_name_db(&pool, kontakt_id, current_user.id).await?,
            None => kontakt_name_memory(kontakt_id, current_user.id)?,
        };

        let neue = Rueckzahlung {
            id: Uuid::new_v4(),
            datum,
            betrag,
            kontakt_id,
            kontakt,
            benutzer_id: current_user.id,
        };

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query(
                r#"
                INSERT INTO rueckzahlungen (id, datum, betrag, kontakt_id, benutzer_id)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(neue.id)
            .bind(neue.datum)
            .bind(neue.betrag)
            .bind(neue.kontakt_id)
            .bind(neue.benutzer_id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Insert fehlgeschlagen: {e}")))?;

            return Ok(neue);
        }

        let mut guard = store::RUECKZAHLUNGEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.push(neue.clone());
        Ok(neue)
    }

    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

#[server(
    UpdateRueckzahlung,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "update_rueckzahlung"
)]
pub async fn update_rueckzahlung(
    id: Uuid,
    datum: String,
    betrag: f64,
    kontakt_id: Uuid,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use crate::backend::schulden::{kontakt_name_db, kontakt_name_memory, parse_datum};
        use sqlx::PgPool;

        let datum = parse_datum(&datum)?;
        pruefe_betrag(betrag)?;
        let current_user = require_current_user().await?;

        // Der Kontakt muss existieren und dem Benutzer gehören
        match use_context::<PgPool>() {
            Some(pool) => {
                kontakt_name_db(&pool, kontakt_id, current_user.id).await?;
            }
            None => {
                kontakt_name_memory(kontakt_id, current_user.id)?;
            }
        }

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query(
                "UPDATE rueckzahlungen SET datum = $1, betrag = $2, kontakt_id = $3 WHERE id = $4 AND benutzer_id = $5",
            )
            .bind(datum)
            .bind(betrag)
            .bind(kontakt_id)
            .bind(id)
            .bind(current_user.id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Update fehlgeschlagen: {e}")))?;
            return Ok(());
        }

        let mut guard = store::RUECKZAHLUNGEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        if let Some(r) = guard
            .iter_mut()
            .find(|r| r.id == id && r.benutzer_id == current_user.id)
        {
            r.datum = datum;
            r.betrag = betrag;
            r.kontakt_id = kontakt_id;
        }
    }

    Ok(())
}

#[server(
    DeleteRueckzahlung,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "delete_rueckzahlung"
)]
pub async fn delete_rueckzahlung(id: Uuid) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query("DELETE FROM rueckzahlungen WHERE id = $1 AND benutzer_id = $2")
                .bind(id)
                .bind(current_user.id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Delete fehlgeschlagen: {e}")))?;
            return Ok(());
        }

        let mut guard = store::RUECKZAHLUNGEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.retain(|r| !(r.id == id && r.benutzer_id == current_user.id));
    }

    Ok(())
}
