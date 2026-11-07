use chrono::{Datelike, Local};
use leptos::prelude::*;
use uuid::Uuid;

use crate::backend::budget::{get_monatskosten, FixkostenPosten, MonatsKosten};

/// Formatiert einen Saldo mit Vorzeichen (z.B. "+ CHF 12.00" / "- CHF 3.50").
fn saldo_text(saldo: f64) -> String {
    if saldo >= 0.0 {
        format!("+ CHF {:.2}", saldo)
    } else {
        format!("- CHF {:.2}", saldo.abs())
    }
}

fn saldo_class(saldo: f64) -> &'static str {
    if saldo >= 0.0 {
        "text-right font-semibold text-success whitespace-nowrap"
    } else {
        "text-right font-semibold text-error whitespace-nowrap"
    }
}

fn betrag_text(wert: f64) -> String {
    format!("{:.2}", wert)
}

/// Eine fertig formatierte Zelle der Tabelle.
struct Zelle {
    text: String,
    klasse: &'static str,
}

/// Eine Zeile pro Fixkosten-Vertrag mit dem Betrag je Monat.
struct FixkostenZeile {
    titel: String,
    untertitel: &'static str,
    zellen: Vec<Zelle>,
    summe: Zelle,
}

/// Eine Summenzeile unter den Fixkosten (Einnahmen, Ausgaben, Total, Saldo).
struct SummenZeile {
    titel: &'static str,
    untertitel: String,
    zellen: Vec<Zelle>,
    summe: Zelle,
}

/// Ein Fixkosten-Betrag im Monat. Ohne Betrag steht ein Strich.
fn fixkosten_zelle(betrag: Option<f64>) -> Zelle {
    match betrag {
        Some(b) => Zelle {
            text: betrag_text(b),
            klasse: "text-right text-warning whitespace-nowrap",
        },
        None => Zelle {
            text: "–".to_string(),
            klasse: "text-right text-base-content/40 whitespace-nowrap",
        },
    }
}

/// Alle Fixkosten, die in mindestens einem angezeigten Monat Kosten verursachen.
/// Sortiert nach Beginn: der älteste Vertrag steht oben.
fn fixkosten_eindeutig(monate: &[MonatsKosten]) -> Vec<FixkostenPosten> {
    let mut liste: Vec<FixkostenPosten> = Vec::new();
    // Die Monate kommen Januar bis Dezember. So steht ein Vertrag an der Stelle seines Beginns.
    for monat in monate.iter() {
        for posten in &monat.fixkosten_posten {
            if !liste.iter().any(|p| p.id == posten.id) {
                liste.push(posten.clone());
            }
        }
    }
    liste
}

/// Betrag eines Fixkosten-Vertrags in einem Monat. `None`, wenn in diesem Monat nichts anfällt.
fn betrag_im_monat(monat: &MonatsKosten, id: Uuid) -> Option<f64> {
    monat
        .fixkosten_posten
        .iter()
        .find(|p| p.id == id)
        .map(|p| p.monatsbetrag)
}

/// Baut eine Summenzeile. Die Farbe kann von jedem Wert abhängen (z.B. beim Saldo).
fn summen_zeile(
    titel: &'static str,
    untertitel: String,
    werte: Vec<f64>,
    klasse: fn(f64) -> &'static str,
    text: fn(f64) -> String,
) -> SummenZeile {
    let summe: f64 = werte.iter().sum();
    SummenZeile {
        titel,
        untertitel,
        zellen: werte
            .iter()
            .map(|w| Zelle {
                text: text(*w),
                klasse: klasse(*w),
            })
            .collect(),
        summe: Zelle {
            text: text(summe),
            klasse: klasse(summe),
        },
    }
}

fn zelle_view(zelle: Zelle) -> impl IntoView {
    let Zelle { text, klasse } = zelle;
    view! { <td class=klasse>{text}</td> }
}

fn kopf_view((name, anzahl): (String, String)) -> impl IntoView {
    view! {
        <th class="text-right normal-case">
            <div class="flex flex-col items-end">
                <span class="whitespace-nowrap">{name}</span>
                <span class="text-xs font-normal text-base-content/50 whitespace-nowrap">{anzahl}</span>
            </div>
        </th>
    }
}

fn fixkosten_zeile_view(zeile: FixkostenZeile) -> impl IntoView {
    let FixkostenZeile { titel, untertitel, zellen, summe } = zeile;
    view! {
        <tr>
            <td class="sticky left-0 z-10 bg-base-100 whitespace-nowrap font-medium">
                <div class="flex flex-col">
                    <span>{titel}</span>
                    <span class="text-xs font-normal text-base-content/50">{untertitel}</span>
                </div>
            </td>
            {zellen.into_iter().map(zelle_view).collect_view()}
            {zelle_view(summe)}
        </tr>
    }
}

fn summen_zeile_view(zeile: SummenZeile) -> impl IntoView {
    let SummenZeile { titel, untertitel, zellen, summe } = zeile;
    view! {
        <tr class="font-bold text-sm sm:text-base bg-base-200/50">
            <td class="sticky left-0 z-10 bg-base-200 whitespace-nowrap">
                <div class="flex flex-col">
                    <span>{titel}</span>
                    <span class="text-xs font-normal text-base-content/70">{untertitel}</span>
                </div>
            </td>
            {zellen.into_iter().map(zelle_view).collect_view()}
            {zelle_view(summe)}
        </tr>
    }
}

/// Die Tabelle: Monate als Spalten, Fixkosten als Zeilen, unten die Summen.
fn monatstabelle(monate: Vec<MonatsKosten>) -> impl IntoView {
    let fix_zeilen: Vec<FixkostenZeile> = fixkosten_eindeutig(&monate)
        .into_iter()
        .map(|posten| {
            let werte: Vec<Option<f64>> = monate
                .iter()
                .map(|m| betrag_im_monat(m, posten.id))
                .collect();
            let summe: f64 = werte.iter().flatten().sum();
            FixkostenZeile {
                titel: posten.buchungstext,
                untertitel: posten.zahlungsperiode.label(),
                zellen: werte.into_iter().map(fixkosten_zelle).collect(),
                summe: Zelle {
                    text: betrag_text(summe),
                    klasse: "text-right font-semibold text-warning whitespace-nowrap",
                },
            }
        })
        .collect();

    // Der Durchschnitt zählt nur Monate bis heute. Zukunftsmonate enthalten nur die Prognose.
    let heute = Local::now().date_naive();
    let vergangene: Vec<&MonatsKosten> = monate
        .iter()
        .filter(|m| (m.jahr, m.monat) <= (heute.year(), heute.month()))
        .collect();
    let anzahl_monate = vergangene.len().max(1) as f64;
    let total_summe: f64 = vergangene.iter().map(|m| m.total_kosten).sum();
    let summen = vec![
        summen_zeile(
            "Einnahmen",
            String::new(),
            monate.iter().map(|m| m.einnahmen).collect(),
            |_: f64| "text-right text-success whitespace-nowrap",
            betrag_text,
        ),
        summen_zeile(
            "Ausgaben",
            String::new(),
            monate.iter().map(|m| m.ausgaben).collect(),
            |_: f64| "text-right text-error whitespace-nowrap",
            betrag_text,
        ),
        summen_zeile(
            "Fixkosten gesamt",
            String::new(),
            monate.iter().map(|m| m.fixkosten).collect(),
            |_: f64| "text-right text-warning whitespace-nowrap",
            betrag_text,
        ),
        summen_zeile(
            "Total Kosten",
            format!("Ø Kosten: CHF {:.2} / Monat", total_summe / anzahl_monate),
            monate.iter().map(|m| m.total_kosten).collect(),
            |_: f64| "text-right font-bold whitespace-nowrap",
            betrag_text,
        ),
        summen_zeile(
            "Saldo",
            String::new(),
            monate.iter().map(|m| m.saldo).collect(),
            saldo_class,
            saldo_text,
        ),
    ];

    let kopf: Vec<(String, String)> = monate
        .iter()
        .map(|m| (m.monat_name.clone(), format!("{} Buchungen", m.anzahl_buchungen)))
        .collect();
    let leer = fix_zeilen.is_empty();
    let colspan = (monate.len() + 2).to_string();

    view! {
        <table class="table table-zebra table-xs w-full">
            <thead>
                <tr class="text-xs sm:text-sm uppercase text-base-content/70">
                    <th class="sticky left-0 z-10 bg-base-100">"Fixkosten"</th>
                    {kopf.into_iter().map(kopf_view).collect_view()}
                    <th class="text-right">"Summe"</th>
                </tr>
            </thead>
            <tbody>
                {leer.then(move || view! {
                    <tr>
                        <td colspan=colspan class="text-center text-base-content/60">
                            "Keine Fixkosten erfasst"
                        </td>
                    </tr>
                })}
                {fix_zeilen.into_iter().map(fixkosten_zeile_view).collect_view()}
            </tbody>
            <tfoot>
                {summen.into_iter().map(summen_zeile_view).collect_view()}
            </tfoot>
        </table>
    }
}

#[component]
pub fn MonatskostenSection() -> impl IntoView {
    let monate_resource = Resource::new(|| (), |_| get_monatskosten());

    view! {
        <section class="card bg-base-100 shadow-sm border border-base-300">
            <div class="card-body p-4 sm:p-6 space-y-6">
                <div>
                    <h2 class="card-title text-lg sm:text-xl">"Kosten pro Monat"</h2>
                    <p class="text-xs sm:text-sm text-base-content/70">
                        "Übersicht über das ganze Jahr: Einnahmen, Ausgaben und Fixkosten in CHF"
                    </p>
                </div>

                <div class="overflow-x-auto rounded-box border border-base-300">
                    <Suspense fallback=move || view! {
                        <div class="flex justify-center p-8">
                            <span class="loading loading-spinner loading-md text-primary"></span>
                        </div>
                    }>
                        {move || {
                            monate_resource.get().map(|res| match res {
                                Ok(monate) => monatstabelle(monate).into_any(),
                                Err(err) => view! {
                                    <div class="alert alert-error">
                                        <span>{format!("Fehler beim Laden der Monatsübersicht: {err}")}</span>
                                    </div>
                                }.into_any(),
                            })
                        }}
                    </Suspense>
                </div>

                <p class="text-xs text-base-content/60">
                    "Total Kosten = Ausgaben + Fixkosten. Fixkosten zählen ab ihrem Beginnmonat, solange sie nicht beendet sind. Vierteljährliche, halbjährliche und jährliche Zahlungen werden auf den Monat umgerechnet. Ein Strich (–) bedeutet: In diesem Monat fällt für diese Fixkosten nichts an. Ab dem aktuellen Monat zeigt die Tabelle nur die Fixkosten als Prognose."
                </p>
            </div>
        </section>
    }
}
