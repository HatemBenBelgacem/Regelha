use chrono::{Datelike, Local, NaiveDate};
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
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FixkostenEintrag {
    pub id: Uuid,
    pub buchungstext: String,
    pub betrag: f64,
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
    use std::sync::{LazyLock, Mutex};

    pub static BUCHUNGEN_MEMORY: LazyLock<Mutex<Vec<Buchung>>> = LazyLock::new(|| {
        let today = Local::now().date_naive();
        let first_of_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);
        Mutex::new(vec![
            Buchung {
                id: Uuid::new_v4(),
                buchungstext: "Gehalt".to_string(),
                betrag: 3450.00,
                art: BuchungsArt::Eingang,
                erstellt_am: first_of_month,
            },
            Buchung {
                id: Uuid::new_v4(),
                buchungstext: "Wocheneinkauf Supermarkt".to_string(),
                betrag: 142.80,
                art: BuchungsArt::Ausgang,
                erstellt_am: today,
            },
            Buchung {
                id: Uuid::new_v4(),
                buchungstext: "Tankstelle".to_string(),
                betrag: 68.50,
                art: BuchungsArt::Ausgang,
                erstellt_am: today,
            },
        ])
    });

    pub static FIXKOSTEN_MEMORY: LazyLock<Mutex<Vec<FixkostenEintrag>>> = LazyLock::new(|| {
        Mutex::new(vec![
            FixkostenEintrag {
                id: Uuid::new_v4(),
                buchungstext: "Warmmiete".to_string(),
                betrag: 980.00,
            },
            FixkostenEintrag {
                id: Uuid::new_v4(),
                buchungstext: "Strom & Gas Abschlag".to_string(),
                betrag: 135.00,
            },
            FixkostenEintrag {
                id: Uuid::new_v4(),
                buchungstext: "Internet & Mobilfunk".to_string(),
                betrag: 59.90,
            },
        ])
    });
}

#[server(GetDashboardKennzahlen, "/api")]
pub async fn get_dashboard_kennzahlen() -> Result<DashboardKennzahlen, ServerFnError> {
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

    // Durchschnittliche Ausgaben des aktuellen Monats (Total Ausgaben / Anzahl Tage bis heute)
    let durchschnitt_aktueller_monat = ausgaben_aktueller_monat / (tag_im_monat as f64);

    // Voraussichtliche Ausgaben bis Ende Monat (Hochrechnung auf alle Tage des Monats)
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

#[server(GetBuchungen, "/api")]
pub async fn get_buchungen() -> Result<Vec<Buchung>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::{PgPool, Row};

        if let Some(pool) = use_context::<PgPool>() {
            let rows = sqlx::query(
                "SELECT id, buchungstext, betrag, art, erstellt_am FROM buchungen ORDER BY erstellt_am DESC",
            )
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
                    }
                })
                .collect();
            return Ok(list);
        }

        let guard = store::BUCHUNGEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        Ok(guard.clone())
    }
}

#[server(AddBuchung, "/api")]
pub async fn add_buchung(
    buchungstext: String,
    betrag: f64,
    art: BuchungsArt,
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

    let today = Local::now().date_naive();
    let neue_buchung = Buchung {
        id: Uuid::new_v4(),
        buchungstext: trimmed,
        betrag: betrag.abs(),
        art,
        erstellt_am: today,
    };

    #[cfg(feature = "ssr")]
    {
        use sqlx::PgPool;

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query(
                "INSERT INTO buchungen (id, buchungstext, betrag, art, erstellt_am) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(neue_buchung.id)
            .bind(&neue_buchung.buchungstext)
            .bind(neue_buchung.betrag)
            .bind(neue_buchung.art.as_str())
            .bind(neue_buchung.erstellt_am)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Insert fehlgeschlagen: {e}")))?;

            return Ok(neue_buchung);
        }

        let mut guard = store::BUCHUNGEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.push(neue_buchung.clone());
    }

    Ok(neue_buchung)
}

#[server(DeleteBuchung, "/api")]
pub async fn delete_buchung(id: Uuid) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::PgPool;

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query("DELETE FROM buchungen WHERE id = $1")
                .bind(id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Delete fehlgeschlagen: {e}")))?;
            return Ok(());
        }

        let mut guard = store::BUCHUNGEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.retain(|b| b.id != id);
    }

    Ok(())
}

#[server(GetFixkosten, "/api")]
pub async fn get_fixkosten() -> Result<Vec<FixkostenEintrag>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::{PgPool, Row};

        if let Some(pool) = use_context::<PgPool>() {
            let rows = sqlx::query("SELECT id, buchungstext, betrag FROM fixkosten")
                .fetch_all(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Fehler (fixkosten): {e}")))?;

            let list = rows
                .into_iter()
                .map(|row| FixkostenEintrag {
                    id: row.get("id"),
                    buchungstext: row.get("buchungstext"),
                    betrag: row.get("betrag"),
                })
                .collect();
            return Ok(list);
        }

        let guard = store::FIXKOSTEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        Ok(guard.clone())
    }
}

#[server(AddFixkosten, "/api")]
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

    let eintrag = FixkostenEintrag {
        id: Uuid::new_v4(),
        buchungstext: trimmed,
        betrag: betrag.abs(),
    };

    #[cfg(feature = "ssr")]
    {
        use sqlx::PgPool;

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query("INSERT INTO fixkosten (id, buchungstext, betrag) VALUES ($1, $2, $3)")
                .bind(eintrag.id)
                .bind(&eintrag.buchungstext)
                .bind(eintrag.betrag)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Insert fehlgeschlagen: {e}")))?;

            return Ok(eintrag);
        }

        let mut guard = store::FIXKOSTEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.push(eintrag.clone());
    }

    Ok(eintrag)
}

#[server(DeleteFixkosten, "/api")]
pub async fn delete_fixkosten(id: Uuid) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::PgPool;

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query("DELETE FROM fixkosten WHERE id = $1")
                .bind(id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Delete fehlgeschlagen: {e}")))?;
            return Ok(());
        }

        let mut guard = store::FIXKOSTEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.retain(|f| f.id != id);
    }

    Ok(())
}
