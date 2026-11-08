#[cfg(feature = "ssr")]
use chrono::Local;
use chrono::NaiveDate;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Eine Schuld mit Datum, Grund und Betrag in CHF. Gehört einem Benutzer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Schuld {
    pub id: Uuid,
    pub datum: NaiveDate,
    pub grund: String,
    pub betrag: f64,
    pub benutzer_id: Uuid,
}

#[cfg(feature = "ssr")]
mod store {
    use super::*;
    use crate::backend::auth::default_admin_uuid;
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
                benutzer_id: admin_id,
            },
            Schuld {
                id: Uuid::new_v4(),
                datum: today,
                grund: "Restbetrag Ferienwohnung".to_string(),
                betrag: 240.00,
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
fn parse_datum(text: &str) -> Result<NaiveDate, ServerFnError> {
    let text = text.trim();
    NaiveDate::parse_from_str(text, "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(text, "%d.%m.%Y"))
        .map_err(|_| ServerFnError::new("Bitte ein gültiges Datum auswählen."))
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
                SELECT id, datum, grund, betrag, benutzer_id
                FROM schulden
                WHERE benutzer_id = $1
                ORDER BY datum DESC, grund ASC
                "#,
            )
            .bind(current_user.id)
            .fetch_all(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Fehler (schulden): {e}")))?;

            let list = rows
                .into_iter()
                .map(|row| Schuld {
                    id: row.get("id"),
                    datum: row.get("datum"),
                    grund: row.get("grund"),
                    betrag: row.get("betrag"),
                    benutzer_id: row.get("benutzer_id"),
                })
                .collect();
            return Ok(list);
        }

        let guard = store::SCHULDEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        let mut liste: Vec<Schuld> = guard
            .iter()
            .filter(|s| s.benutzer_id == current_user.id)
            .cloned()
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
) -> Result<Schuld, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let datum = parse_datum(&datum)?;
        let grund = pruefe_grund_und_betrag(&grund, betrag)?;
        let current_user = require_current_user().await?;

        let neue_schuld = Schuld {
            id: Uuid::new_v4(),
            datum,
            grund,
            betrag,
            benutzer_id: current_user.id,
        };

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query(
                r#"
                INSERT INTO schulden (id, datum, grund, betrag, benutzer_id)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(neue_schuld.id)
            .bind(neue_schuld.datum)
            .bind(&neue_schuld.grund)
            .bind(neue_schuld.betrag)
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
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let datum = parse_datum(&datum)?;
        let grund = pruefe_grund_und_betrag(&grund, betrag)?;
        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query(
                "UPDATE schulden SET datum = $1, grund = $2, betrag = $3 WHERE id = $4 AND benutzer_id = $5",
            )
            .bind(datum)
            .bind(&grund)
            .bind(betrag)
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
