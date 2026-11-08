#[cfg(feature = "ssr")]
use chrono::Local;
use chrono::{NaiveDate, NaiveTime};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Ein Fasten an einem Tag: Datum, Beginn und Ende (Uhrzeit).
/// Liegt das Ende vor dem Beginn, dauert das Fasten über Mitternacht.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fasten {
    pub id: Uuid,
    pub datum: NaiveDate,
    pub beginn: NaiveTime,
    pub ende: NaiveTime,
    pub benutzer_id: Uuid,
}

/// Dauer des Fastens in Minuten (berechnet, nicht gespeichert).
/// Liegt das Ende vor dem Beginn, läuft das Fasten über Mitternacht.
pub fn dauer_minuten(beginn: NaiveTime, ende: NaiveTime) -> i64 {
    let differenz = ende.signed_duration_since(beginn).num_minutes();
    if differenz <= 0 {
        differenz + 24 * 60
    } else {
        differenz
    }
}

#[cfg(feature = "ssr")]
pub(crate) mod store {
    use super::*;
    use crate::backend::auth::default_admin_uuid;
    use std::sync::{LazyLock, Mutex};

    pub static FASTEN_MEMORY: LazyLock<Mutex<Vec<Fasten>>> = LazyLock::new(|| {
        let today = Local::now().date_naive();
        let admin_id = default_admin_uuid();
        Mutex::new(vec![Fasten {
            id: Uuid::new_v4(),
            datum: today,
            beginn: NaiveTime::from_hms_opt(20, 0, 0).expect("gültige Beispielzeit"),
            ende: NaiveTime::from_hms_opt(12, 0, 0).expect("gültige Beispielzeit"),
            benutzer_id: admin_id,
        }])
    });
}

/// Liest eine Uhrzeit im Format HH:MM (aus dem Formular) oder HH:MM:SS.
#[cfg(feature = "ssr")]
fn parse_zeit(text: &str) -> Result<NaiveTime, ServerFnError> {
    let text = text.trim();
    NaiveTime::parse_from_str(text, "%H:%M")
        .or_else(|_| NaiveTime::parse_from_str(text, "%H:%M:%S"))
        .map_err(|_| ServerFnError::new("Bitte eine gültige Uhrzeit eingeben."))
}

/// Beginn und Ende dürfen nicht gleich sein, sonst ist die Dauer unklar.
#[cfg(feature = "ssr")]
fn pruefe_zeiten(beginn: NaiveTime, ende: NaiveTime) -> Result<(), ServerFnError> {
    if beginn == ende {
        return Err(ServerFnError::new(
            "Beginn und Ende dürfen nicht gleich sein.",
        ));
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
                SELECT id, datum, beginn, ende, benutzer_id
                FROM fasten
                WHERE benutzer_id = $1
                ORDER BY datum DESC, beginn DESC
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
                    datum: row.get("datum"),
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
        liste.sort_by(|a, b| b.datum.cmp(&a.datum).then_with(|| b.beginn.cmp(&a.beginn)));
        Ok(liste)
    }
}

#[server(
    AddFasten,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "add_fasten"
)]
pub async fn add_fasten(
    datum: String,
    beginn: String,
    ende: String,
) -> Result<Fasten, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use crate::backend::schulden::parse_datum;
        use sqlx::PgPool;

        let datum = parse_datum(&datum)?;
        let beginn = parse_zeit(&beginn)?;
        let ende = parse_zeit(&ende)?;
        pruefe_zeiten(beginn, ende)?;
        let current_user = require_current_user().await?;

        let neues_fasten = Fasten {
            id: Uuid::new_v4(),
            datum,
            beginn,
            ende,
            benutzer_id: current_user.id,
        };

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query(
                r#"
                INSERT INTO fasten (id, datum, beginn, ende, benutzer_id)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(neues_fasten.id)
            .bind(neues_fasten.datum)
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
pub async fn update_fasten(
    id: Uuid,
    datum: String,
    beginn: String,
    ende: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use crate::backend::schulden::parse_datum;
        use sqlx::PgPool;

        let datum = parse_datum(&datum)?;
        let beginn = parse_zeit(&beginn)?;
        let ende = parse_zeit(&ende)?;
        pruefe_zeiten(beginn, ende)?;
        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query(
                "UPDATE fasten SET datum = $1, beginn = $2, ende = $3 WHERE id = $4 AND benutzer_id = $5",
            )
            .bind(datum)
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
            f.datum = datum;
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
