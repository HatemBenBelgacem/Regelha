#[cfg(feature = "ssr")]
use chrono::Local;
use chrono::NaiveDate;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Aufgabe {
    pub id: Uuid,
    pub beschreibung: String,
    pub datum: NaiveDate,
    pub erledigt: bool,
    pub benutzer_id: Uuid,
    pub benutzer: String,
}

#[cfg(feature = "ssr")]
mod store {
    use super::*;
    use crate::backend::auth::default_admin_uuid;
    use std::sync::{LazyLock, Mutex};

    pub static AUFGABEN_MEMORY: LazyLock<Mutex<Vec<Aufgabe>>> = LazyLock::new(|| {
        let today = Local::now().date_naive();
        let admin_id = default_admin_uuid();
        Mutex::new(vec![
            Aufgabe {
                id: Uuid::new_v4(),
                beschreibung: "Monatsbudget prüfen".to_string(),
                datum: today,
                erledigt: false,
                benutzer_id: admin_id,
                benutzer: "admin".to_string(),
            },
            Aufgabe {
                id: Uuid::new_v4(),
                beschreibung: "Steuerunterlagen sortieren".to_string(),
                datum: today,
                erledigt: false,
                benutzer_id: admin_id,
                benutzer: "admin".to_string(),
            },
        ])
    });
}

#[server(GetAufgaben, "/api", endpoint = "get_aufgaben")]
pub async fn get_aufgaben() -> Result<Vec<Aufgabe>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::{PgPool, Row};

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            let rows = sqlx::query(
                r#"
                SELECT
                    a.id,
                    a.beschreibung,
                    a.datum,
                    a.erledigt,
                    a.benutzer_id,
                    u.benutzername AS benutzer
                FROM aufgaben a
                INNER JOIN benutzer u ON a.benutzer_id = u.id
                WHERE a.benutzer_id = $1
                ORDER BY a.erledigt ASC, a.datum ASC
                "#,
            )
            .bind(current_user.id)
            .fetch_all(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Fehler (aufgaben): {e}")))?;

            let list = rows
                .into_iter()
                .map(|row| Aufgabe {
                    id: row.get("id"),
                    beschreibung: row.get("beschreibung"),
                    datum: row.get("datum"),
                    erledigt: row.get("erledigt"),
                    benutzer_id: row.get("benutzer_id"),
                    benutzer: row.get("benutzer"),
                })
                .collect();
            return Ok(list);
        }

        let guard = store::AUFGABEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        let filtered = guard
            .iter()
            .filter(|a| a.benutzer_id == current_user.id || a.benutzer == current_user.benutzername)
            .cloned()
            .collect();
        Ok(filtered)
    }
}

#[server(AddAufgabe, "/api", endpoint = "add_aufgabe")]
pub async fn add_aufgabe(
    beschreibung: String,
    datum: Option<String>,
) -> Result<Aufgabe, ServerFnError> {
    let trimmed = beschreibung.trim().to_string();
    if trimmed.is_empty() {
        return Err(ServerFnError::new(
            "Bitte eine Beschreibung für die Aufgabe eingeben.",
        ));
    }

    let aufgaben_datum = match datum.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(d_str) => NaiveDate::parse_from_str(d_str, "%Y-%m-%d")
            .or_else(|_| NaiveDate::parse_from_str(d_str, "%d.%m.%Y"))
            .map_err(|_| ServerFnError::new("Bitte ein gültiges Datum auswählen."))?,
        None => Local::now().date_naive(),
    };

    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let current_user = require_current_user().await?;

        let neue_aufgabe = Aufgabe {
            id: Uuid::new_v4(),
            beschreibung: trimmed,
            datum: aufgaben_datum,
            erledigt: false,
            benutzer_id: current_user.id,
            benutzer: current_user.benutzername,
        };

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query(
                r#"
                INSERT INTO aufgaben (id, beschreibung, datum, erledigt, benutzer_id)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(neue_aufgabe.id)
            .bind(&neue_aufgabe.beschreibung)
            .bind(neue_aufgabe.datum)
            .bind(neue_aufgabe.erledigt)
            .bind(neue_aufgabe.benutzer_id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Insert fehlgeschlagen: {e}")))?;

            return Ok(neue_aufgabe);
        }

        let mut guard = store::AUFGABEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.push(neue_aufgabe.clone());
        return Ok(neue_aufgabe);
    }

    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

#[server(ToggleAufgabeErledigt, "/api", endpoint = "toggle_aufgabe_erledigt")]
pub async fn toggle_aufgabe_erledigt(id: Uuid, erledigt: bool) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query("UPDATE aufgaben SET erledigt = $1 WHERE id = $2 AND benutzer_id = $3")
                .bind(erledigt)
                .bind(id)
                .bind(current_user.id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Update fehlgeschlagen: {e}")))?;
            return Ok(());
        }

        let mut guard = store::AUFGABEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        if let Some(item) = guard.iter_mut().find(|a| {
            a.id == id
                && (a.benutzer_id == current_user.id || a.benutzer == current_user.benutzername)
        }) {
            item.erledigt = erledigt;
        }
    }

    Ok(())
}

#[server(DeleteAufgabe, "/api", endpoint = "delete_aufgabe")]
pub async fn delete_aufgabe(id: Uuid) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query("DELETE FROM aufgaben WHERE id = $1 AND benutzer_id = $2")
                .bind(id)
                .bind(current_user.id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Delete fehlgeschlagen: {e}")))?;
            return Ok(());
        }

        let mut guard = store::AUFGABEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.retain(|a| {
            !(a.id == id
                && (a.benutzer_id == current_user.id || a.benutzer == current_user.benutzername))
        });
    }

    Ok(())
}
