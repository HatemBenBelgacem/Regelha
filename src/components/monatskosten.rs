use leptos::prelude::*;
use uuid::Uuid;

use crate::backend::budget::{get_monatskosten, FixkostenPosten, MonatsKosten};
use crate::backend::kategorien::{get_kategorien, kategorie_name, Kategorie};

/// Formatiert einen Saldo mit Vorzeichen (z.B. "+ CHF 12.00" / "- CHF 3.50").
fn saldo_text(saldo: f64) -> String {
    let saldo = saldo + 0.0; // -0.0 vermeiden, siehe betrag_text
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
    // `+ 0.0` macht aus -0.0 (Summe einer leeren Liste) eine 0, damit nicht "-0.00" erscheint
    format!("{:.2}", wert + 0.0)
}

/// Farben der Beträge: Fixkosten schwarz, Fixeinnahmen grün.
#[derive(Clone, Copy)]
struct Farben {
    zelle: &'static str,
    summe: &'static str,
}

const FARBEN_AUSGABEN: Farben = Farben {
    zelle: "text-right text-black whitespace-nowrap",
    summe: "text-right font-semibold text-black whitespace-nowrap",
};

const FARBEN_EINNAHMEN: Farben = Farben {
    zelle: "text-right text-success whitespace-nowrap",
    summe: "text-right font-semibold text-success whitespace-nowrap",
};

/// Eine fertig formatierte Zelle der Tabelle.
struct Zelle {
    text: String,
    klasse: &'static str,
}

/// Eine Zeile pro Fixkosten- oder Fixeinnahme-Vertrag mit dem Betrag je Monat.
struct PostenZeile {
    titel: String,
    untertitel: String,
    zellen: Vec<Zelle>,
    summe: Zelle,
}

/// Eine Kategorie mit ihren Posten. Die Zellen der Kategorie zeigen die Summe ihrer Posten je Monat.
struct Gruppe {
    titel: String,
    /// Ohne Kategorie steht die Gruppe ganz unten
    ohne_kategorie: bool,
    zeilen: Vec<PostenZeile>,
    zellen: Vec<Zelle>,
    summe: Zelle,
}

/// Eine Summenzeile unter der Tabelle (Einnahmen, Ausgaben, Total, Saldo).
struct SummenZeile {
    titel: &'static str,
    untertitel: String,
    zellen: Vec<Zelle>,
    summe: Zelle,
}

/// Ein Betrag im Monat. Ohne Betrag steht ein Strich.
fn posten_zelle(betrag: Option<f64>, farben: Farben) -> Zelle {
    match betrag {
        Some(b) => Zelle {
            text: betrag_text(b),
            klasse: farben.zelle,
        },
        None => Zelle {
            text: "–".to_string(),
            klasse: "text-right text-base-content/40 whitespace-nowrap",
        },
    }
}

/// Alle Posten (Fixkosten oder Fixeinnahmen, je nach `auswahl`), die in mindestens einem angezeigten Monat anfallen.
/// Sortiert nach Beginn: der älteste Vertrag steht oben.
fn posten_eindeutig(
    monate: &[MonatsKosten],
    auswahl: impl Fn(&MonatsKosten) -> &[FixkostenPosten],
) -> Vec<FixkostenPosten> {
    let mut liste: Vec<FixkostenPosten> = Vec::new();
    // Die Monate kommen Januar bis Dezember. So steht ein Vertrag an der Stelle seines Beginns.
    for monat in monate.iter() {
        for posten in auswahl(monat) {
            if !liste.iter().any(|p| p.id == posten.id) {
                liste.push(posten.clone());
            }
        }
    }
    liste
}

/// Betrag eines Postens in einem Monat. `None`, wenn in diesem Monat nichts anfällt.
fn betrag_im_monat(posten: &[FixkostenPosten], id: Uuid) -> Option<f64> {
    posten.iter().find(|p| p.id == id).map(|p| p.monatsbetrag)
}

/// Die Zeile eines Postens mit seinem Betrag in jedem Monat.
fn posten_zeile(
    posten: &FixkostenPosten,
    monate: &[MonatsKosten],
    auswahl: &impl Fn(&MonatsKosten) -> &[FixkostenPosten],
    farben: Farben,
) -> PostenZeile {
    let werte: Vec<Option<f64>> = monate
        .iter()
        .map(|m| betrag_im_monat(auswahl(m), posten.id))
        .collect();
    let summe: f64 = werte.iter().flatten().sum();
    PostenZeile {
        titel: posten.buchungstext.clone(),
        untertitel: posten.zahlungsperiode.label().to_string(),
        zellen: werte.into_iter().map(|w| posten_zelle(w, farben)).collect(),
        summe: Zelle {
            text: betrag_text(summe),
            klasse: farben.summe,
        },
    }
}

/// Gruppiert die Posten nach Kategorie. Die Kategorien stehen nach Name sortiert, "Ohne Kategorie" am Ende.
fn gruppen(
    monate: &[MonatsKosten],
    kategorien: &[Kategorie],
    auswahl: impl Fn(&MonatsKosten) -> &[FixkostenPosten],
    farben: Farben,
) -> Vec<Gruppe> {
    let posten = posten_eindeutig(monate, &auswahl);

    // Alle vorkommenden Kategorien (None = ohne Kategorie)
    let mut ids: Vec<Option<Uuid>> = Vec::new();
    for p in &posten {
        if !ids.contains(&p.kategorie_id) {
            ids.push(p.kategorie_id);
        }
    }

    let mut gruppen: Vec<Gruppe> = ids
        .into_iter()
        .map(|id| {
            let mitglieder: Vec<&FixkostenPosten> =
                posten.iter().filter(|p| p.kategorie_id == id).collect();
            let zeilen: Vec<PostenZeile> = mitglieder
                .iter()
                .map(|p| posten_zeile(p, monate, &auswahl, farben))
                .collect();
            // Summe der Kategorie je Monat. Ohne Betrag in diesem Monat steht ein Strich.
            let werte: Vec<Option<f64>> = monate
                .iter()
                .map(|m| {
                    let betraege: Vec<f64> = mitglieder
                        .iter()
                        .filter_map(|p| betrag_im_monat(auswahl(m), p.id))
                        .collect();
                    (!betraege.is_empty()).then(|| betraege.iter().sum::<f64>())
                })
                .collect();
            let summe: f64 = werte.iter().flatten().sum();
            Gruppe {
                titel: kategorie_name(kategorien, id),
                ohne_kategorie: id.is_none(),
                zeilen,
                zellen: werte.into_iter().map(|w| posten_zelle(w, farben)).collect(),
                summe: Zelle {
                    text: betrag_text(summe),
                    klasse: farben.summe,
                },
            }
        })
        .collect();

    gruppen.sort_by_key(|g| (g.ohne_kategorie, g.titel.to_lowercase()));
    gruppen
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

/// Eine Monatszelle. Die Zelle des laufenden Monats bekommt eine Hintergrundfarbe.
fn zelle_view(zelle: Zelle, hervorgehoben: bool) -> impl IntoView {
    let Zelle { text, klasse } = zelle;
    let klasse = if hervorgehoben {
        format!("{klasse} bg-primary/10")
    } else {
        klasse.to_string()
    };
    view! { <td class=klasse>{text}</td> }
}

/// Alle Monatszellen einer Zeile. `aktuell` ist die Spalte des laufenden Monats (falls vorhanden).
fn zellen_views(zellen: Vec<Zelle>, aktuell: Option<usize>) -> impl IntoView {
    zellen
        .into_iter()
        .enumerate()
        .map(|(i, zelle)| zelle_view(zelle, Some(i) == aktuell))
        .collect_view()
}

/// Kopfzelle eines Monats. Der laufende Monat ist farblich und mit einem Hinweis markiert.
fn kopf_view((name, anzahl, aktuell): (String, String, bool)) -> impl IntoView {
    let klasse = if aktuell {
        "text-right normal-case bg-primary/10 text-primary"
    } else {
        "text-right normal-case"
    };
    view! {
        <th class=klasse aria-current=aktuell.then_some("date")>
            <div class="flex flex-col items-end">
                <span class="whitespace-nowrap">{name}</span>
                <span class="text-xs font-normal text-base-content/50 whitespace-nowrap">{anzahl}</span>
                {aktuell.then(|| view! {
                    <span class="badge badge-primary badge-xs mt-1 whitespace-nowrap">"Aktuell"</span>
                })}
            </div>
        </th>
    }
}

/// Eine Posten-Zeile. Sie ist unter ihrer Kategorie eingerückt.
fn posten_zeile_view(zeile: PostenZeile, aktuell: Option<usize>) -> impl IntoView {
    let PostenZeile { titel, untertitel, zellen, summe } = zeile;
    view! {
        <tr>
            <td class="sticky left-0 z-10 bg-base-100 whitespace-nowrap font-medium pl-6">
                <div class="flex flex-col">
                    <span>{titel}</span>
                    <span class="text-xs font-normal text-base-content/50">{untertitel}</span>
                </div>
            </td>
            {zellen_views(zellen, aktuell)}
            {zelle_view(summe, false)}
        </tr>
    }
}

/// Eine Kategorie: zuerst die Summenzeile der Kategorie, darunter ihre Posten.
fn gruppe_view(gruppe: Gruppe, aktuell: Option<usize>) -> impl IntoView {
    let Gruppe { titel, zeilen, zellen, summe, .. } = gruppe;
    view! {
        <tr class="font-semibold bg-base-200/60">
            <td class="sticky left-0 z-10 bg-base-200 whitespace-nowrap">{titel}</td>
            {zellen_views(zellen, aktuell)}
            {zelle_view(summe, false)}
        </tr>
        {zeilen.into_iter().map(|z| posten_zeile_view(z, aktuell)).collect_view()}
    }
}

/// Ein Abschnitt der Tabelle ("Fixkosten" oder "Fixeinnahmen") mit allen Kategorien.
/// Ohne Posten in diesem Abschnitt bleibt er leer.
fn abschnitt_view(
    titel: &'static str,
    gruppen: Vec<Gruppe>,
    spalten: String,
    aktuell: Option<usize>,
) -> AnyView {
    if gruppen.is_empty() {
        return ().into_any();
    }
    view! {
        <tr class="bg-base-300/40">
            <td colspan=spalten class="font-bold text-xs uppercase tracking-wide text-base-content/80">
                {titel}
            </td>
        </tr>
        {gruppen.into_iter().map(|g| gruppe_view(g, aktuell)).collect_view()}
    }
    .into_any()
}

fn summen_zeile_view(zeile: SummenZeile, aktuell: Option<usize>) -> impl IntoView {
    let SummenZeile { titel, untertitel, zellen, summe } = zeile;
    view! {
        <tr class="font-bold text-xs bg-base-200/50">
            <td class="sticky left-0 z-10 bg-base-200 whitespace-nowrap">
                <div class="flex flex-col">
                    <span>{titel}</span>
                    <span class="text-xs font-normal text-base-content/70">{untertitel}</span>
                </div>
            </td>
            {zellen_views(zellen, aktuell)}
            {zelle_view(summe, false)}
        </tr>
    }
}

/// Die Tabelle: Monate als Spalten. Fixkosten und Fixeinnahmen stehen je nach Kategorie gruppiert, unten die Summen.
/// Die Spalte des laufenden Monats ist hervorgehoben.
fn monatstabelle(monate: Vec<MonatsKosten>, kategorien: Vec<Kategorie>) -> impl IntoView {
    let ausgaben = gruppen(
        &monate,
        &kategorien,
        |m| m.fixkosten_posten.as_slice(),
        FARBEN_AUSGABEN,
    );
    let einnahmen = gruppen(
        &monate,
        &kategorien,
        |m| m.fixeinnahmen_posten.as_slice(),
        FARBEN_EINNAHMEN,
    );
    // Position der Spalte des laufenden Monats. Die Entscheidung trifft der Server, damit Server und Browser gleich rendern.
    let aktuell = monate.iter().position(|m| m.ist_aktueller_monat);

    // Der Durchschnitt zählt nur Monate bis zum aktuellen Monat. Zukunftsmonate enthalten nur die Prognose.
    let vergangene: Vec<&MonatsKosten> = monate
        .iter()
        .take(aktuell.map_or(0, |i| i + 1))
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
            |_: f64| "text-right text-black whitespace-nowrap",
            betrag_text,
        ),
        summen_zeile(
            "Fixeinnahmen gesamt",
            String::new(),
            monate.iter().map(|m| m.fixeinnahmen).collect(),
            |_: f64| "text-right text-success whitespace-nowrap",
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

    let kopf: Vec<(String, String, bool)> = monate
        .iter()
        .map(|m| {
            (
                m.monat_name.clone(),
                format!("{} Buchungen", m.anzahl_buchungen),
                m.ist_aktueller_monat,
            )
        })
        .collect();
    let leer = ausgaben.is_empty() && einnahmen.is_empty();
    let colspan = (monate.len() + 2).to_string();
    let colspan_leer = colspan.clone();

    view! {
        <table class="table table-zebra table-xs w-full">
            <thead>
                <tr class="text-xs uppercase text-base-content/70">
                    <th class="sticky left-0 z-10 bg-base-100">"Fixkosten & Fixeinnahmen"</th>
                    {kopf.into_iter().map(kopf_view).collect_view()}
                    <th class="text-right">"Summe"</th>
                </tr>
            </thead>
            <tbody>
                {leer.then(move || view! {
                    <tr>
                        <td colspan=colspan_leer class="text-center text-base-content/60">
                            "Keine Fixkosten oder Fixeinnahmen erfasst"
                        </td>
                    </tr>
                })}
                {abschnitt_view("Fixkosten", ausgaben, colspan.clone(), aktuell)}
                {abschnitt_view("Fixeinnahmen", einnahmen, colspan, aktuell)}
            </tbody>
            <tfoot>
                {summen.into_iter().map(|z| summen_zeile_view(z, aktuell)).collect_view()}
            </tfoot>
        </table>
    }
}

#[component]
pub fn MonatskostenSection() -> impl IntoView {
    let monate_resource = Resource::new(|| (), |_| get_monatskosten());
    let kategorien_resource = Resource::new(|| (), |_| get_kategorien());

    view! {
        <section class="card bg-base-100 shadow-sm border border-base-300">
            <div class="card-body p-4 sm:p-6 space-y-6">
                <div>
                    <h2 class="card-title text-lg sm:text-xl">"Kosten pro Monat"</h2>
                    <p class="text-xs sm:text-sm text-base-content/70">
                        "Übersicht über das ganze Jahr: Einnahmen, Ausgaben, Fixkosten und Fixeinnahmen nach Kategorien in CHF"
                    </p>
                </div>

                <div class="overflow-x-auto rounded-box border border-base-300">
                    <Suspense fallback=move || view! {
                        <div class="flex justify-center p-8">
                            <span class="loading loading-spinner loading-md text-primary"></span>
                        </div>
                    }>
                        {move || {
                            let kategorien = kategorien_resource.get();
                            monate_resource.get().map(|res| match (res, kategorien) {
                                (Ok(monate), Some(Ok(kategorien))) => monatstabelle(monate, kategorien).into_any(),
                                (Err(err), _) => view! {
                                    <div class="alert alert-error">
                                        <span>{format!("Fehler beim Laden der Monatsübersicht: {err}")}</span>
                                    </div>
                                }.into_any(),
                                (_, Some(Err(err))) => view! {
                                    <div class="alert alert-error">
                                        <span>{format!("Fehler beim Laden der Kategorien: {err}")}</span>
                                    </div>
                                }.into_any(),
                                _ => ().into_any(),
                            })
                        }}
                    </Suspense>
                </div>

                <p class="text-xs text-base-content/60">
                    "Total Kosten = Ausgaben + Fixkosten. Saldo = Einnahmen + Fixeinnahmen − Total Kosten. Fixkosten und Fixeinnahmen sind nach Kategorie gruppiert; die Zeile einer Kategorie zeigt die Summe ihrer Posten je Monat. Ohne Kategorie steht ganz unten. Der laufende Monat ist hervorgehoben. Fixkosten und Fixeinnahmen zählen ab ihrem Beginnmonat, solange sie nicht beendet sind. Vierteljährliche, halbjährliche und jährliche Zahlungen werden auf den Monat umgerechnet. Ein Strich (–) bedeutet: In diesem Monat fällt dafür nichts an. Ab dem aktuellen Monat zeigt die Tabelle nur die Fixkosten und Fixeinnahmen als Prognose."
                </p>
            </div>
        </section>
    }
}
