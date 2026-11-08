use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Ein Kontakt mit Vorname (Pflicht) und Nachname (optional). Gehört einem Benutzer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Kontakt {
    pub id: Uuid,
    pub vorname: String,
    pub nachname: String,
    pub benutzer_id: Uuid,
}

/// Anzeigename für Listen und Auswahlfelder, z.B. "Max Muster" oder nur "Max".
pub fn anzeigename(vorname: &str, nachname: &str) -> String {
    let nachname = nachname.trim();
    if nachname.is_empty() {
        vorname.trim().to_string()
    } else {
        format!("{} {}", vorname.trim(), nachname)
    }
}

#[cfg(feature = "ssr")]
const KONTAKT_HAT_SCHULDEN: &str =
    "Dieser Kontakt hat noch Schulden und kann nicht gelöscht werden.";

#[cfg(feature = "ssr")]
pub(crate) mod store {
    use super::*;
    use crate::backend::auth::default_admin_uuid;
    use std::sync::{LazyLock, Mutex};

    /// Feste Ids der Beispielkontakte, damit die Beispiel-Schulden darauf verweisen können.
    pub const MAX_MUSTER_ID: Uuid = Uuid::from_u128(0x6b5a_0000_0000_4000_8000_0000_0000_0001);
    pub const SARA_BEISPIEL_ID: Uuid = Uuid::from_u128(0x6b5a_0000_0000_4000_8000_0000_0000_0002);

    pub static KONTAKTE_MEMORY: LazyLock<Mutex<Vec<Kontakt>>> = LazyLock::new(|| {
        let admin_id = default_admin_uuid();
        Mutex::new(vec![
            Kontakt {
                id: MAX_MUSTER_ID,
                vorname: "Max".to_string(),
                nachname: "Muster".to_string(),
                benutzer_id: admin_id,
            },
            Kontakt {
                id: SARA_BEISPIEL_ID,
                vorname: "Sara".to_string(),
                nachname: "Beispiel".to_string(),
                benutzer_id: admin_id,
            },
        ])
    });
}

/// Prüft den Vornamen (Pflicht) und gibt Vor- und Nachnamen bereinigt zurück.
#[cfg(feature = "ssr")]
fn pruefe_namen(vorname: &str, nachname: &str) -> Result<(String, String), ServerFnError> {
    let vorname = vorname.trim().to_string();
    if vorname.is_empty() {
        return Err(ServerFnError::new("Bitte einen Vornamen eingeben."));
    }
    Ok((vorname, nachname.trim().to_string()))
}

#[server(
    GetKontakte,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "get_kontakte"
)]
pub async fn get_kontakte() -> Result<Vec<Kontakt>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::{PgPool, Row};

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            let rows = sqlx::query(
                r#"
                SELECT id, vorname, nachname, benutzer_id
                FROM kontakte
                WHERE benutzer_id = $1
                ORDER BY nachname ASC, vorname ASC
                "#,
            )
            .bind(current_user.id)
            .fetch_all(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Fehler (kontakte): {e}")))?;

            let list = rows
                .into_iter()
                .map(|row| Kontakt {
                    id: row.get("id"),
                    vorname: row.get("vorname"),
                    nachname: row.get("nachname"),
                    benutzer_id: row.get("benutzer_id"),
                })
                .collect();
            return Ok(list);
        }

        let guard = store::KONTAKTE_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        let mut liste: Vec<Kontakt> = guard
            .iter()
            .filter(|k| k.benutzer_id == current_user.id)
            .cloned()
            .collect();
        liste.sort_by(|a, b| (&a.nachname, &a.vorname).cmp(&(&b.nachname, &b.vorname)));
        Ok(liste)
    }
}

#[server(
    AddKontakt,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "add_kontakt"
)]
pub async fn add_kontakt(vorname: String, nachname: String) -> Result<Kontakt, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let (vorname, nachname) = pruefe_namen(&vorname, &nachname)?;
        let current_user = require_current_user().await?;

        let neuer_kontakt = Kontakt {
            id: Uuid::new_v4(),
            vorname,
            nachname,
            benutzer_id: current_user.id,
        };

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query(
                r#"
                INSERT INTO kontakte (id, vorname, nachname, benutzer_id)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(neuer_kontakt.id)
            .bind(&neuer_kontakt.vorname)
            .bind(&neuer_kontakt.nachname)
            .bind(neuer_kontakt.benutzer_id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Insert fehlgeschlagen: {e}")))?;

            return Ok(neuer_kontakt);
        }

        let mut guard = store::KONTAKTE_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.push(neuer_kontakt.clone());
        Ok(neuer_kontakt)
    }

    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

#[server(
    UpdateKontakt,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "update_kontakt"
)]
pub async fn update_kontakt(
    id: Uuid,
    vorname: String,
    nachname: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let (vorname, nachname) = pruefe_namen(&vorname, &nachname)?;
        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query(
                "UPDATE kontakte SET vorname = $1, nachname = $2 WHERE id = $3 AND benutzer_id = $4",
            )
            .bind(&vorname)
            .bind(&nachname)
            .bind(id)
            .bind(current_user.id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Update fehlgeschlagen: {e}")))?;
            return Ok(());
        }

        let mut guard = store::KONTAKTE_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        if let Some(k) = guard
            .iter_mut()
            .find(|k| k.id == id && k.benutzer_id == current_user.id)
        {
            k.vorname = vorname;
            k.nachname = nachname;
        }
    }

    Ok(())
}

/// Löscht einen Kontakt. Das ist nur möglich, wenn keine Schulden mehr darauf verweisen.
#[server(
    DeleteKontakt,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "delete_kontakt"
)]
pub async fn delete_kontakt(id: Uuid) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            let anzahl: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM schulden WHERE kontakt_id = $1 AND benutzer_id = $2",
            )
            .bind(id)
            .bind(current_user.id)
            .fetch_one(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Fehler (schulden): {e}")))?;
            if anzahl > 0 {
                return Err(ServerFnError::new(KONTAKT_HAT_SCHULDEN));
            }

            sqlx::query("DELETE FROM kontakte WHERE id = $1 AND benutzer_id = $2")
                .bind(id)
                .bind(current_user.id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Delete fehlgeschlagen: {e}")))?;
            return Ok(());
        }

        let hat_schulden = crate::backend::schulden::store::SCHULDEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?
            .iter()
            .any(|s| s.kontakt_id == Some(id) && s.benutzer_id == current_user.id);
        if hat_schulden {
            return Err(ServerFnError::new(KONTAKT_HAT_SCHULDEN));
        }

        let mut guard = store::KONTAKTE_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.retain(|k| !(k.id == id && k.benutzer_id == current_user.id));
    }

    Ok(())
}
