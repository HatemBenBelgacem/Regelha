#[cfg(feature = "ssr")]
use chrono::Local;
use chrono::NaiveDate;
#[cfg(feature = "ssr")]
use crate::backend::kontakte::anzeigename;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Eine Schuld mit Datum, Grund und Betrag in CHF.
/// Beziehung 1:N: Ein Kontakt kann viele Schulden haben, jede Schuld gehört zu einem Kontakt.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Schuld {
    pub id: Uuid,
    pub datum: NaiveDate,
    pub grund: String,
    pub betrag: f64,
    /// Id des zugehörigen Kontakts. Leer nur bei Altdaten ohne Kontakt.
    pub kontakt_id: Option<Uuid>,
    /// Anzeigename des Kontakts, z.B. "Max Muster". Wird beim Lesen aus dem Kontakt geholt.
    pub kontakt: Option<String>,
    pub benutzer_id: Uuid,
}

#[cfg(feature = "ssr")]
const KONTAKT_UNGUELTIG: &str = "Bitte einen gültigen Kontakt auswählen.";

#[cfg(feature = "ssr")]
pub(crate) mod store {
    use super::*;
    use crate::backend::auth::default_admin_uuid;
    use crate::backend::kontakte::store::{MAX_MUSTER_ID, SARA_BEISPIEL_ID};
    use std::sync::{LazyLock, Mutex};

    pub static SCHULDEN_MEMORY: LazyLock<Mutex<Vec<Schuld>>> = LazyLock::new(|| {
        let today = Local::now().date_naive();
        let admin_id = default_admin_uuid();
        Mutex::new(vec![
            Schuld {
                id: Uuid::new_v4(),
                datum: today,
                grund: "Darlehen von Freund".to_string(),
                betrag: 500.00,
                kontakt_id: Some(MAX_MUSTER_ID),
                kontakt: None,
                benutzer_id: admin_id,
            },
            Schuld {
                id: Uuid::new_v4(),
                datum: today,
                grund: "Restbetrag Ferienwohnung".to_string(),
                betrag: 240.00,
                kontakt_id: Some(SARA_BEISPIEL_ID),
                kontakt: None,
                benutzer_id: admin_id,
            },
        ])
    });
}

/// Prüft Grund und Betrag. Gibt den bereinigten Grund zurück.
#[cfg(feature = "ssr")]
fn pruefe_grund_und_betrag(grund: &str, betrag: f64) -> Result<String, ServerFnError> {
    let trimmed = grund.trim().to_string();
    if trimmed.is_empty() {
        return Err(ServerFnError::new("Bitte einen Grund eingeben."));
    }
    if betrag <= 0.0 || !betrag.is_finite() {
        return Err(ServerFnError::new(
            "Bitte einen gültigen positiven Betrag eingeben.",
        ));
    }
    Ok(trimmed)
}

/// Liest ein Datum im Format JJJJ-MM-TT (aus dem Formular) oder TT.MM.JJJJ.
#[cfg(feature = "ssr")]
pub(crate) fn parse_datum(text: &str) -> Result<NaiveDate, ServerFnError> {
    let text = text.trim();
    NaiveDate::parse_from_str(text, "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(text, "%d.%m.%Y"))
        .map_err(|_| ServerFnError::new("Bitte ein gültiges Datum auswählen."))
}

/// Holt den Namen eines Kontakts aus der Datenbank. Gehört er nicht dem Benutzer, gibt es einen Fehler.
#[cfg(feature = "ssr")]
pub(crate) async fn kontakt_name_db(
    pool: &sqlx::PgPool,
    kontakt_id: Uuid,
    benutzer_id: Uuid,
) -> Result<String, ServerFnError> {
    use sqlx::Row;

    let zeile = sqlx::query("SELECT vorname, nachname FROM kontakte WHERE id = $1 AND benutzer_id = $2")
        .bind(kontakt_id)
        .bind(benutzer_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ServerFnError::new(format!("DB-Fehler (kontakte): {e}")))?;

    match zeile {
        Some(z) => Ok(anzeigename(
            &z.get::<String, _>("vorname"),
            &z.get::<String, _>("nachname"),
        )),
        None => Err(ServerFnError::new(KONTAKT_UNGUELTIG)),
    }
}

/// Holt den Namen eines Kontakts aus dem In-Memory-Speicher.
#[cfg(feature = "ssr")]
pub(crate) fn kontakt_name_memory(kontakt_id: Uuid, benutzer_id: Uuid) -> Result<String, ServerFnError> {
    let guard = crate::backend::kontakte::store::KONTAKTE_MEMORY
        .lock()
        .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
    let name = guard
        .iter()
        .find(|k| k.id == kontakt_id && k.benutzer_id == benutzer_id)
        .map(|k| anzeigename(&k.vorname, &k.nachname));
    name.ok_or_else(|| ServerFnError::new(KONTAKT_UNGUELTIG))
}

#[server(
    GetSchulden,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "get_schulden"
)]
pub async fn get_schulden() -> Result<Vec<Schuld>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::{PgPool, Row};

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            let rows = sqlx::query(
                r#"
                SELECT s.id, s.datum, s.grund, s.betrag, s.kontakt_id, s.benutzer_id,
                       k.vorname, k.nachname
                FROM schulden s
                LEFT JOIN kontakte k ON s.kontakt_id = k.id
                WHERE s.benutzer_id = $1
                ORDER BY s.datum DESC, s.grund ASC
                "#,
            )
            .bind(current_user.id)
            .fetch_all(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Fehler (schulden): {e}")))?;

            let list = rows
                .into_iter()
                .map(|row| {
                    let vorname: Option<String> = row.get("vorname");
                    let nachname: Option<String> = row.get("nachname");
                    Schuld {
                        id: row.get("id"),
                        datum: row.get("datum"),
                        grund: row.get("grund"),
                        betrag: row.get("betrag"),
                        kontakt_id: row.get("kontakt_id"),
                        kontakt: vorname.map(|v| anzeigename(&v, nachname.as_deref().unwrap_or(""))),
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
        let guard = store::SCHULDEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        let mut liste: Vec<Schuld> = guard
            .iter()
            .filter(|s| s.benutzer_id == current_user.id)
            .map(|s| {
                let mut s = s.clone();
                s.kontakt = s.kontakt_id.and_then(|kid| {
                    kontakte
                        .iter()
                        .find(|k| k.id == kid && k.benutzer_id == current_user.id)
                        .map(|k| anzeigename(&k.vorname, &k.nachname))
                });
                s
            })
            .collect();
        liste.sort_by(|a, b| b.datum.cmp(&a.datum).then_with(|| a.grund.cmp(&b.grund)));
        Ok(liste)
    }
}

#[server(
    AddSchuld,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "add_schuld"
)]
pub async fn add_schuld(
    datum: String,
    grund: String,
    betrag: f64,
    kontakt_id: Uuid,
) -> Result<Schuld, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let datum = parse_datum(&datum)?;
        let grund = pruefe_grund_und_betrag(&grund, betrag)?;
        let current_user = require_current_user().await?;

        let kontakt = match use_context::<PgPool>() {
            Some(pool) => kontakt_name_db(&pool, kontakt_id, current_user.id).await?,
            None => kontakt_name_memory(kontakt_id, current_user.id)?,
        };

        let neue_schuld = Schuld {
            id: Uuid::new_v4(),
            datum,
            grund,
            betrag,
            kontakt_id: Some(kontakt_id),
            kontakt: Some(kontakt),
            benutzer_id: current_user.id,
        };

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query(
                r#"
                INSERT INTO schulden (id, datum, grund, betrag, kontakt_id, benutzer_id)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(neue_schuld.id)
            .bind(neue_schuld.datum)
            .bind(&neue_schuld.grund)
            .bind(neue_schuld.betrag)
            .bind(kontakt_id)
            .bind(neue_schuld.benutzer_id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Insert fehlgeschlagen: {e}")))?;

            return Ok(neue_schuld);
        }

        let mut guard = store::SCHULDEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.push(neue_schuld.clone());
        Ok(neue_schuld)
    }

    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

#[server(
    UpdateSchuld,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "update_schuld"
)]
pub async fn update_schuld(
    id: Uuid,
    datum: String,
    grund: String,
    betrag: f64,
    kontakt_id: Uuid,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let datum = parse_datum(&datum)?;
        let grund = pruefe_grund_und_betrag(&grund, betrag)?;
        let current_user = require_current_user().await?;

        let kontakt = match use_context::<PgPool>() {
            Some(pool) => kontakt_name_db(&pool, kontakt_id, current_user.id).await?,
            None => kontakt_name_memory(kontakt_id, current_user.id)?,
        };

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query(
                "UPDATE schulden SET datum = $1, grund = $2, betrag = $3, kontakt_id = $4 WHERE id = $5 AND benutzer_id = $6",
            )
            .bind(datum)
            .bind(&grund)
            .bind(betrag)
            .bind(kontakt_id)
            .bind(id)
            .bind(current_user.id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Update fehlgeschlagen: {e}")))?;
            return Ok(());
        }

        let mut guard = store::SCHULDEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        if let Some(s) = guard
            .iter_mut()
            .find(|s| s.id == id && s.benutzer_id == current_user.id)
        {
            s.datum = datum;
            s.grund = grund;
            s.betrag = betrag;
            s.kontakt_id = Some(kontakt_id);
            s.kontakt = Some(kontakt);
        }
    }

    Ok(())
}

#[server(
    DeleteSchuld,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "delete_schuld"
)]
pub async fn delete_schuld(id: Uuid) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query("DELETE FROM schulden WHERE id = $1 AND benutzer_id = $2")
                .bind(id)
                .bind(current_user.id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Delete fehlgeschlagen: {e}")))?;
            return Ok(());
        }

        let mut guard = store::SCHULDEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.retain(|s| !(s.id == id && s.benutzer_id == current_user.id));
    }

    Ok(())
}
