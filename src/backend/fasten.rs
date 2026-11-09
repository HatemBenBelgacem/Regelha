#[cfg(feature = "ssr")]
use chrono::Local;
use chrono::NaiveDateTime;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Ein Fasten. Beginn und Ende sind Zeitpunkte (Datum und Uhrzeit).
/// Das Datum steckt im Beginn, deshalb gibt es kein eigenes Datumsfeld mehr.
/// Ohne Ende (`None`) läuft das Fasten noch.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fasten {
    pub id: Uuid,
    pub beginn: NaiveDateTime,
    pub ende: Option<NaiveDateTime>,
    pub benutzer_id: Uuid,
}

/// Dauer des Fastens in Minuten (berechnet, nicht gespeichert).
pub fn dauer_minuten(beginn: NaiveDateTime, ende: NaiveDateTime) -> i64 {
    ende.signed_duration_since(beginn).num_minutes()
}

#[cfg(feature = "ssr")]
pub(crate) mod store {
    use super::*;
    use crate::backend::auth::default_admin_uuid;
    use chrono::NaiveTime;
    use std::sync::{LazyLock, Mutex};

    pub static FASTEN_MEMORY: LazyLock<Mutex<Vec<Fasten>>> = LazyLock::new(|| {
        let today = Local::now().date_naive();
        let morgen = today.succ_opt().expect("gültiges Beispieldatum");
        let admin_id = default_admin_uuid();
        Mutex::new(vec![Fasten {
            id: Uuid::new_v4(),
            beginn: today.and_time(NaiveTime::from_hms_opt(20, 0, 0).expect("gültige Beispielzeit")),
            ende: Some(morgen.and_time(NaiveTime::from_hms_opt(12, 0, 0).expect("gültige Beispielzeit"))),
            benutzer_id: admin_id,
        }])
    });
}

/// Liest einen Zeitpunkt im Format JJJJ-MM-TTTHH:MM (aus dem Formular) oder mit Sekunden.
#[cfg(feature = "ssr")]
fn parse_zeitpunkt(text: &str) -> Result<NaiveDateTime, ServerFnError> {
    let text = text.trim();
    NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M")
        .or_else(|_| NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S"))
        .map_err(|_| ServerFnError::new("Bitte einen gültigen Zeitpunkt eingeben."))
}

/// Liest das Ende. Ein leeres Feld heisst: das Fasten läuft noch.
#[cfg(feature = "ssr")]
fn parse_ende(text: &str) -> Result<Option<NaiveDateTime>, ServerFnError> {
    if text.trim().is_empty() {
        Ok(None)
    } else {
        parse_zeitpunkt(text).map(Some)
    }
}

/// Das Ende muss nach dem Beginn liegen (falls es ein Ende gibt).
#[cfg(feature = "ssr")]
fn pruefe_zeitraum(beginn: NaiveDateTime, ende: Option<NaiveDateTime>) -> Result<(), ServerFnError> {
    if ende.is_some_and(|ende| ende <= beginn) {
        return Err(ServerFnError::new("Das Ende muss nach dem Beginn liegen."));
    }
    Ok(())
}

#[server(
    GetFasten,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "get_fasten"
)]
pub async fn get_fasten() -> Result<Vec<Fasten>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::{PgPool, Row};

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            let rows = sqlx::query(
                r#"
                SELECT id, beginn, ende, benutzer_id
                FROM fasten
                WHERE benutzer_id = $1
                ORDER BY beginn DESC
                "#,
            )
            .bind(current_user.id)
            .fetch_all(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Fehler (fasten): {e}")))?;

            let list = rows
                .into_iter()
                .map(|row| Fasten {
                    id: row.get("id"),
                    beginn: row.get("beginn"),
                    ende: row.get("ende"),
                    benutzer_id: row.get("benutzer_id"),
                })
                .collect();
            return Ok(list);
        }

        let guard = store::FASTEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        let mut liste: Vec<Fasten> = guard
            .iter()
            .filter(|f| f.benutzer_id == current_user.id)
            .cloned()
            .collect();
        liste.sort_by_key(|f| std::cmp::Reverse(f.beginn));
        Ok(liste)
    }
}

#[server(
    AddFasten,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "add_fasten"
)]
pub async fn add_fasten(beginn: String, ende: String) -> Result<Fasten, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let beginn = parse_zeitpunkt(&beginn)?;
        let ende = parse_ende(&ende)?;
        pruefe_zeitraum(beginn, ende)?;
        let current_user = require_current_user().await?;

        let neues_fasten = Fasten {
            id: Uuid::new_v4(),
            beginn,
            ende,
            benutzer_id: current_user.id,
        };

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query(
                r#"
                INSERT INTO fasten (id, beginn, ende, benutzer_id)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(neues_fasten.id)
            .bind(neues_fasten.beginn)
            .bind(neues_fasten.ende)
            .bind(neues_fasten.benutzer_id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Insert fehlgeschlagen: {e}")))?;

            return Ok(neues_fasten);
        }

        let mut guard = store::FASTEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.push(neues_fasten.clone());
        Ok(neues_fasten)
    }

    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

#[server(
    UpdateFasten,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "update_fasten"
)]
pub async fn update_fasten(id: Uuid, beginn: String, ende: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let beginn = parse_zeitpunkt(&beginn)?;
        let ende = parse_ende(&ende)?;
        pruefe_zeitraum(beginn, ende)?;
        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query("UPDATE fasten SET beginn = $1, ende = $2 WHERE id = $3 AND benutzer_id = $4")
                .bind(beginn)
                .bind(ende)
                .bind(id)
                .bind(current_user.id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Update fehlgeschlagen: {e}")))?;
            return Ok(());
        }

        let mut guard = store::FASTEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        if let Some(f) = guard
            .iter_mut()
            .find(|f| f.id == id && f.benutzer_id == current_user.id)
        {
            f.beginn = beginn;
            f.ende = ende;
        }
    }

    Ok(())
}

#[server(
    DeleteFasten,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "delete_fasten"
)]
pub async fn delete_fasten(id: Uuid) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query("DELETE FROM fasten WHERE id = $1 AND benutzer_id = $2")
                .bind(id)
                .bind(current_user.id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Delete fehlgeschlagen: {e}")))?;
            return Ok(());
        }

        let mut guard = store::FASTEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.retain(|f| !(f.id == id && f.benutzer_id == current_user.id));
    }

    Ok(())
}
