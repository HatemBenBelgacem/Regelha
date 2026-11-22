use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Eine Kategorie (z.B. "Wohnen", "Versicherung"), mit der Fixkosten und Fixeinnahmen gruppiert werden.
/// Kategorien werden in den Einstellungen verwaltet und gehören einem Benutzer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Kategorie {
    pub id: Uuid,
    pub name: String,
    pub benutzer_id: Uuid,
}

/// Anzeigename einer Kategorie zu einer Id. Ohne Kategorie (oder bei unbekannter Id) steht dort "Ohne Kategorie".
pub fn kategorie_name(kategorien: &[Kategorie], id: Option<Uuid>) -> String {
    id.and_then(|id| kategorien.iter().find(|k| k.id == id))
        .map(|k| k.name.clone())
        .unwrap_or_else(|| "Ohne Kategorie".to_string())
}

#[cfg(feature = "ssr")]
const KATEGORIE_HAT_EINTRAEGE: &str =
    "Diese Kategorie wird noch von Fixkosten oder Fixeinnahmen verwendet und kann nicht gelöscht werden.";

#[cfg(feature = "ssr")]
const KATEGORIE_DOPPELT: &str = "Es gibt bereits eine Kategorie mit diesem Namen.";

#[cfg(feature = "ssr")]
const KATEGORIE_UNBEKANNT: &str = "Die gewählte Kategorie ist nicht vorhanden.";

#[cfg(feature = "ssr")]
pub(crate) mod store {
    use super::*;
    use crate::backend::auth::default_admin_uuid;
    use std::sync::{LazyLock, Mutex};

    /// Feste Ids der Beispielkategorien, damit die Beispiel-Fixkosten darauf verweisen können.
    pub const WOHNEN_ID: Uuid = Uuid::from_u128(0x6b5a_0000_0000_4000_8000_0000_0000_0101);
    pub const GESUNDHEIT_ID: Uuid = Uuid::from_u128(0x6b5a_0000_0000_4000_8000_0000_0000_0102);
    pub const KOMMUNIKATION_ID: Uuid = Uuid::from_u128(0x6b5a_0000_0000_4000_8000_0000_0000_0103);

    pub static KATEGORIEN_MEMORY: LazyLock<Mutex<Vec<Kategorie>>> = LazyLock::new(|| {
        let admin_id = default_admin_uuid();
        Mutex::new(vec![
            Kategorie {
                id: WOHNEN_ID,
                name: "Wohnen".to_string(),
                benutzer_id: admin_id,
            },
            Kategorie {
                id: GESUNDHEIT_ID,
                name: "Gesundheit".to_string(),
                benutzer_id: admin_id,
            },
            Kategorie {
                id: KOMMUNIKATION_ID,
                name: "Kommunikation".to_string(),
                benutzer_id: admin_id,
            },
        ])
    });
}

/// Prüft den Namen einer Kategorie (Pflicht, höchstens 60 Zeichen) und gibt ihn bereinigt zurück.
#[cfg(feature = "ssr")]
fn pruefe_name(name: &str) -> Result<String, ServerFnError> {
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        return Err(ServerFnError::new("Bitte einen Namen für die Kategorie eingeben."));
    }
    if trimmed.chars().count() > 60 {
        return Err(ServerFnError::new(
            "Der Name der Kategorie darf höchstens 60 Zeichen lang sein.",
        ));
    }
    Ok(trimmed)
}

/// Prüft, ob es für diesen Benutzer schon eine Kategorie mit diesem Namen gibt
/// (Groß- und Kleinschreibung werden nicht unterschieden). `ausser` schliesst eine Kategorie aus, z.B. beim Umbenennen.
#[cfg(feature = "ssr")]
async fn name_ist_vergeben(
    name: &str,
    benutzer_id: Uuid,
    ausser: Option<Uuid>,
) -> Result<bool, ServerFnError> {
    use sqlx::PgPool;

    if let Some(pool) = use_context::<PgPool>() {
        let vergeben: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM kategorien
                WHERE benutzer_id = $1 AND LOWER(name) = LOWER($2) AND ($3::uuid IS NULL OR id <> $3)
            )
            "#,
        )
        .bind(benutzer_id)
        .bind(name)
        .bind(ausser)
        .fetch_one(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("DB-Fehler (kategorien): {e}")))?;
        return Ok(vergeben);
    }

    let guard = store::KATEGORIEN_MEMORY
        .lock()
        .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
    Ok(guard.iter().any(|k| {
        k.benutzer_id == benutzer_id
            && k.name.to_lowercase() == name.to_lowercase()
            && Some(k.id) != ausser
    }))
}

/// Prüft, ob eine Kategorie existiert und dem angemeldeten Benutzer gehört.
/// `None` ist erlaubt und bedeutet "ohne Kategorie".
#[cfg(feature = "ssr")]
pub(crate) async fn pruefe_kategorie_zugehoerig(
    kategorie_id: Option<Uuid>,
    benutzer_id: Uuid,
) -> Result<(), ServerFnError> {
    use sqlx::PgPool;

    let Some(id) = kategorie_id else {
        return Ok(());
    };

    let gehoert = if let Some(pool) = use_context::<PgPool>() {
        let vorhanden: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM kategorien WHERE id = $1 AND benutzer_id = $2)",
        )
        .bind(id)
        .bind(benutzer_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("DB-Fehler (kategorien): {e}")))?;
        vorhanden
    } else {
        store::KATEGORIEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?
            .iter()
            .any(|k| k.id == id && k.benutzer_id == benutzer_id)
    };

    if gehoert {
        Ok(())
    } else {
        Err(ServerFnError::new(KATEGORIE_UNBEKANNT))
    }
}

#[server(
    GetKategorien,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "get_kategorien"
)]
pub async fn get_kategorien() -> Result<Vec<Kategorie>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::{PgPool, Row};

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            let rows = sqlx::query(
                r#"
                SELECT id, name, benutzer_id
                FROM kategorien
                WHERE benutzer_id = $1
                ORDER BY LOWER(name) ASC
                "#,
            )
            .bind(current_user.id)
            .fetch_all(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Fehler (kategorien): {e}")))?;

            let list = rows
                .into_iter()
                .map(|row| Kategorie {
                    id: row.get("id"),
                    name: row.get("name"),
                    benutzer_id: row.get("benutzer_id"),
                })
                .collect();
            return Ok(list);
        }

        let guard = store::KATEGORIEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        let mut liste: Vec<Kategorie> = guard
            .iter()
            .filter(|k| k.benutzer_id == current_user.id)
            .cloned()
            .collect();
        liste.sort_by_key(|k| k.name.to_lowercase());
        Ok(liste)
    }
}

#[server(
    AddKategorie,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "add_kategorie"
)]
pub async fn add_kategorie(name: String) -> Result<Kategorie, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let name = pruefe_name(&name)?;
        let current_user = require_current_user().await?;

        if name_ist_vergeben(&name, current_user.id, None).await? {
            return Err(ServerFnError::new(KATEGORIE_DOPPELT));
        }

        let neue_kategorie = Kategorie {
            id: Uuid::new_v4(),
            name,
            benutzer_id: current_user.id,
        };

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query("INSERT INTO kategorien (id, name, benutzer_id) VALUES ($1, $2, $3)")
                .bind(neue_kategorie.id)
                .bind(&neue_kategorie.name)
                .bind(neue_kategorie.benutzer_id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Insert fehlgeschlagen: {e}")))?;

            return Ok(neue_kategorie);
        }

        let mut guard = store::KATEGORIEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.push(neue_kategorie.clone());
        Ok(neue_kategorie)
    }

    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

#[server(
    UpdateKategorie,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "update_kategorie"
)]
pub async fn update_kategorie(id: Uuid, name: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let name = pruefe_name(&name)?;
        let current_user = require_current_user().await?;

        if name_ist_vergeben(&name, current_user.id, Some(id)).await? {
            return Err(ServerFnError::new(KATEGORIE_DOPPELT));
        }

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query("UPDATE kategorien SET name = $1 WHERE id = $2 AND benutzer_id = $3")
                .bind(&name)
                .bind(id)
                .bind(current_user.id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Update fehlgeschlagen: {e}")))?;
            return Ok(());
        }

        let mut guard = store::KATEGORIEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        if let Some(k) = guard
            .iter_mut()
            .find(|k| k.id == id && k.benutzer_id == current_user.id)
        {
            k.name = name;
        }
    }

    Ok(())
}

/// Löscht eine Kategorie. Das ist nur möglich, wenn keine Fixkosten oder Fixeinnahmen mehr darauf verweisen.
#[server(
    DeleteKategorie,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "delete_kategorie"
)]
pub async fn delete_kategorie(id: Uuid) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            let anzahl: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM fixkosten WHERE kategorie_id = $1 AND benutzer_id = $2",
            )
            .bind(id)
            .bind(current_user.id)
            .fetch_one(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Fehler (fixkosten): {e}")))?;
            if anzahl > 0 {
                return Err(ServerFnError::new(KATEGORIE_HAT_EINTRAEGE));
            }

            sqlx::query("DELETE FROM kategorien WHERE id = $1 AND benutzer_id = $2")
                .bind(id)
                .bind(current_user.id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Delete fehlgeschlagen: {e}")))?;
            return Ok(());
        }

        let wird_verwendet = crate::backend::budget::store::FIXKOSTEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?
            .iter()
            .any(|f| f.kategorie_id == Some(id) && f.benutzer_id == current_user.id);
        if wird_verwendet {
            return Err(ServerFnError::new(KATEGORIE_HAT_EINTRAEGE));
        }

        let mut guard = store::KATEGORIEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.retain(|k| !(k.id == id && k.benutzer_id == current_user.id));
    }

    Ok(())
}
