#[cfg(feature = "ssr")]
use chrono::Local;
use chrono::{Datelike, NaiveDate};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::backend::kategorien::{kategorie_name, Kategorie};

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

/// Wie oft eine Fixkosten-Position bezahlt wird. Der erfasste Betrag gilt pro Periode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Zahlungsperiode {
    Monatlich,
    Vierteljaehrlich,
    Halbjaehrlich,
    Jaehrlich,
}

impl Zahlungsperiode {
    pub const ALLE: [Self; 4] = [
        Self::Monatlich,
        Self::Vierteljaehrlich,
        Self::Halbjaehrlich,
        Self::Jaehrlich,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Monatlich => "Monatlich",
            Self::Vierteljaehrlich => "Vierteljaehrlich",
            Self::Halbjaehrlich => "Halbjaehrlich",
            Self::Jaehrlich => "Jaehrlich",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Monatlich => "Monatlich",
            Self::Vierteljaehrlich => "Vierteljährlich",
            Self::Halbjaehrlich => "Halbjährlich",
            Self::Jaehrlich => "Jährlich",
        }
    }

    pub fn from_db_str(s: &str) -> Self {
        match s {
            "Vierteljaehrlich" => Self::Vierteljaehrlich,
            "Halbjaehrlich" => Self::Halbjaehrlich,
            "Jaehrlich" => Self::Jaehrlich,
            _ => Self::Monatlich,
        }
    }

    /// Anzahl Monate, die eine Zahlung abdeckt.
    pub fn monate(&self) -> u32 {
        match self {
            Self::Monatlich => 1,
            Self::Vierteljaehrlich => 3,
            Self::Halbjaehrlich => 6,
            Self::Jaehrlich => 12,
        }
    }
}

/// Art eines wiederkehrenden Postens: Fixkosten (Ausgabe) oder Fixeinnahme (Einnahme).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FixkostenArt {
    Ausgabe,
    Einnahme,
}

impl FixkostenArt {
    pub const ALLE: [Self; 2] = [Self::Ausgabe, Self::Einnahme];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ausgabe => "Ausgabe",
            Self::Einnahme => "Einnahme",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Ausgabe => "Fixkosten",
            Self::Einnahme => "Fixeinnahme",
        }
    }

    pub fn from_db_str(s: &str) -> Self {
        match s {
            "Einnahme" => Self::Einnahme,
            _ => Self::Ausgabe,
        }
    }
}

/// Status eines Fixkosten-Vertrags.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FixkostenStatus {
    /// Vertrag läuft.
    Aktiv,
    /// Vertrag ist gekündigt, läuft aber bis zum Ablauf der Kündigungsfrist weiter.
    Gekuendigt,
    /// Vertrag ist beendet, es fallen keine Kosten mehr an.
    Beendet,
}

impl FixkostenStatus {
    pub const ALLE: [Self; 3] = [Self::Aktiv, Self::Gekuendigt, Self::Beendet];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Aktiv => "Aktiv",
            Self::Gekuendigt => "Gekuendigt",
            Self::Beendet => "Beendet",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Aktiv => "Aktiv",
            Self::Gekuendigt => "Gekündigt",
            Self::Beendet => "Beendet",
        }
    }

    pub fn from_db_str(s: &str) -> Self {
        match s {
            "Gekuendigt" => Self::Gekuendigt,
            "Beendet" => Self::Beendet,
            _ => Self::Aktiv,
        }
    }

    /// Verursacht ein Vertrag mit diesem Status (noch) Kosten?
    pub fn verursacht_kosten(&self) -> bool {
        !matches!(self, Self::Beendet)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FixkostenEintrag {
    pub id: Uuid,
    pub buchungstext: String,
    /// Betrag pro Zahlungsperiode in CHF
    pub betrag: f64,
    pub beginndatum: NaiveDate,
    /// Kündigungsfrist in Monaten (0 = jederzeit kündbar)
    pub kuendigungsfrist_monate: i32,
    pub zahlungsperiode: Zahlungsperiode,
    pub status: FixkostenStatus,
    /// Fixkosten (Ausgabe) oder Fixeinnahme (Einnahme)
    pub art: FixkostenArt,
    /// Kategorie aus der Tabelle `kategorien`; None = ohne Kategorie
    pub kategorie_id: Option<Uuid>,
    pub benutzer_id: Uuid,
    pub benutzer: String,
}

/// Die Angaben aus dem Formular für eine Fixkosten- oder Fixeinnahme-Position.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FixkostenDaten {
    pub buchungstext: String,
    /// Betrag pro Zahlungsperiode in CHF
    pub betrag: f64,
    /// Datum im Format JJJJ-MM-TT
    pub beginndatum: String,
    pub kuendigungsfrist_monate: i32,
    pub zahlungsperiode: Zahlungsperiode,
    pub status: FixkostenStatus,
    pub art: FixkostenArt,
    pub kategorie_id: Option<Uuid>,
}

impl FixkostenEintrag {
    /// Auf einen Monat umgerechneter Betrag (z.B. Jahresbetrag / 12).
    pub fn monatsbetrag(&self) -> f64 {
        self.betrag / self.zahlungsperiode.monate() as f64
    }

    /// Ist dieser Eintrag eine Fixeinnahme (und keine Fixkosten)?
    pub fn ist_einnahme(&self) -> bool {
        self.art == FixkostenArt::Einnahme
    }

    /// Fallen für diesen Eintrag im angegebenen Monat Kosten an?
    /// (Vertrag hat bis Ende des Monats begonnen und ist nicht beendet.)
    pub fn zaehlt_im_monat(&self, jahr: i32, monat: u32) -> bool {
        let beginn = (self.beginndatum.year(), self.beginndatum.month());
        self.status.verursacht_kosten() && beginn <= (jahr, monat)
    }

    /// Wird dieser Eintrag im angegebenen Monat fällig (also bezahlt)?
    /// Die Zahlung kommt im Beginnmonat und danach im Abstand der Zahlungsperiode:
    /// monatlich jeden Monat, vierteljährlich alle 3 Monate, halbjährlich alle 6 Monate, jährlich alle 12 Monate.
    pub fn faellig_im_monat(&self, jahr: i32, monat: u32) -> bool {
        if !self.zaehlt_im_monat(jahr, monat) {
            return false;
        }
        let beginn = self.beginndatum.year() * 12 + self.beginndatum.month() as i32;
        let aktuell = jahr * 12 + monat as i32;
        (aktuell - beginn) % self.zahlungsperiode.monate() as i32 == 0
    }
}

/// Text für eine Kündigungsfrist in Monaten.
pub fn kuendigungsfrist_text(monate: i32) -> String {
    match monate {
        m if m <= 0 => "Keine".to_string(),
        1 => "1 Monat".to_string(),
        m => format!("{m} Monate"),
    }
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
    /// Fixkosten des Monats je Kategorie, absteigend nach Betrag (für das Säulendiagramm)
    pub fixkosten_pro_kategorie: Vec<KategorieSumme>,
}

/// Fixkosten eines Monats für eine Kategorie (Summe der Monatsbeträge). `kategorie_id` None = ohne Kategorie.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KategorieSumme {
    pub kategorie_id: Option<Uuid>,
    pub name: String,
    pub betrag: f64,
}

/// Summiert die Fixkosten eines Monats nach Kategorie. Fixeinnahmen zählen nicht.
/// Die Liste ist nach Betrag absteigend sortiert, bei Gleichstand nach Name.
pub fn berechne_kategorie_summen(
    fixkosten: &[FixkostenEintrag],
    kategorien: &[Kategorie],
    jahr: i32,
    monat: u32,
) -> Vec<KategorieSumme> {
    let mut summen: Vec<KategorieSumme> = Vec::new();
    for f in fixkosten
        .iter()
        .filter(|f| !f.ist_einnahme() && f.zaehlt_im_monat(jahr, monat))
    {
        match summen.iter_mut().find(|s| s.kategorie_id == f.kategorie_id) {
            Some(s) => s.betrag += f.monatsbetrag(),
            None => summen.push(KategorieSumme {
                kategorie_id: f.kategorie_id,
                name: kategorie_name(kategorien, f.kategorie_id),
                betrag: f.monatsbetrag(),
            }),
        }
    }
    summen.sort_by(|a, b| {
        b.betrag
            .total_cmp(&a.betrag)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    summen
}

/// Filter nach Kategorie in der Fixkosten-Liste.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KategorieFilter {
    /// Keine Einschränkung auf eine Kategorie
    #[default]
    Alle,
    /// Nur Einträge ohne Kategorie
    OhneKategorie,
    /// Nur Einträge dieser Kategorie
    Kategorie(Uuid),
}

/// Filter für die Liste der Fixkosten und Fixeinnahmen. Ein nicht gesetztes Feld schränkt nichts ein.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FixkostenFilter {
    pub kategorie: KategorieFilter,
    /// Nur Einträge mit diesem Status (None = alle Status)
    pub status: Option<FixkostenStatus>,
    /// Nur Einträge, die ab diesem Datum beginnen (inklusive)
    pub beginn_von: Option<NaiveDate>,
    /// Nur Einträge, die bis zu diesem Datum beginnen (inklusive)
    pub beginn_bis: Option<NaiveDate>,
}

impl FixkostenFilter {
    /// Ist mindestens ein Filter gesetzt?
    pub fn ist_aktiv(&self) -> bool {
        *self != Self::default()
    }

    /// Passt der Eintrag zu allen gesetzten Filtern?
    pub fn passt(&self, eintrag: &FixkostenEintrag) -> bool {
        let kategorie_passt = match self.kategorie {
            KategorieFilter::Alle => true,
            KategorieFilter::OhneKategorie => eintrag.kategorie_id.is_none(),
            KategorieFilter::Kategorie(id) => eintrag.kategorie_id == Some(id),
        };
        kategorie_passt
            && self.status.is_none_or(|status| eintrag.status == status)
            && self.beginn_von.is_none_or(|von| eintrag.beginndatum >= von)
            && self.beginn_bis.is_none_or(|bis| eintrag.beginndatum <= bis)
    }
}

/// Die Einträge, die zum Filter passen. Die Reihenfolge bleibt erhalten.
pub fn filtere_fixkosten(
    eintraege: &[FixkostenEintrag],
    filter: &FixkostenFilter,
) -> Vec<FixkostenEintrag> {
    eintraege
        .iter()
        .filter(|e| filter.passt(e))
        .cloned()
        .collect()
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
pub(crate) mod store {
    use super::*;
    use crate::backend::auth::default_admin_uuid;
    use crate::backend::kategorien::store::{GESUNDHEIT_ID, KOMMUNIKATION_ID, WOHNEN_ID};
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
                beginndatum: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                kuendigungsfrist_monate: 3,
                zahlungsperiode: Zahlungsperiode::Monatlich,
                status: FixkostenStatus::Aktiv,
                art: FixkostenArt::Ausgabe,
                kategorie_id: Some(WOHNEN_ID),
                benutzer_id: admin_id,
                benutzer: "admin".to_string(),
            },
            FixkostenEintrag {
                id: Uuid::new_v4(),
                buchungstext: "Krankenkasse".to_string(),
                betrag: 345.00,
                beginndatum: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                kuendigungsfrist_monate: 3,
                zahlungsperiode: Zahlungsperiode::Monatlich,
                status: FixkostenStatus::Aktiv,
                art: FixkostenArt::Ausgabe,
                kategorie_id: Some(GESUNDHEIT_ID),
                benutzer_id: admin_id,
                benutzer: "admin".to_string(),
            },
            FixkostenEintrag {
                id: Uuid::new_v4(),
                buchungstext: "Internet & Mobilfunk".to_string(),
                betrag: 59.90,
                beginndatum: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                kuendigungsfrist_monate: 1,
                zahlungsperiode: Zahlungsperiode::Monatlich,
                status: FixkostenStatus::Aktiv,
                art: FixkostenArt::Ausgabe,
                kategorie_id: Some(KOMMUNIKATION_ID),
                benutzer_id: admin_id,
                benutzer: "admin".to_string(),
            },
        ])
    });
}

#[server(
    GetDashboardKennzahlen,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "get_dashboard_kennzahlen"
)]
pub async fn get_dashboard_kennzahlen() -> Result<DashboardKennzahlen, ServerFnError> {
    // `get_buchungen` und `get_fixkosten` liefern ausschließlich die Einträge des angemeldeten Benutzers
    let buchungen = get_buchungen().await?;
    let fixkosten = get_fixkosten().await?;
    let kategorien = crate::backend::kategorien::get_kategorien().await?;

    let today = Local::now().date_naive();
    let current_year = today.year();
    let current_month = today.month();
    let tag_im_monat = today.day().max(1);
    let tage_im_monat = days_in_month(current_year, current_month);

    let ausgaben_heute: f64 = buchungen
        .iter()
        .filter(|b| b.art == BuchungsArt::Ausgang && b.erstellt_am == today)
        .map(|b| b.betrag)
        .fold(0.0, |acc, x| acc + x);

    let ausgaben_aktueller_monat: f64 = buchungen
        .iter()
        .filter(|b| {
            b.art == BuchungsArt::Ausgang
                && b.erstellt_am.year() == current_year
                && b.erstellt_am.month() == current_month
        })
        .map(|b| b.betrag)
        .fold(0.0, |acc, x| acc + x);

    let einnahmen_aktueller_monat: f64 = buchungen
        .iter()
        .filter(|b| {
            b.art == BuchungsArt::Eingang
                && b.erstellt_am.year() == current_year
                && b.erstellt_am.month() == current_month
        })
        .map(|b| b.betrag)
        .fold(0.0, |acc, x| acc + x);

    // Fixeinnahmen sind keine Kosten und werden hier nicht mitgezählt
    let summe_fixkosten: f64 = fixkosten
        .iter()
        .filter(|f| !f.ist_einnahme() && f.zaehlt_im_monat(current_year, current_month))
        .map(FixkostenEintrag::monatsbetrag)
        .sum();

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
        fixkosten_pro_kategorie: berechne_kategorie_summen(
            &fixkosten,
            &kategorien,
            current_year,
            current_month,
        ),
    })
}

#[server(
    GetBuchungen,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "get_buchungen"
)]
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

/// Prüft Buchungstext und Betrag (für Buchungen und Fixkosten).
/// Gibt den bereinigten Buchungstext zurück.
#[cfg(feature = "ssr")]
fn pruefe_text_und_betrag(buchungstext: &str, betrag: f64) -> Result<String, ServerFnError> {
    let trimmed = buchungstext.trim().to_string();
    if trimmed.is_empty() {
        return Err(ServerFnError::new("Bitte einen Buchungstext eingeben."));
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
fn parse_datum(text: &str, fehlertext: &str) -> Result<NaiveDate, ServerFnError> {
    let text = text.trim();
    NaiveDate::parse_from_str(text, "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(text, "%d.%m.%Y"))
        .map_err(|_| ServerFnError::new(fehlertext))
}

#[server(
    AddBuchung,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "add_buchung"
)]
pub async fn add_buchung(
    buchungstext: String,
    betrag: f64,
    art: BuchungsArt,
    datum: Option<String>,
) -> Result<Buchung, ServerFnError> {
    let trimmed = pruefe_text_und_betrag(&buchungstext, betrag)?;

    let buchungs_datum = match datum.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(d_str) => parse_datum(d_str, "Bitte ein gültiges Datum auswählen.")?,
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

#[server(
    UpdateBuchung,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "update_buchung"
)]
pub async fn update_buchung(
    id: Uuid,
    buchungstext: String,
    betrag: f64,
    art: BuchungsArt,
    datum: String,
) -> Result<(), ServerFnError> {
    let trimmed = pruefe_text_und_betrag(&buchungstext, betrag)?;
    let erstellt_am = parse_datum(&datum, "Bitte ein gültiges Datum auswählen.")?;

    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query(
                r#"
                UPDATE buchungen
                SET buchungstext = $1, betrag = $2, art = $3, erstellt_am = $4
                WHERE id = $5 AND benutzer_id = $6
                "#,
            )
            .bind(&trimmed)
            .bind(betrag.abs())
            .bind(art.as_str())
            .bind(erstellt_am)
            .bind(id)
            .bind(current_user.id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Update fehlgeschlagen: {e}")))?;
            return Ok(());
        }

        let mut guard = store::BUCHUNGEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        if let Some(b) = guard.iter_mut().find(|b| {
            b.id == id && (b.benutzer_id == current_user.id || b.benutzer == current_user.benutzername)
        }) {
            b.buchungstext = trimmed;
            b.betrag = betrag.abs();
            b.art = art;
            b.erstellt_am = erstellt_am;
        }
    }

    Ok(())
}

#[server(
    DeleteBuchung,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "delete_buchung"
)]
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

#[server(
    GetFixkosten,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "get_fixkosten"
)]
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
                    f.beginndatum,
                    f.kuendigungsfrist_monate,
                    f.zahlungsperiode,
                    f.status,
                    f.art,
                    f.kategorie_id,
                    f.benutzer_id,
                    u.benutzername AS benutzer
                FROM fixkosten f
                INNER JOIN benutzer u ON f.benutzer_id = u.id
                WHERE f.benutzer_id = $1
                ORDER BY f.beginndatum DESC, f.buchungstext
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
                    beginndatum: row.get("beginndatum"),
                    kuendigungsfrist_monate: row.get("kuendigungsfrist_monate"),
                    zahlungsperiode: Zahlungsperiode::from_db_str(
                        &row.get::<String, _>("zahlungsperiode"),
                    ),
                    status: FixkostenStatus::from_db_str(&row.get::<String, _>("status")),
                    art: FixkostenArt::from_db_str(&row.get::<String, _>("art")),
                    kategorie_id: row.get("kategorie_id"),
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

#[server(
    AddFixkosten,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "add_fixkosten"
)]
pub async fn add_fixkosten(daten: FixkostenDaten) -> Result<FixkostenEintrag, ServerFnError> {
    let FixkostenDaten {
        buchungstext,
        betrag,
        beginndatum,
        kuendigungsfrist_monate,
        zahlungsperiode,
        status,
        art,
        kategorie_id,
    } = daten;
    let trimmed = pruefe_text_und_betrag(&buchungstext, betrag)?;
    if !(0..=36).contains(&kuendigungsfrist_monate) {
        return Err(ServerFnError::new(
            "Die Kündigungsfrist muss zwischen 0 und 36 Monaten liegen.",
        ));
    }
    let beginndatum = parse_datum(&beginndatum, "Bitte ein gültiges Beginndatum auswählen.")?;

    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use crate::backend::kategorien::pruefe_kategorie_zugehoerig;
        use sqlx::PgPool;

        let current_user = require_current_user().await?;
        pruefe_kategorie_zugehoerig(kategorie_id, current_user.id).await?;

        let eintrag = FixkostenEintrag {
            id: Uuid::new_v4(),
            buchungstext: trimmed,
            betrag: betrag.abs(),
            beginndatum,
            kuendigungsfrist_monate,
            zahlungsperiode,
            status,
            art,
            kategorie_id,
            benutzer_id: current_user.id,
            benutzer: current_user.benutzername,
        };

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query(
                r#"
                INSERT INTO fixkosten
                    (id, buchungstext, betrag, beginndatum, kuendigungsfrist_monate, zahlungsperiode, status, art, kategorie_id, benutzer_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
            )
            .bind(eintrag.id)
            .bind(&eintrag.buchungstext)
            .bind(eintrag.betrag)
            .bind(eintrag.beginndatum)
            .bind(eintrag.kuendigungsfrist_monate)
            .bind(eintrag.zahlungsperiode.as_str())
            .bind(eintrag.status.as_str())
            .bind(eintrag.art.as_str())
            .bind(eintrag.kategorie_id)
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

#[server(
    UpdateFixkosten,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "update_fixkosten"
)]
pub async fn update_fixkosten(id: Uuid, daten: FixkostenDaten) -> Result<(), ServerFnError> {
    let FixkostenDaten {
        buchungstext,
        betrag,
        beginndatum,
        kuendigungsfrist_monate,
        zahlungsperiode,
        status,
        art,
        kategorie_id,
    } = daten;
    let trimmed = pruefe_text_und_betrag(&buchungstext, betrag)?;
    if !(0..=36).contains(&kuendigungsfrist_monate) {
        return Err(ServerFnError::new(
            "Die Kündigungsfrist muss zwischen 0 und 36 Monaten liegen.",
        ));
    }
    let beginn = parse_datum(&beginndatum, "Bitte ein gültiges Beginndatum auswählen.")?;

    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use crate::backend::kategorien::pruefe_kategorie_zugehoerig;
        use sqlx::PgPool;

        let current_user = require_current_user().await?;
        pruefe_kategorie_zugehoerig(kategorie_id, current_user.id).await?;

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query(
                r#"
                UPDATE fixkosten
                SET buchungstext = $1, betrag = $2, beginndatum = $3,
                    kuendigungsfrist_monate = $4, zahlungsperiode = $5, status = $6,
                    art = $7, kategorie_id = $8
                WHERE id = $9 AND benutzer_id = $10
                "#,
            )
            .bind(&trimmed)
            .bind(betrag.abs())
            .bind(beginn)
            .bind(kuendigungsfrist_monate)
            .bind(zahlungsperiode.as_str())
            .bind(status.as_str())
            .bind(art.as_str())
            .bind(kategorie_id)
            .bind(id)
            .bind(current_user.id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Update fehlgeschlagen: {e}")))?;
            return Ok(());
        }

        let mut guard = store::FIXKOSTEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        if let Some(f) = guard.iter_mut().find(|f| {
            f.id == id && (f.benutzer_id == current_user.id || f.benutzer == current_user.benutzername)
        }) {
            f.buchungstext = trimmed;
            f.betrag = betrag.abs();
            f.beginndatum = beginn;
            f.kuendigungsfrist_monate = kuendigungsfrist_monate;
            f.zahlungsperiode = zahlungsperiode;
            f.status = status;
            f.art = art;
            f.kategorie_id = kategorie_id;
        }
    }

    Ok(())
}

#[server(
    UpdateFixkostenStatus,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "update_fixkosten_status"
)]
pub async fn update_fixkosten_status(
    id: Uuid,
    status: FixkostenStatus,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query("UPDATE fixkosten SET status = $1 WHERE id = $2 AND benutzer_id = $3")
                .bind(status.as_str())
                .bind(id)
                .bind(current_user.id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Update fehlgeschlagen: {e}")))?;
            return Ok(());
        }

        let mut guard = store::FIXKOSTEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        if let Some(f) = guard.iter_mut().find(|f| {
            f.id == id && (f.benutzer_id == current_user.id || f.benutzer == current_user.benutzername)
        }) {
            f.status = status;
        }
    }

    Ok(())
}

#[server(
    DeleteFixkosten,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "delete_fixkosten"
)]
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

/// Ein Fixkosten-Vertrag mit seinem Betrag in einem bestimmten Monat.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FixkostenPosten {
    pub id: Uuid,
    pub buchungstext: String,
    pub zahlungsperiode: Zahlungsperiode,
    /// Betrag in CHF, der in diesem Monat anfällt (volle Zahlung, nicht umgerechnet)
    pub betrag: f64,
    /// Kategorie des Postens; None = ohne Kategorie
    pub kategorie_id: Option<Uuid>,
}

/// Eine Zeile der Monatsübersicht (Kosten pro Monat).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MonatsKosten {
    pub jahr: i32,
    pub monat: u32,
    pub monat_name: String,
    pub einnahmen: f64,
    pub ausgaben: f64,
    pub fixkosten: f64,
    /// Einzelne Fixkosten-Verträge, die in diesem Monat Kosten verursachen
    pub fixkosten_posten: Vec<FixkostenPosten>,
    /// Summe der Fixeinnahmen in diesem Monat
    pub fixeinnahmen: f64,
    /// Einzelne Fixeinnahmen, die in diesem Monat anfallen
    pub fixeinnahmen_posten: Vec<FixkostenPosten>,
    pub total_kosten: f64,
    /// Einnahmen (Buchungen) + Fixeinnahmen - Total Kosten
    pub saldo: f64,
    pub anzahl_buchungen: usize,
    /// Ist dies der laufende Monat? (Für die Hervorhebung in der Übersicht)
    pub ist_aktueller_monat: bool,
}

/// Gruppiert Buchungen nach Monat und rechnet die monatlichen Fixkosten und Fixeinnahmen dazu.
/// Fixkosten und Fixeinnahmen zählen ab ihrem Beginnmonat, solange sie nicht beendet sind; nicht monatliche
/// Zahlungen werden auf den Monat umgerechnet (z.B. Jahresbetrag / 12).
/// Die Liste umfasst alle zwölf Monate des Jahres von `bis` (Januar bis Dezember, älteste zuerst).
/// Monate in der Zukunft enthalten nur die Fixkosten und Fixeinnahmen als Prognose.
pub fn berechne_monatskosten(
    buchungen: &[Buchung],
    fixkosten: &[FixkostenEintrag],
    bis: NaiveDate,
) -> Vec<MonatsKosten> {
    let jahr = bis.year();

    (1..=12u32)
        .map(|monat| {
            let im_monat = buchungen
                .iter()
                .filter(|b| b.erstellt_am.year() == jahr && b.erstellt_am.month() == monat);

            let mut einnahmen = 0.0;
            let mut ausgaben = 0.0;
            let mut anzahl_buchungen = 0;
            for b in im_monat {
                anzahl_buchungen += 1;
                match b.art {
                    BuchungsArt::Eingang => einnahmen += b.betrag,
                    BuchungsArt::Ausgang => ausgaben += b.betrag,
                }
            }

            let posten = |f: &FixkostenEintrag| FixkostenPosten {
                id: f.id,
                buchungstext: f.buchungstext.clone(),
                zahlungsperiode: f.zahlungsperiode,
                betrag: f.betrag,
                kategorie_id: f.kategorie_id,
            };
            // Nur Posten, die in diesem Monat fällig sind. Die übrigen Monate zeigen einen Strich.
            let aktive: Vec<&FixkostenEintrag> = fixkosten
                .iter()
                .filter(|f| f.faellig_im_monat(jahr, monat))
                .collect();
            let fixkosten_posten: Vec<FixkostenPosten> = aktive
                .iter()
                .filter(|f| !f.ist_einnahme())
                .map(|f| posten(f))
                .collect();
            let fixeinnahmen_posten: Vec<FixkostenPosten> = aktive
                .iter()
                .filter(|f| f.ist_einnahme())
                .map(|f| posten(f))
                .collect();
            let summe_fixkosten: f64 = fixkosten_posten.iter().map(|p| p.betrag).sum();
            let summe_fixeinnahmen: f64 = fixeinnahmen_posten.iter().map(|p| p.betrag).sum();
            let total_kosten = ausgaben + summe_fixkosten;

            MonatsKosten {
                jahr,
                monat,
                monat_name: format!("{} {}", german_month_name(monat), jahr),
                einnahmen,
                ausgaben,
                fixkosten: summe_fixkosten,
                fixkosten_posten,
                fixeinnahmen: summe_fixeinnahmen,
                fixeinnahmen_posten,
                total_kosten,
                saldo: einnahmen + summe_fixeinnahmen - total_kosten,
                anzahl_buchungen,
                ist_aktueller_monat: monat == bis.month(),
            }
        })
        .collect()
}

#[server(
    GetMonatskosten,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "get_monatskosten"
)]
pub async fn get_monatskosten() -> Result<Vec<MonatsKosten>, ServerFnError> {
    // `get_buchungen` und `get_fixkosten` liefern ausschließlich die Einträge des angemeldeten Benutzers
    let buchungen = get_buchungen().await?;
    let fixkosten = get_fixkosten().await?;

    Ok(berechne_monatskosten(
        &buchungen,
        &fixkosten,
        Local::now().date_naive(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buchung(datum: (i32, u32, u32), betrag: f64, art: BuchungsArt) -> Buchung {
        Buchung {
            id: Uuid::new_v4(),
            buchungstext: "Test".to_string(),
            betrag,
            art,
            erstellt_am: NaiveDate::from_ymd_opt(datum.0, datum.1, datum.2).unwrap(),
            benutzer_id: Uuid::nil(),
            benutzer: "test".to_string(),
        }
    }

    fn fixkosten(
        betrag: f64,
        beginn: (i32, u32, u32),
        periode: Zahlungsperiode,
        status: FixkostenStatus,
    ) -> FixkostenEintrag {
        FixkostenEintrag {
            id: Uuid::new_v4(),
            buchungstext: "Test".to_string(),
            betrag,
            beginndatum: NaiveDate::from_ymd_opt(beginn.0, beginn.1, beginn.2).unwrap(),
            kuendigungsfrist_monate: 3,
            zahlungsperiode: periode,
            status,
            art: FixkostenArt::Ausgabe,
            kategorie_id: None,
            benutzer_id: Uuid::nil(),
            benutzer: "test".to_string(),
        }
    }

    fn fixeinnahme(
        betrag: f64,
        beginn: (i32, u32, u32),
        periode: Zahlungsperiode,
        status: FixkostenStatus,
    ) -> FixkostenEintrag {
        FixkostenEintrag {
            art: FixkostenArt::Einnahme,
            ..fixkosten(betrag, beginn, periode, status)
        }
    }

    #[test]
    fn fixeinnahmen_erhoehen_saldo_aber_nicht_fixkosten() {
        let buchungen = vec![buchung((2026, 1, 1), 3000.0, BuchungsArt::Eingang)];
        let liste = vec![
            fixkosten(1000.0, (2024, 1, 1), Zahlungsperiode::Monatlich, FixkostenStatus::Aktiv),
            fixeinnahme(500.0, (2024, 1, 1), Zahlungsperiode::Monatlich, FixkostenStatus::Aktiv),
            fixeinnahme(1200.0, (2024, 1, 1), Zahlungsperiode::Jaehrlich, FixkostenStatus::Beendet),
        ];
        let bis = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let zeilen = berechne_monatskosten(&buchungen, &liste, bis);

        // Januar: Fixkosten nur der Ausgabe-Eintrag, Fixeinnahmen nur der aktive Eintrag
        assert_eq!(zeilen[0].fixkosten, 1000.0);
        assert_eq!(zeilen[0].fixkosten_posten.len(), 1);
        assert_eq!(zeilen[0].fixeinnahmen, 500.0);
        assert_eq!(zeilen[0].fixeinnahmen_posten.len(), 1);
        // Total Kosten enthalten keine Fixeinnahmen
        assert_eq!(zeilen[0].total_kosten, 1000.0);
        // Saldo = Einnahmen (3000) + Fixeinnahmen (500) - Total Kosten (1000)
        assert_eq!(zeilen[0].saldo, 2500.0);
    }

    #[test]
    fn fixkosten_standardwerte_sind_ausgaben_ohne_kategorie() {
        let eintrag = fixkosten(100.0, (2025, 1, 1), Zahlungsperiode::Monatlich, FixkostenStatus::Aktiv);
        assert!(!eintrag.ist_einnahme());
        assert_eq!(eintrag.kategorie_id, None);
        assert!(fixeinnahme(100.0, (2025, 1, 1), Zahlungsperiode::Monatlich, FixkostenStatus::Aktiv).ist_einnahme());
    }

    #[test]
    fn fixkosten_beginn_periode_und_status() {
        let liste = vec![
            fixkosten(1000.0, (2025, 1, 15), Zahlungsperiode::Monatlich, FixkostenStatus::Aktiv),
            fixkosten(1200.0, (2025, 12, 1), Zahlungsperiode::Jaehrlich, FixkostenStatus::Gekuendigt),
            fixkosten(500.0, (2020, 1, 1), Zahlungsperiode::Monatlich, FixkostenStatus::Beendet),
        ];
        let bis = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let zeilen = berechne_monatskosten(&[], &liste, bis);

        // Januar 2026: nur der monatliche Vertrag. Der jährliche zahlt erst im Dezember, der beendete zählt nicht
        assert_eq!(zeilen[0].fixkosten, 1000.0);
        assert_eq!(zeilen[0].fixkosten_posten.len(), 1);
        // Dezember 2026: monatlich 1000 plus die volle Jahreszahlung 1200
        assert_eq!(zeilen[11].fixkosten, 2200.0);
        // Die Liste zeigt nur das Jahr 2026, also kein November 2025
        let nov = zeilen.iter().find(|z| (z.jahr, z.monat) == (2025, 11));
        assert!(nov.is_none(), "die Liste umfasst nur das Jahr von `bis`");
        assert_eq!(zeilen.len(), 12);

        assert_eq!(liste[1].monatsbetrag(), 100.0);
        assert!(!liste[1].zaehlt_im_monat(2025, 11));
        assert!(liste[1].zaehlt_im_monat(2025, 12));
        assert!(!liste[2].zaehlt_im_monat(2026, 1));
    }

    #[test]
    fn zahlungen_fallen_nur_in_ihren_monaten_an() {
        let bis = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let monatlich = fixkosten(100.0, (2025, 1, 1), Zahlungsperiode::Monatlich, FixkostenStatus::Aktiv);
        // Vierteljährlich ab Juni 2025: 2026 im März, Juni, September und Dezember
        let vierteljaehrlich = fixkosten(25.5, (2025, 6, 1), Zahlungsperiode::Vierteljaehrlich, FixkostenStatus::Aktiv);
        // Halbjährlich ab März 2025: 2026 im März und September
        let halbjaehrlich = fixkosten(300.0, (2025, 3, 1), Zahlungsperiode::Halbjaehrlich, FixkostenStatus::Aktiv);
        // Jährlich ab März 2025: 2026 nur im März
        let jaehrlich = fixkosten(1200.0, (2025, 3, 1), Zahlungsperiode::Jaehrlich, FixkostenStatus::Aktiv);
        let liste = vec![monatlich.clone(), vierteljaehrlich.clone(), halbjaehrlich.clone(), jaehrlich.clone()];
        let zeilen = berechne_monatskosten(&[], &liste, bis);

        // Monatsnummern (1 = Januar), in denen ein Vertrag Kosten verursacht
        let monate_von = |id: Uuid| -> Vec<u32> {
            zeilen
                .iter()
                .filter(|z| z.fixkosten_posten.iter().any(|p| p.id == id))
                .map(|z| z.monat)
                .collect()
        };
        assert_eq!(monate_von(monatlich.id).len(), 12);
        assert_eq!(monate_von(vierteljaehrlich.id), vec![3, 6, 9, 12]);
        assert_eq!(monate_von(halbjaehrlich.id), vec![3, 9]);
        assert_eq!(monate_von(jaehrlich.id), vec![3]);

        // Der volle Betrag fällt in seinem Monat an, nicht anteilig
        let jaehrlicher_posten = zeilen[2].fixkosten_posten.iter().find(|p| p.id == jaehrlich.id).unwrap();
        assert_eq!(jaehrlicher_posten.betrag, 1200.0);
        // März: monatlich 100 + vierteljährlich 25.5 + halbjährlich 300 + jährlich 1200
        assert_eq!(zeilen[2].fixkosten, 1625.5);
        // Januar: nur der monatliche Vertrag
        assert_eq!(zeilen[0].fixkosten, 100.0);
    }

    #[test]
    fn monatskosten_ganzes_jahr() {
        let buchungen = vec![
            // Buchungen aus dem Vorjahr zählen nicht in die Jahresübersicht 2026
            buchung((2025, 11, 3), 100.0, BuchungsArt::Ausgang),
            buchung((2025, 11, 20), 50.0, BuchungsArt::Ausgang),
            buchung((2026, 1, 1), 3000.0, BuchungsArt::Eingang),
        ];
        let bis = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let fix = vec![fixkosten(1000.0, (2024, 1, 1), Zahlungsperiode::Monatlich, FixkostenStatus::Aktiv)];
        let zeilen = berechne_monatskosten(&buchungen, &fix, bis);

        // Alle zwölf Monate, älteste zuerst
        assert_eq!(zeilen.len(), 12);
        assert_eq!((zeilen[0].jahr, zeilen[0].monat), (2026, 1));
        assert_eq!((zeilen[11].jahr, zeilen[11].monat), (2026, 12));

        // Januar: Eingang 3000, Fixkosten 1000
        assert_eq!(zeilen[0].monat_name, "Januar 2026");
        assert_eq!(zeilen[0].anzahl_buchungen, 1);
        assert_eq!(zeilen[0].total_kosten, 1000.0);
        assert_eq!(zeilen[0].saldo, 2000.0);

        // Dezember (Zukunft): nur die Fixkosten als Prognose
        assert_eq!(zeilen[11].einnahmen, 0.0);
        assert_eq!(zeilen[11].fixkosten, 1000.0);
        assert_eq!(zeilen[11].total_kosten, 1000.0);

        // Die Ausgaben aus 2025 sind nicht enthalten
        assert_eq!(zeilen.iter().map(|z| z.ausgaben).sum::<f64>(), 0.0);
    }

    #[test]
    fn posten_tragen_ihre_kategorie() {
        let kategorie = Uuid::from_u128(7);
        let mut miete = fixkosten(980.0, (2024, 1, 1), Zahlungsperiode::Monatlich, FixkostenStatus::Aktiv);
        miete.kategorie_id = Some(kategorie);
        let ohne = fixkosten(10.0, (2024, 1, 1), Zahlungsperiode::Monatlich, FixkostenStatus::Aktiv);
        let bis = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let zeilen = berechne_monatskosten(&[], &[miete.clone(), ohne.clone()], bis);

        let posten = &zeilen[0].fixkosten_posten;
        let miete_posten = posten.iter().find(|p| p.id == miete.id).unwrap();
        let ohne_posten = posten.iter().find(|p| p.id == ohne.id).unwrap();
        assert_eq!(miete_posten.kategorie_id, Some(kategorie));
        assert_eq!(ohne_posten.kategorie_id, None);
    }

    #[test]
    fn kategorie_summen_nur_fixkosten_des_monats() {
        let wohnen = Uuid::from_u128(1);
        let versicherung = Uuid::from_u128(2);
        let kategorien = vec![
            Kategorie { id: wohnen, name: "Wohnen".to_string(), benutzer_id: Uuid::nil() },
            Kategorie { id: versicherung, name: "Versicherung".to_string(), benutzer_id: Uuid::nil() },
        ];
        let mut miete = fixkosten(980.0, (2024, 1, 1), Zahlungsperiode::Monatlich, FixkostenStatus::Aktiv);
        miete.kategorie_id = Some(wohnen);
        // 25,50 pro Quartal = 8,50 pro Monat
        let mut haftpflicht = fixkosten(25.5, (2025, 6, 1), Zahlungsperiode::Vierteljaehrlich, FixkostenStatus::Aktiv);
        haftpflicht.kategorie_id = Some(versicherung);
        // Fixeinnahmen zählen nicht
        let mut lohn = fixeinnahme(3000.0, (2024, 1, 1), Zahlungsperiode::Monatlich, FixkostenStatus::Aktiv);
        lohn.kategorie_id = Some(wohnen);
        let ohne = fixkosten(10.0, (2024, 1, 1), Zahlungsperiode::Monatlich, FixkostenStatus::Aktiv);
        // Beendete Verträge zählen nicht
        let mut alt = fixkosten(500.0, (2020, 1, 1), Zahlungsperiode::Monatlich, FixkostenStatus::Beendet);
        alt.kategorie_id = Some(versicherung);

        let summen = berechne_kategorie_summen(&[miete, haftpflicht, lohn, ohne, alt], &kategorien, 2026, 1);

        assert_eq!(summen.len(), 3);
        assert_eq!((summen[0].name.as_str(), summen[0].betrag), ("Wohnen", 980.0));
        assert_eq!((summen[1].name.as_str(), summen[1].betrag), ("Ohne Kategorie", 10.0));
        assert_eq!((summen[2].name.as_str(), summen[2].betrag), ("Versicherung", 8.5));
    }

    #[test]
    fn monatskosten_ohne_buchungen() {
        let bis = NaiveDate::from_ymd_opt(2026, 11, 6).unwrap();
        let zeilen = berechne_monatskosten(&[], &[], bis);
        assert_eq!(zeilen.len(), 12);
        assert_eq!(zeilen[0].monat_name, "Januar 2026");
        assert_eq!(zeilen[11].monat_name, "Dezember 2026");
        assert!(zeilen.iter().all(|z| z.total_kosten == 0.0));
        // Nur November (Monat von `bis`) ist der aktuelle Monat
        assert_eq!(zeilen.iter().filter(|z| z.ist_aktueller_monat).count(), 1);
        assert!(zeilen[10].ist_aktueller_monat);
    }

    #[test]
    fn filter_nach_status() {
        let aktiv = fixkosten(100.0, (2025, 1, 1), Zahlungsperiode::Monatlich, FixkostenStatus::Aktiv);
        let gekuendigt = fixkosten(200.0, (2025, 1, 1), Zahlungsperiode::Monatlich, FixkostenStatus::Gekuendigt);
        let beendet = fixkosten(300.0, (2025, 1, 1), Zahlungsperiode::Monatlich, FixkostenStatus::Beendet);
        let liste = vec![aktiv, gekuendigt.clone(), beendet];

        let filter = FixkostenFilter {
            status: Some(FixkostenStatus::Gekuendigt),
            ..Default::default()
        };
        assert!(filter.ist_aktiv());
        assert_eq!(filtere_fixkosten(&liste, &filter), vec![gekuendigt]);
        // Ohne Filter bleiben alle Einträge erhalten
        assert!(!FixkostenFilter::default().ist_aktiv());
        assert_eq!(filtere_fixkosten(&liste, &FixkostenFilter::default()).len(), 3);
    }

    #[test]
    fn filter_nach_kategorie() {
        let wohnen = Uuid::new_v4();
        let mut miete = fixkosten(980.0, (2024, 1, 1), Zahlungsperiode::Monatlich, FixkostenStatus::Aktiv);
        miete.kategorie_id = Some(wohnen);
        let ohne = fixkosten(10.0, (2024, 1, 1), Zahlungsperiode::Monatlich, FixkostenStatus::Aktiv);
        let liste = vec![miete.clone(), ohne.clone()];

        let nur_wohnen = FixkostenFilter {
            kategorie: KategorieFilter::Kategorie(wohnen),
            ..Default::default()
        };
        assert_eq!(filtere_fixkosten(&liste, &nur_wohnen), vec![miete]);

        let nur_ohne = FixkostenFilter {
            kategorie: KategorieFilter::OhneKategorie,
            ..Default::default()
        };
        assert_eq!(filtere_fixkosten(&liste, &nur_ohne), vec![ohne]);
    }

    #[test]
    fn filter_nach_beginn_datum_mit_grenzen_inklusive() {
        let vor_zeitraum = fixkosten(1.0, (2024, 12, 31), Zahlungsperiode::Monatlich, FixkostenStatus::Aktiv);
        let von = fixkosten(2.0, (2025, 1, 1), Zahlungsperiode::Monatlich, FixkostenStatus::Aktiv);
        let bis = fixkosten(3.0, (2025, 6, 1), Zahlungsperiode::Monatlich, FixkostenStatus::Aktiv);
        let nach_zeitraum = fixkosten(4.0, (2025, 6, 2), Zahlungsperiode::Monatlich, FixkostenStatus::Aktiv);
        let liste = vec![vor_zeitraum, von.clone(), bis.clone(), nach_zeitraum];

        let filter = FixkostenFilter {
            beginn_von: Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
            beginn_bis: Some(NaiveDate::from_ymd_opt(2025, 6, 1).unwrap()),
            ..Default::default()
        };
        assert_eq!(filtere_fixkosten(&liste, &filter), vec![von, bis]);
    }
}
