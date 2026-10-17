#[cfg(feature = "ssr")]
use chrono::Local;
use chrono::{Datelike, NaiveDate};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuchungsArt {
    Eingang,
    Ausgang,
}

impl BuchungsArt {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Eingang => "Eingang",
            Self::Ausgang => "Ausgang",
        }
    }

    pub fn from_db_str(s: &str) -> Self {
        match s {
            "Eingang" => Self::Eingang,
            _ => Self::Ausgang,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Buchung {
    pub id: Uuid,
    pub buchungstext: String,
    pub betrag: f64,
    pub art: BuchungsArt,
    pub erstellt_am: NaiveDate,
    pub benutzer_id: Uuid,
    pub benutzer: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FixkostenEintrag {
    pub id: Uuid,
    pub buchungstext: String,
    pub betrag: f64,
    pub benutzer_id: Uuid,
    pub benutzer: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DashboardKennzahlen {
    pub heute_datum: String,
    pub monat_name: String,
    pub tag_im_monat: u32,
    pub tage_im_monat: u32,
    pub ausgaben_heute: f64,
    pub ausgaben_aktueller_monat: f64,
    pub durchschnitt_aktueller_monat: f64,
    pub voraussichtliche_ausgaben_monatsende: f64,
    pub einnahmen_aktueller_monat: f64,
    pub summe_fixkosten: f64,
    pub voraussichtliche_gesamtausgaben_inkl_fixkosten: f64,
}

/// Berechnet die Anzahl der Tage eines gegebenen Monats und Jahres.
pub fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let first_of_next = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap();
    let last_of_current = first_of_next.pred_opt().unwrap();
    last_of_current.day()
}

pub fn german_month_name(month: u32) -> &'static str {
    match month {
        1 => "Januar",
        2 => "Februar",
        3 => "März",
        4 => "April",
        5 => "Mai",
        6 => "Juni",
        7 => "Juli",
        8 => "August",
        9 => "September",
        10 => "Oktober",
        11 => "November",
        12 => "Dezember",
        _ => "",
    }
}

#[cfg(feature = "ssr")]
mod store {
    use super::*;
    use crate::backend::auth::default_admin_uuid;
    use std::sync::{LazyLock, Mutex};

    pub static BUCHUNGEN_MEMORY: LazyLock<Mutex<Vec<Buchung>>> = LazyLock::new(|| {
        let today = Local::now().date_naive();
        let first_of_month =
            NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);
        let admin_id = default_admin_uuid();
        Mutex::new(vec![
            Buchung {
                id: Uuid::new_v4(),
                buchungstext: "Lohn".to_string(),
                betrag: 3450.00,
                art: BuchungsArt::Eingang,
                erstellt_am: first_of_month,
                benutzer_id: admin_id,
                benutzer: "admin".to_string(),
            },
            Buchung {
                id: Uuid::new_v4(),
                buchungstext: "Wocheneinkauf Supermarkt".to_string(),
                betrag: 142.80,
                art: BuchungsArt::Ausgang,
                erstellt_am: today,
                benutzer_id: admin_id,
                benutzer: "admin".to_string(),
            },
            Buchung {
                id: Uuid::new_v4(),
                buchungstext: "Tankstelle".to_string(),
                betrag: 68.50,
                art: BuchungsArt::Ausgang,
                erstellt_am: today,
                benutzer_id: admin_id,
                benutzer: "admin".to_string(),
            },
        ])
    });

    pub static FIXKOSTEN_MEMORY: LazyLock<Mutex<Vec<FixkostenEintrag>>> = LazyLock::new(|| {
        let admin_id = default_admin_uuid();
        Mutex::new(vec![
            FixkostenEintrag {
                id: Uuid::new_v4(),
                buchungstext: "Warmmiete".to_string(),
                betrag: 980.00,
                benutzer_id: admin_id,
                benutzer: "admin".to_string(),
            },
            FixkostenEintrag {
                id: Uuid::new_v4(),
                buchungstext: "Krankenkasse".to_string(),
                betrag: 345.00,
                benutzer_id: admin_id,
                benutzer: "admin".to_string(),
            },
            FixkostenEintrag {
                id: Uuid::new_v4(),
                buchungstext: "Internet & Mobilfunk".to_string(),
                betrag: 59.90,
                benutzer_id: admin_id,
                benutzer: "admin".to_string(),
            },
        ])
    });
}

#[server(GetDashboardKennzahlen, "/api", endpoint = "get_dashboard_kennzahlen")]
pub async fn get_dashboard_kennzahlen() -> Result<DashboardKennzahlen, ServerFnError> {
    // `get_buchungen` und `get_fixkosten` liefern ausschließlich die Einträge des angemeldeten Benutzers
    let buchungen = get_buchungen().await?;
    let fixkosten = get_fixkosten().await?;

    let today = Local::now().date_naive();
    let current_year = today.year();
    let current_month = today.month();
    let tag_im_monat = today.day().max(1);
    let tage_im_monat = days_in_month(current_year, current_month);

    let ausgaben_heute: f64 = buchungen
        .iter()
        .filter(|b| b.art == BuchungsArt::Ausgang && b.erstellt_am == today)
        .map(|b| b.betrag)
        .sum();

    let ausgaben_aktueller_monat: f64 = buchungen
        .iter()
        .filter(|b| {
            b.art == BuchungsArt::Ausgang
                && b.erstellt_am.year() == current_year
                && b.erstellt_am.month() == current_month
        })
        .map(|b| b.betrag)
        .sum();

    let einnahmen_aktueller_monat: f64 = buchungen
        .iter()
        .filter(|b| {
            b.art == BuchungsArt::Eingang
                && b.erstellt_am.year() == current_year
                && b.erstellt_am.month() == current_month
        })
        .map(|b| b.betrag)
        .sum();

    let summe_fixkosten: f64 = fixkosten.iter().map(|f| f.betrag).sum();

    let durchschnitt_aktueller_monat = ausgaben_aktueller_monat / (tag_im_monat as f64);

    let voraussichtliche_ausgaben_monatsende =
        durchschnitt_aktueller_monat * (tage_im_monat as f64);

    let voraussichtliche_gesamtausgaben_inkl_fixkosten =
        voraussichtliche_ausgaben_monatsende + summe_fixkosten;

    Ok(DashboardKennzahlen {
        heute_datum: today.format("%d.%m.%Y").to_string(),
        monat_name: format!("{} {}", german_month_name(current_month), current_year),
        tag_im_monat,
        tage_im_monat,
        ausgaben_heute,
        ausgaben_aktueller_monat,
        durchschnitt_aktueller_monat,
        voraussichtliche_ausgaben_monatsende,
        einnahmen_aktueller_monat,
        summe_fixkosten,
        voraussichtliche_gesamtausgaben_inkl_fixkosten,
    })
}

#[server(GetBuchungen, "/api", endpoint = "get_buchungen")]
pub async fn get_buchungen() -> Result<Vec<Buchung>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::{PgPool, Row};

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            let rows = sqlx::query(
                r#"
                SELECT
                    b.id,
                    b.buchungstext,
                    b.betrag,
                    b.art,
                    b.erstellt_am,
                    b.benutzer_id,
                    u.benutzername AS benutzer
                FROM buchungen b
                INNER JOIN benutzer u ON b.benutzer_id = u.id
                WHERE b.benutzer_id = $1
                ORDER BY b.erstellt_am DESC
                "#,
            )
            .bind(current_user.id)
            .fetch_all(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Fehler (buchungen): {e}")))?;

            let list = rows
                .into_iter()
                .map(|row| {
                    let art_str: String = row.get("art");
                    Buchung {
                        id: row.get("id"),
                        buchungstext: row.get("buchungstext"),
                        betrag: row.get("betrag"),
                        art: BuchungsArt::from_db_str(&art_str),
                        erstellt_am: row.get("erstellt_am"),
                        benutzer_id: row.get("benutzer_id"),
                        benutzer: row.get("benutzer"),
                    }
                })
                .collect();
            return Ok(list);
        }

        let guard = store::BUCHUNGEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        let filtered = guard
            .iter()
            .filter(|b| b.benutzer_id == current_user.id || b.benutzer == current_user.benutzername)
            .cloned()
            .collect();
        Ok(filtered)
    }
}

#[server(AddBuchung, "/api", endpoint = "add_buchung")]
pub async fn add_buchung(
    buchungstext: String,
    betrag: f64,
    art: BuchungsArt,
    datum: Option<String>,
) -> Result<Buchung, ServerFnError> {
    let trimmed = buchungstext.trim().to_string();
    if trimmed.is_empty() {
        return Err(ServerFnError::new("Bitte einen Buchungstext eingeben."));
    }
    if betrag <= 0.0 || !betrag.is_finite() {
        return Err(ServerFnError::new(
            "Bitte einen gültigen positiven Betrag eingeben.",
        ));
    }

    let buchungs_datum = match datum.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
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

        let neue_buchung = Buchung {
            id: Uuid::new_v4(),
            buchungstext: trimmed,
            betrag: betrag.abs(),
            art,
            erstellt_am: buchungs_datum,
            benutzer_id: current_user.id,
            benutzer: current_user.benutzername,
        };

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query(
                r#"
                INSERT INTO buchungen (id, buchungstext, betrag, art, erstellt_am, benutzer_id)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(neue_buchung.id)
            .bind(&neue_buchung.buchungstext)
            .bind(neue_buchung.betrag)
            .bind(neue_buchung.art.as_str())
            .bind(neue_buchung.erstellt_am)
            .bind(neue_buchung.benutzer_id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Insert fehlgeschlagen: {e}")))?;

            return Ok(neue_buchung);
        }

        let mut guard = store::BUCHUNGEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.push(neue_buchung.clone());
        return Ok(neue_buchung);
    }

    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

#[server(DeleteBuchung, "/api", endpoint = "delete_buchung")]
pub async fn delete_buchung(id: Uuid) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query("DELETE FROM buchungen WHERE id = $1 AND benutzer_id = $2")
                .bind(id)
                .bind(current_user.id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Delete fehlgeschlagen: {e}")))?;
            return Ok(());
        }

        let mut guard = store::BUCHUNGEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.retain(|b| !(b.id == id && (b.benutzer_id == current_user.id || b.benutzer == current_user.benutzername)));
    }

    Ok(())
}

#[server(GetFixkosten, "/api", endpoint = "get_fixkosten")]
pub async fn get_fixkosten() -> Result<Vec<FixkostenEintrag>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::{PgPool, Row};

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            let rows = sqlx::query(
                r#"
                SELECT
                    f.id,
                    f.buchungstext,
                    f.betrag,
                    f.benutzer_id,
                    u.benutzername AS benutzer
                FROM fixkosten f
                INNER JOIN benutzer u ON f.benutzer_id = u.id
                WHERE f.benutzer_id = $1
                "#,
            )
            .bind(current_user.id)
            .fetch_all(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Fehler (fixkosten): {e}")))?;

            let list = rows
                .into_iter()
                .map(|row| FixkostenEintrag {
                    id: row.get("id"),
                    buchungstext: row.get("buchungstext"),
                    betrag: row.get("betrag"),
                    benutzer_id: row.get("benutzer_id"),
                    benutzer: row.get("benutzer"),
                })
                .collect();
            return Ok(list);
        }

        let guard = store::FIXKOSTEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        let filtered = guard
            .iter()
            .filter(|f| f.benutzer_id == current_user.id || f.benutzer == current_user.benutzername)
            .cloned()
            .collect();
        Ok(filtered)
    }
}

#[server(AddFixkosten, "/api", endpoint = "add_fixkosten")]
pub async fn add_fixkosten(
    buchungstext: String,
    betrag: f64,
) -> Result<FixkostenEintrag, ServerFnError> {
    let trimmed = buchungstext.trim().to_string();
    if trimmed.is_empty() {
        return Err(ServerFnError::new("Bitte einen Buchungstext eingeben."));
    }
    if betrag <= 0.0 || !betrag.is_finite() {
        return Err(ServerFnError::new(
            "Bitte einen gültigen positiven Betrag eingeben.",
        ));
    }

    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let current_user = require_current_user().await?;

        let eintrag = FixkostenEintrag {
            id: Uuid::new_v4(),
            buchungstext: trimmed,
            betrag: betrag.abs(),
            benutzer_id: current_user.id,
            benutzer: current_user.benutzername,
        };

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query(
                "INSERT INTO fixkosten (id, buchungstext, betrag, benutzer_id) VALUES ($1, $2, $3, $4)",
            )
            .bind(eintrag.id)
            .bind(&eintrag.buchungstext)
            .bind(eintrag.betrag)
            .bind(eintrag.benutzer_id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Insert fehlgeschlagen: {e}")))?;

            return Ok(eintrag);
        }

        let mut guard = store::FIXKOSTEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.push(eintrag.clone());
        return Ok(eintrag);
    }

    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

#[server(DeleteFixkosten, "/api", endpoint = "delete_fixkosten")]
pub async fn delete_fixkosten(id: Uuid) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query("DELETE FROM fixkosten WHERE id = $1 AND benutzer_id = $2")
                .bind(id)
                .bind(current_user.id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Delete fehlgeschlagen: {e}")))?;
            return Ok(());
        }

        let mut guard = store::FIXKOSTEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.retain(|f| !(f.id == id && (f.benutzer_id == current_user.id || f.benutzer == current_user.benutzername)));
    }

    Ok(())
}
