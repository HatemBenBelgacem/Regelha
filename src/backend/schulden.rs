#[cfg(feature = "ssr")]
use chrono::Local;
use chrono::NaiveDate;
#[cfg(feature = "ssr")]
use crate::backend::kontakte::anzeigename;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Eine Schuld gegenüber einem Kontakt. Die Tabelle enthält nur den Kontakt;
/// die einzelnen Beträge stehen in den Schuldenpositionen (Beziehung Schuld - Schuldenposition 1:N).
/// Pro Kontakt gibt es höchstens eine Schuld.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Schuld {
    pub id: Uuid,
    /// Id des zugehörigen Kontakts. Leer nur bei Altdaten ohne Kontakt.
    pub kontakt_id: Option<Uuid>,
    /// Anzeigename des Kontakts, z.B. "Max Muster". Wird beim Lesen aus dem Kontakt geholt.
    pub kontakt: Option<String>,
    pub benutzer_id: Uuid,
    /// Summe aller Schuldenpositionen (wird beim Lesen berechnet).
    pub total_schulden: f64,
    /// Summe aller Rückzahlungen an diesen Kontakt (wird beim Lesen berechnet).
    pub total_rueckzahlungen: f64,
}

impl Schuld {
    /// Restbetrag = Schulden minus Rückzahlungen. Ein negativer Wert bedeutet Guthaben.
    pub fn restbetrag(&self) -> f64 {
        self.total_schulden - self.total_rueckzahlungen
    }

    /// Name für die Anzeige.
    pub fn name(&self) -> String {
        self.kontakt.clone().unwrap_or_else(|| "Ohne Kontakt".to_string())
    }
}

/// Eine Position einer Schuld mit Datum und Betrag in CHF.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Schuldenposition {
    pub id: Uuid,
    pub schuld_id: Uuid,
    pub datum: NaiveDate,
    pub betrag: f64,
    pub benutzer_id: Uuid,
}

#[cfg(feature = "ssr")]
const KONTAKT_UNGUELTIG: &str = "Bitte einen gültigen Kontakt auswählen.";
#[cfg(feature = "ssr")]
const KONTAKT_HAT_SCHULD: &str = "Für diesen Kontakt gibt es bereits eine Schuld.";
#[cfg(feature = "ssr")]
const SCHULD_UNGUELTIG: &str = "Diese Schuld wurde nicht gefunden.";
#[cfg(feature = "ssr")]
const LOCK_FEHLER: &str = "Speicher-Lock fehlgeschlagen";

/// Interner Datensatz der In-Memory-Tabelle `schulden` (nur Kontakt).
#[cfg(feature = "ssr")]
#[derive(Clone, Debug)]
pub(crate) struct SchuldEintrag {
    pub id: Uuid,
    pub kontakt_id: Option<Uuid>,
    pub benutzer_id: Uuid,
}

#[cfg(feature = "ssr")]
pub(crate) mod store {
    use super::*;
    use crate::backend::auth::default_admin_uuid;
    use crate::backend::kontakte::store::{MAX_MUSTER_ID, SARA_BEISPIEL_ID};
    use std::sync::{LazyLock, Mutex};

    const SCHULD_MAX_ID: Uuid = Uuid::from_u128(0x5c4d_0000_0000_4000_8000_0000_0000_0001);
    const SCHULD_SARA_ID: Uuid = Uuid::from_u128(0x5c4d_0000_0000_4000_8000_0000_0000_0002);

    pub static SCHULDEN_MEMORY: LazyLock<Mutex<Vec<SchuldEintrag>>> = LazyLock::new(|| {
        let admin_id = default_admin_uuid();
        Mutex::new(vec![
            SchuldEintrag { id: SCHULD_MAX_ID, kontakt_id: Some(MAX_MUSTER_ID), benutzer_id: admin_id },
            SchuldEintrag { id: SCHULD_SARA_ID, kontakt_id: Some(SARA_BEISPIEL_ID), benutzer_id: admin_id },
        ])
    });

    pub static SCHULDENPOSITIONEN_MEMORY: LazyLock<Mutex<Vec<Schuldenposition>>> =
        LazyLock::new(|| {
            let today = Local::now().date_naive();
            let admin_id = default_admin_uuid();
            let position = |schuld_id: Uuid, tage: i64, betrag: f64| Schuldenposition {
                id: Uuid::new_v4(),
                schuld_id,
                datum: today - chrono::Duration::days(tage),
                betrag,
                benutzer_id: admin_id,
            };
            Mutex::new(vec![
                position(SCHULD_MAX_ID, 40, 300.00),
                position(SCHULD_MAX_ID, 0, 200.00),
                position(SCHULD_SARA_ID, 0, 240.00),
            ])
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


/// Baut im In-Memory-Modus die Schulden eines Benutzers mit Kontaktnamen und Summen.
#[cfg(feature = "ssr")]
fn schulden_memory(benutzer_id: Uuid) -> Result<Vec<Schuld>, ServerFnError> {
    let kontakte = crate::backend::kontakte::store::KONTAKTE_MEMORY
        .lock()
        .map_err(|_| ServerFnError::new(LOCK_FEHLER))?
        .to_vec();
    let positionen = store::SCHULDENPOSITIONEN_MEMORY
        .lock()
        .map_err(|_| ServerFnError::new(LOCK_FEHLER))?
        .to_vec();
    let rueckzahlungen = crate::backend::rueckzahlungen::store::RUECKZAHLUNGEN_MEMORY
        .lock()
        .map_err(|_| ServerFnError::new(LOCK_FEHLER))?
        .to_vec();
    let guard = store::SCHULDEN_MEMORY
        .lock()
        .map_err(|_| ServerFnError::new(LOCK_FEHLER))?;

    let mut liste: Vec<Schuld> = guard
        .iter()
        .filter(|s| s.benutzer_id == benutzer_id)
        .map(|s| Schuld {
            id: s.id,
            kontakt_id: s.kontakt_id,
            kontakt: s.kontakt_id.and_then(|kid| {
                kontakte
                    .iter()
                    .find(|k| k.id == kid && k.benutzer_id == benutzer_id)
                    .map(|k| anzeigename(&k.vorname, &k.nachname))
            }),
            benutzer_id: s.benutzer_id,
            total_schulden: positionen
                .iter()
                .filter(|p| p.schuld_id == s.id)
                .map(|p| p.betrag)
                .sum::<f64>()
                + 0.0, // leere Summe ist -0.0, so wird daraus 0.0
            total_rueckzahlungen: rueckzahlungen
                .iter()
                .filter(|r| Some(r.kontakt_id) == s.kontakt_id && r.benutzer_id == benutzer_id)
                .map(|r| r.betrag)
                .sum::<f64>()
                + 0.0,
        })
        .collect();
    liste.sort_by_key(|s| s.name());
    Ok(liste)
}

/// Prüft, ob die Schuld existiert und dem Benutzer gehört.
#[cfg(feature = "ssr")]
async fn pruefe_schuld(schuld_id: Uuid, benutzer_id: Uuid) -> Result<(), ServerFnError> {
    use sqlx::PgPool;

    let vorhanden = if let Some(pool) = use_context::<PgPool>() {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM schulden WHERE id = $1 AND benutzer_id = $2")
            .bind(schuld_id)
            .bind(benutzer_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Fehler (schulden): {e}")))?
            > 0
    } else {
        store::SCHULDEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new(LOCK_FEHLER))?
            .iter()
            .any(|s| s.id == schuld_id && s.benutzer_id == benutzer_id)
    };

    if vorhanden {
        Ok(())
    } else {
        Err(ServerFnError::new(SCHULD_UNGUELTIG))
    }
}

/// Prüft, ob der Kontakt gültig ist und noch keine andere Schuld hat.
/// `ausser` ist die Schuld, die gerade bearbeitet wird (sie darf den Kontakt behalten).
#[cfg(feature = "ssr")]
async fn pruefe_kontakt_frei(
    kontakt_id: Uuid,
    benutzer_id: Uuid,
    ausser: Option<Uuid>,
) -> Result<(), ServerFnError> {
    use sqlx::PgPool;

    let belegt = if let Some(pool) = use_context::<PgPool>() {
        kontakt_name_db(&pool, kontakt_id, benutzer_id).await?;
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM schulden WHERE kontakt_id = $1 AND benutzer_id = $2 AND ($3::uuid IS NULL OR id <> $3)",
        )
        .bind(kontakt_id)
        .bind(benutzer_id)
        .bind(ausser)
        .fetch_one(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("DB-Fehler (schulden): {e}")))?
            > 0
    } else {
        kontakt_name_memory(kontakt_id, benutzer_id)?;
        store::SCHULDEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new(LOCK_FEHLER))?
            .iter()
            .any(|s| {
                s.kontakt_id == Some(kontakt_id)
                    && s.benutzer_id == benutzer_id
                    && Some(s.id) != ausser
            })
    };

    if belegt {
        Err(ServerFnError::new(KONTAKT_HAT_SCHULD))
    } else {
        Ok(())
    }
}

/// Alle Schulden des Benutzers mit Total Schulden und Rückzahlungen je Kontakt.
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
                SELECT s.id, s.kontakt_id, s.benutzer_id, k.vorname, k.nachname,
                       COALESCE((SELECT SUM(p.betrag) FROM schuldenpositionen p
                                 WHERE p.schuld_id = s.id), 0)::float8 AS total_schulden,
                       COALESCE((SELECT SUM(r.betrag) FROM rueckzahlungen r
                                 WHERE r.kontakt_id = s.kontakt_id AND r.benutzer_id = s.benutzer_id), 0)::float8
                                 AS total_rueckzahlungen
                FROM schulden s
                LEFT JOIN kontakte k ON s.kontakt_id = k.id
                WHERE s.benutzer_id = $1
                "#,
            )
            .bind(current_user.id)
            .fetch_all(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Fehler (schulden): {e}")))?;

            let mut liste: Vec<Schuld> = rows
                .into_iter()
                .map(|row| {
                    let vorname: Option<String> = row.get("vorname");
                    let nachname: Option<String> = row.get("nachname");
                    Schuld {
                        id: row.get("id"),
                        kontakt_id: row.get("kontakt_id"),
                        kontakt: vorname.map(|v| anzeigename(&v, nachname.as_deref().unwrap_or(""))),
                        benutzer_id: row.get("benutzer_id"),
                        total_schulden: row.get("total_schulden"),
                        total_rueckzahlungen: row.get("total_rueckzahlungen"),
                    }
                })
                .collect();
            liste.sort_by_key(|s| s.name());
            return Ok(liste);
        }

        schulden_memory(current_user.id)
    }
}

/// Legt eine neue Schuld für einen Kontakt an (pro Kontakt nur eine Schuld).
#[server(
    AddSchuld,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "add_schuld"
)]
pub async fn add_schuld(kontakt_id: Uuid) -> Result<Uuid, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let current_user = require_current_user().await?;
        pruefe_kontakt_frei(kontakt_id, current_user.id, None).await?;
        let id = Uuid::new_v4();

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query("INSERT INTO schulden (id, kontakt_id, benutzer_id) VALUES ($1, $2, $3)")
                .bind(id)
                .bind(kontakt_id)
                .bind(current_user.id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Insert fehlgeschlagen: {e}")))?;
            return Ok(id);
        }

        store::SCHULDEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new(LOCK_FEHLER))?
            .push(SchuldEintrag {
                id,
                kontakt_id: Some(kontakt_id),
                benutzer_id: current_user.id,
            });
        Ok(id)
    }

    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

/// Ändert den Kontakt einer Schuld.
#[server(
    UpdateSchuld,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "update_schuld"
)]
pub async fn update_schuld(id: Uuid, kontakt_id: Uuid) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let current_user = require_current_user().await?;
        pruefe_schuld(id, current_user.id).await?;
        pruefe_kontakt_frei(kontakt_id, current_user.id, Some(id)).await?;

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query("UPDATE schulden SET kontakt_id = $1 WHERE id = $2 AND benutzer_id = $3")
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
            .map_err(|_| ServerFnError::new(LOCK_FEHLER))?;
        if let Some(s) = guard
            .iter_mut()
            .find(|s| s.id == id && s.benutzer_id == current_user.id)
        {
            s.kontakt_id = Some(kontakt_id);
        }
    }

    Ok(())
}

/// Löscht eine Schuld samt allen Schuldenpositionen. Die Rückzahlungen bleiben beim Kontakt.
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
            // Die Positionen werden per ON DELETE CASCADE mitgelöscht
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
            .map_err(|_| ServerFnError::new(LOCK_FEHLER))?;
        let vorher = guard.len();
        guard.retain(|s| !(s.id == id && s.benutzer_id == current_user.id));
        if guard.len() < vorher {
            store::SCHULDENPOSITIONEN_MEMORY
                .lock()
                .map_err(|_| ServerFnError::new(LOCK_FEHLER))?
                .retain(|p| p.schuld_id != id);
        }
    }

    Ok(())
}

/// Alle Positionen einer Schuld, neueste zuerst.
#[server(
    GetSchuldenpositionen,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "get_schuldenpositionen"
)]
pub async fn get_schuldenpositionen(schuld_id: Uuid) -> Result<Vec<Schuldenposition>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::{PgPool, Row};

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            let rows = sqlx::query(
                r#"
                SELECT id, schuld_id, datum, betrag, benutzer_id
                FROM schuldenpositionen
                WHERE schuld_id = $1 AND benutzer_id = $2
                ORDER BY datum DESC, betrag DESC
                "#,
            )
            .bind(schuld_id)
            .bind(current_user.id)
            .fetch_all(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Fehler (schuldenpositionen): {e}")))?;

            return Ok(rows
                .into_iter()
                .map(|row| Schuldenposition {
                    id: row.get("id"),
                    schuld_id: row.get("schuld_id"),
                    datum: row.get("datum"),
                    betrag: row.get("betrag"),
                    benutzer_id: row.get("benutzer_id"),
                })
                .collect());
        }

        let mut liste: Vec<Schuldenposition> = store::SCHULDENPOSITIONEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new(LOCK_FEHLER))?
            .iter()
            .filter(|p| p.schuld_id == schuld_id && p.benutzer_id == current_user.id)
            .cloned()
            .collect();
        liste.sort_by(|a, b| {
            b.datum
                .cmp(&a.datum)
                .then_with(|| b.betrag.total_cmp(&a.betrag))
        });
        Ok(liste)
    }
}

/// Fügt einer Schuld eine neue Position hinzu.
#[server(
    AddSchuldenposition,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "add_schuldenposition"
)]
pub async fn add_schuldenposition(
    schuld_id: Uuid,
    datum: String,
    betrag: f64,
) -> Result<Schuldenposition, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let datum = parse_datum(&datum)?;
        pruefe_betrag(betrag)?;
        let current_user = require_current_user().await?;
        pruefe_schuld(schuld_id, current_user.id).await?;

        let neue = Schuldenposition {
            id: Uuid::new_v4(),
            schuld_id,
            datum,
            betrag,
            benutzer_id: current_user.id,
        };

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query(
                r#"
                INSERT INTO schuldenpositionen (id, schuld_id, datum, betrag, benutzer_id)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(neue.id)
            .bind(neue.schuld_id)
            .bind(neue.datum)
            .bind(neue.betrag)
            .bind(neue.benutzer_id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Insert fehlgeschlagen: {e}")))?;
            return Ok(neue);
        }

        store::SCHULDENPOSITIONEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new(LOCK_FEHLER))?
            .push(neue.clone());
        Ok(neue)
    }

    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

/// Ändert Datum und Betrag einer Position.
#[server(
    UpdateSchuldenposition,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "update_schuldenposition"
)]
pub async fn update_schuldenposition(
    id: Uuid,
    datum: String,
    betrag: f64,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let datum = parse_datum(&datum)?;
        pruefe_betrag(betrag)?;
        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query(
                "UPDATE schuldenpositionen SET datum = $1, betrag = $2 WHERE id = $3 AND benutzer_id = $4",
            )
            .bind(datum)
            .bind(betrag)
            .bind(id)
            .bind(current_user.id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Update fehlgeschlagen: {e}")))?;
            return Ok(());
        }

        let mut guard = store::SCHULDENPOSITIONEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new(LOCK_FEHLER))?;
        if let Some(p) = guard
            .iter_mut()
            .find(|p| p.id == id && p.benutzer_id == current_user.id)
        {
            p.datum = datum;
            p.betrag = betrag;
        }
    }

    Ok(())
}

/// Löscht eine Position.
#[server(
    DeleteSchuldenposition,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "delete_schuldenposition"
)]
pub async fn delete_schuldenposition(id: Uuid) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query("DELETE FROM schuldenpositionen WHERE id = $1 AND benutzer_id = $2")
                .bind(id)
                .bind(current_user.id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Delete fehlgeschlagen: {e}")))?;
            return Ok(());
        }

        store::SCHULDENPOSITIONEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new(LOCK_FEHLER))?
            .retain(|p| !(p.id == id && p.benutzer_id == current_user.id));
    }

    Ok(())
}
