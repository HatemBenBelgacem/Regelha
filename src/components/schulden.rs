use std::collections::BTreeMap;

use chrono::{Datelike, Local};
use leptos::prelude::*;
use uuid::Uuid;

use crate::backend::kontakte::{anzeigename, get_kontakte};
use crate::backend::rueckzahlungen::{get_rueckzahlungen, Rueckzahlung};
use crate::backend::schulden::{add_schuld, delete_schuld, get_schulden, update_schuld, Schuld};
use crate::components::modal::Modal;
use crate::components::rueckzahlungen::RueckzahlungTab;

/// Heutiges Datum im Format JJJJ-MM-TT (für das Datumsfeld).
fn heute_iso() -> String {
    Local::now().date_naive().format("%Y-%m-%d").to_string()
}

/// Deutsche Monatsnamen für die Gruppierung (Index 0 = Januar).
const MONATE: [&str; 12] = [
    "Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober",
    "November", "Dezember",
];

/// Gruppiert die Schulden nach Monat, z.B. ("November 2026", [...]).
/// Die Liste kommt bereits nach Datum absteigend sortiert, deshalb reicht ein Durchgang.
fn nach_monat(items: Vec<Schuld>) -> Vec<(String, Vec<Schuld>)> {
    let mut gruppen: Vec<((i32, u32), Vec<Schuld>)> = Vec::new();
    for s in items {
        let schluessel = (s.datum.year(), s.datum.month());
        let gehoert_zur_letzten = gruppen.last().is_some_and(|(k, _)| *k == schluessel);
        if gehoert_zur_letzten {
            if let Some((_, liste)) = gruppen.last_mut() {
                liste.push(s);
            }
        } else {
            gruppen.push((schluessel, vec![s]));
        }
    }
    gruppen
        .into_iter()
        .map(|((jahr, monat), liste)| {
            (format!("{} {}", MONATE[(monat - 1) as usize], jahr), liste)
        })
        .collect()
}

/// Saldo eines Kontakts: Schulden und Rückzahlungen werden gegeneinander verrechnet.
#[derive(Clone, Debug, PartialEq)]
pub struct KontaktSaldo {
    pub name: String,
    pub schulden: f64,
    pub rueckzahlungen: f64,
}

impl KontaktSaldo {
    /// Restbetrag = Schulden minus Rückzahlungen. Ein negativer Wert bedeutet Guthaben.
    pub fn restbetrag(&self) -> f64 {
        self.schulden - self.rueckzahlungen
    }
}

/// Verrechnet Schulden und Rückzahlungen je Kontakt. Der Kontakt ist das Bezugsobjekt.
pub fn saldo_je_kontakt(schulden: &[Schuld], rueckzahlungen: &[Rueckzahlung]) -> Vec<KontaktSaldo> {
    let mut map: BTreeMap<Option<Uuid>, KontaktSaldo> = BTreeMap::new();

    for s in schulden {
        let eintrag = map.entry(s.kontakt_id).or_insert_with(|| KontaktSaldo {
            name: s.kontakt.clone().unwrap_or_else(|| "Ohne Kontakt".to_string()),
            schulden: 0.0,
            rueckzahlungen: 0.0,
        });
        eintrag.schulden += s.betrag;
    }

    for r in rueckzahlungen {
        let eintrag = map.entry(Some(r.kontakt_id)).or_insert_with(|| KontaktSaldo {
            name: r.kontakt.clone(),
            schulden: 0.0,
            rueckzahlungen: 0.0,
        });
        eintrag.rueckzahlungen += r.betrag;
    }

    let mut liste: Vec<KontaktSaldo> = map.into_values().collect();
    liste.sort_by(|a, b| a.name.cmp(&b.name));
    liste
}

/// Card oben: Total Schulden, Rückzahlungen und Restbetrag, dazu der Restbetrag je Kontakt.
fn saldo_card(schulden: &[Schuld], rueckzahlungen: &[Rueckzahlung]) -> impl IntoView {
    let total_schulden: f64 = schulden.iter().map(|s| s.betrag).sum();
    let total_rueckzahlungen: f64 = rueckzahlungen.iter().map(|r| r.betrag).sum();
    let restbetrag = total_schulden - total_rueckzahlungen;
    let restbetrag_klasse = if restbetrag > 0.0 { "text-error" } else { "text-success" };
    let saldi = saldo_je_kontakt(schulden, rueckzahlungen);

    view! {
        <div class="card bg-base-200 border border-base-300">
            <div class="card-body p-4 gap-4">
                <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
                    <div>
                        <span class="text-xs uppercase tracking-wide text-base-content/70">"Total Schulden"</span>
                        <div class="text-xl font-bold font-mono">{format!("CHF {:.2}", total_schulden)}</div>
                    </div>
                    <div>
                        <span class="text-xs uppercase tracking-wide text-base-content/70">"Rückzahlungen"</span>
                        <div class="text-xl font-bold font-mono text-success">{format!("CHF {:.2}", total_rueckzahlungen)}</div>
                    </div>
                    <div>
                        <span class="text-xs uppercase tracking-wide text-base-content/70">"Restbetrag"</span>
                        <div class=format!("text-2xl font-bold font-mono {restbetrag_klasse}")>
                            {format!("CHF {:.2}", restbetrag)}
                        </div>
                    </div>
                </div>

                {(!saldi.is_empty()).then(|| view! {
                    <div class="border-t border-base-300 pt-3 space-y-1">
                        <span class="text-xs uppercase tracking-wide text-base-content/70">"Restbetrag je Kontakt"</span>
                        <ul class="space-y-1 text-sm">
                            {saldi.into_iter().map(|k| {
                                let rest = k.restbetrag();
                                let text = if rest < 0.0 {
                                    format!("Guthaben CHF {:.2}", rest.abs())
                                } else {
                                    format!("CHF {:.2}", rest)
                                };
                                let klasse = if rest > 0.0 {
                                    "font-mono font-semibold text-error"
                                } else {
                                    "font-mono font-semibold text-success"
                                };
                                view! {
                                    <li class="flex justify-between gap-3">
                                        <span>{k.name}</span>
                                        <span class=klasse>{text}</span>
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                    </div>
                })}
            </div>
        </div>
    }
}

/// Die aktiven Reiter unter der Card.
#[derive(Clone, Copy, PartialEq)]
enum Reiter {
    Schulden,
    Rueckzahlungen,
}

fn tab_klasse(aktiv: bool) -> &'static str {
    if aktiv {
        "tab tab-active font-semibold"
    } else {
        "tab"
    }
}

/// Seite Schulden: Card mit Verrechnung, darunter die Reiter Schulden und Rückzahlungen.
#[component]
pub fn SchuldenSection() -> impl IntoView {
    let (reiter, set_reiter) = signal(Reiter::Schulden);
    // Zähler: Wird er erhöht, laden alle Listen neu (nach Anlegen, Ändern oder Löschen)
    let stand = RwSignal::new(0_u32);

    // Daten für die Card oben (die Reiter laden ihre Listen selbst)
    let schulden_res = Resource::new(move || stand.get(), |_| get_schulden());
    let rueckzahlungen_res = Resource::new(move || stand.get(), |_| get_rueckzahlungen());

    view! {
        <section class="card bg-base-100 shadow-sm border border-base-300">
            <div class="card-body p-4 sm:p-6 space-y-6">
                <div>
                    <h2 class="card-title text-lg sm:text-xl">"Schulden"</h2>
                    <p class="text-xs sm:text-sm text-base-content/70">
                        "Schulden pro Kontakt, abzüglich der Rückzahlungen an diesen Kontakt"
                    </p>
                </div>

                {move || match (schulden_res.get(), rueckzahlungen_res.get()) {
                    (Some(Ok(s)), Some(Ok(r))) => saldo_card(&s, &r).into_any(),
                    (Some(Err(e)), _) | (_, Some(Err(e))) => view! {
                        <div class="alert alert-error">
                            <span>{format!("Fehler beim Laden: {e}")}</span>
                        </div>
                    }.into_any(),
                    _ => view! {
                        <div class="flex justify-center p-6">
                            <span class="loading loading-spinner loading-md text-primary"></span>
                        </div>
                    }.into_any(),
                }}

                <div role="tablist" class="tabs tabs-border">
                    <button
                        type="button"
                        role="tab"
                        class=move || tab_klasse(reiter.get() == Reiter::Schulden)
                        on:click=move |_| set_reiter.set(Reiter::Schulden)
                    >
                        "Schulden"
                    </button>
                    <button
                        type="button"
                        role="tab"
                        class=move || tab_klasse(reiter.get() == Reiter::Rueckzahlungen)
                        on:click=move |_| set_reiter.set(Reiter::Rueckzahlungen)
                    >
                        "Rückzahlungen"
                    </button>
                </div>

                {move || match reiter.get() {
                    Reiter::Schulden => view! { <SchuldenTab stand=stand/> }.into_any(),
                    Reiter::Rueckzahlungen => view! { <RueckzahlungTab stand=stand/> }.into_any(),
                }}
            </div>
        </section>
    }
}

/// Reiter Schulden: Eingabemaske und Tabelle, nach Monat gruppiert mit Monatssumme.
#[component]
pub fn SchuldenTab(stand: RwSignal<u32>) -> impl IntoView {
    let (datum_input, set_datum_input) = signal(heute_iso());
    // Id des gewählten Kontakts als Text (leer = noch kein Kontakt gewählt)
    let (kontakt_input, set_kontakt_input) = signal(String::new());
    let (grund_input, set_grund_input) = signal(String::new());
    let (betrag_input, set_betrag_input) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (modal_offen, set_modal_offen) = signal(false);
    // Id der Schuld, die gerade bearbeitet wird (None = neue Schuld)
    let (bearbeiten_id, set_bearbeiten_id) = signal(Option::<uuid::Uuid>::None);
    // Zweiter Klick auf "Löschen" bestätigt das Löschen
    let (loesch_bestaetigen, set_loesch_bestaetigen) = signal(false);

    let schulden_resource = Resource::new(move || stand.get(), |_| get_schulden());
    let kontakte_resource = Resource::new(|| (), |_| get_kontakte());

    let create_action = Action::new(
        move |(datum, grund, betrag, kontakt_id): &(String, String, f64, uuid::Uuid)| {
            let (datum, grund, betrag, kontakt_id) =
                (datum.clone(), grund.clone(), *betrag, *kontakt_id);
            async move { add_schuld(datum, grund, betrag, kontakt_id).await }
        },
    );

    let update_action = Action::new(
        move |(id, datum, grund, betrag, kontakt_id): &(
            uuid::Uuid,
            String,
            String,
            f64,
            uuid::Uuid,
        )| {
            let (id, datum, grund, betrag, kontakt_id) =
                (*id, datum.clone(), grund.clone(), *betrag, *kontakt_id);
            async move { update_schuld(id, datum, grund, betrag, kontakt_id).await }
        },
    );

    let delete_action = Action::new(move |id: &uuid::Uuid| {
        let id = *id;
        async move { delete_schuld(id).await }
    });

    // Formular leeren und schliessen (nach Erfolg)
    let formular_schliessen = move || {
        set_kontakt_input.set(String::new());
        set_grund_input.set(String::new());
        set_betrag_input.set(String::new());
        set_error_msg.set(None);
        set_bearbeiten_id.set(None);
        set_loesch_bestaetigen.set(false);
        set_modal_offen.set(false);
    };

    Effect::new(move |_| {
        if let Some(res) = create_action.value().get() {
            match res {
                Ok(_) => {
                    formular_schliessen();
                    stand.update(|v| *v += 1);
                }
                Err(e) => set_error_msg.set(Some(e.to_string())),
            }
        }
    });

    Effect::new(move |_| {
        if let Some(res) = update_action.value().get() {
            match res {
                Ok(()) => {
                    formular_schliessen();
                    stand.update(|v| *v += 1);
                }
                Err(e) => set_error_msg.set(Some(e.to_string())),
            }
        }
    });

    Effect::new(move |_| {
        if let Some(res) = delete_action.value().get() {
            match res {
                Ok(()) => {
                    formular_schliessen();
                    stand.update(|v| *v += 1);
                }
                Err(e) => {
                    set_loesch_bestaetigen.set(false);
                    set_error_msg.set(Some(e.to_string()));
                }
            }
        }
    });

    // Öffnet die Eingabemaske mit den Werten einer bestehenden Schuld (Doppelklick auf die Zeile)
    let bearbeiten_oeffnen = move |s: Schuld| {
        set_error_msg.set(None);
        set_loesch_bestaetigen.set(false);
        set_bearbeiten_id.set(Some(s.id));
        set_datum_input.set(s.datum.format("%Y-%m-%d").to_string());
        set_kontakt_input.set(s.kontakt_id.map(|id| id.to_string()).unwrap_or_default());
        set_grund_input.set(s.grund.clone());
        set_betrag_input.set(format!("{:.2}", s.betrag));
        set_modal_offen.set(true);
    };

    // Erster Klick fragt nach, zweiter Klick löscht
    let loeschen_klick = move |id: uuid::Uuid| {
        if loesch_bestaetigen.get() {
            delete_action.dispatch(id);
        } else {
            set_loesch_bestaetigen.set(true);
        }
    };

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let kontakt_id = match uuid::Uuid::parse_str(&kontakt_input.get()) {
            Ok(id) => id,
            Err(_) => {
                set_error_msg.set(Some("Bitte einen Kontakt auswählen.".to_string()));
                return;
            }
        };
        let grund = grund_input.get();
        if grund.trim().is_empty() {
            set_error_msg.set(Some("Bitte einen Grund eingeben.".to_string()));
            return;
        }
        let raw_amount = betrag_input.get().replace(',', ".");
        let betrag = match raw_amount.trim().parse::<f64>() {
            Ok(val) if val > 0.0 => val,
            _ => {
                set_error_msg.set(Some(
                    "Bitte einen gültigen positiven Betrag in CHF eingeben (z.B. 120.50)."
                        .to_string(),
                ));
                return;
            }
        };
        set_error_msg.set(None);
        match bearbeiten_id.get() {
            Some(id) => {
                update_action.dispatch((id, datum_input.get(), grund, betrag, kontakt_id));
            }
            None => {
                create_action.dispatch((datum_input.get(), grund, betrag, kontakt_id));
            }
        }
    };

    view! {
        <div class="space-y-4">
            <div class="flex justify-end">
                <button
                    type="button"
                    class="btn btn-primary"
                    on:click=move |_| {
                        set_bearbeiten_id.set(None);
                        set_error_msg.set(None);
                        set_loesch_bestaetigen.set(false);
                        set_kontakt_input.set(String::new());
                        set_grund_input.set(String::new());
                        set_betrag_input.set(String::new());
                        set_datum_input.set(heute_iso());
                        set_modal_offen.set(true);
                    }
                >
                    "+ Neue Schuld"
                </button>
            </div>

            // Eingabemaske zum Erfassen, Bearbeiten und Löschen von Schulden
            <Modal
                offen=modal_offen
                on_close=Callback::new(move |_| set_modal_offen.set(false))
                titel=move || if bearbeiten_id.get().is_some() { "Schuld bearbeiten".to_string() } else { "Neue Schuld".to_string() }
            >
                <form on:submit=on_submit class="grid grid-cols-1 gap-3">
                    <div>
                        <label class="label py-1">
                            <span class="label-text font-medium text-sm">"Datum"</span>
                        </label>
                        <input
                            type="date"
                            class="input input-bordered w-full"
                            prop:value=move || datum_input.get()
                            on:input=move |ev| set_datum_input.set(event_target_value(&ev))
                            required
                        />
                    </div>

                    <div>
                        <label class="label py-1">
                            <span class="label-text font-medium text-sm">"Kontakt"</span>
                        </label>
                        <select
                            class="select select-bordered w-full"
                            on:change=move |ev| set_kontakt_input.set(event_target_value(&ev))
                            required
                        >
                            <option value="" prop:selected=move || kontakt_input.get().is_empty()>
                                "Bitte Kontakt wählen"
                            </option>
                            {move || {
                                kontakte_resource
                                    .get()
                                    .and_then(|r| r.ok())
                                    .unwrap_or_default()
                                    .into_iter()
                                    .map(|k| {
                                        let id_text = k.id.to_string();
                                        let id_fuer_auswahl = id_text.clone();
                                        let name = anzeigename(&k.vorname, &k.nachname);
                                        view! {
                                            <option
                                                value=id_text
                                                prop:selected=move || kontakt_input.get() == id_fuer_auswahl
                                            >
                                                {name}
                                            </option>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </select>
                        {move || {
                            kontakte_resource
                                .get()
                                .and_then(|r| r.ok())
                                .is_some_and(|liste| liste.is_empty())
                                .then(|| view! {
                                    <p class="text-xs text-warning mt-1">
                                        "Noch keine Kontakte vorhanden. Lege zuerst einen Kontakt an."
                                    </p>
                                })
                        }}
                    </div>

                    <div>
                        <label class="label py-1">
                            <span class="label-text font-medium text-sm">"Grund"</span>
                        </label>
                        <input
                            type="text"
                            placeholder="z.B. Darlehen, Restbetrag..."
                            class="input input-bordered w-full"
                            prop:value=move || grund_input.get()
                            on:input=move |ev| set_grund_input.set(event_target_value(&ev))
                            required
                        />
                    </div>

                    <div>
                        <label class="label py-1">
                            <span class="label-text font-medium text-sm">"Betrag (CHF)"</span>
                        </label>
                        <input
                            type="text"
                            inputmode="decimal"
                            placeholder="0.00"
                            class="input input-bordered w-full"
                            prop:value=move || betrag_input.get()
                            on:input=move |ev| set_betrag_input.set(event_target_value(&ev))
                            required
                        />
                    </div>

                    {move || error_msg.get().map(|msg| view! {
                        <div class="alert alert-warning py-2 text-sm">
                            <span>{msg}</span>
                        </div>
                    })}

                    <div class="modal-action mt-2">
                        {move || bearbeiten_id.get().map(|id| view! {
                            <button
                                type="button"
                                class="btn btn-outline btn-error mr-auto"
                                disabled=move || delete_action.pending().get()
                                on:click=move |_| loeschen_klick(id)
                            >
                                {move || if loesch_bestaetigen.get() { "Wirklich löschen?" } else { "Löschen" }}
                            </button>
                        })}
                        <button type="button" class="btn btn-ghost" on:click=move |_| set_modal_offen.set(false)>
                            "Abbrechen"
                        </button>
                        <button
                            type="submit"
                            class="btn btn-primary"
                            disabled=move || create_action.pending().get() || update_action.pending().get()
                        >
                            {move || if bearbeiten_id.get().is_some() { "Speichern" } else { "Hinzufügen" }}
                        </button>
                    </div>
                </form>
            </Modal>

            // Tabelle Schulden, nach Monat gruppiert (nur Ansicht; Doppelklick auf eine Zeile öffnet die Eingabemaske)
            <div class="overflow-x-auto -mx-2 sm:mx-0">
                <Suspense fallback=move || view! {
                    <div class="flex justify-center p-8">
                        <span class="loading loading-spinner loading-md text-primary"></span>
                    </div>
                }>
                    {move || {
                        schulden_resource.get().map(|res| match res {
                            Ok(items) => {
                                let leer = items.is_empty();
                                let monate = nach_monat(items);

                                view! {
                                    <table class="table table-zebra table-xs w-full">
                                        <thead>
                                            <tr class="text-xs uppercase text-base-content/70">
                                                <th class="w-32">"Datum"</th>
                                                <th>"Kontakt"</th>
                                                <th>"Grund"</th>
                                                <th class="text-right">"Betrag (CHF)"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {if leer {
                                                view! {
                                                    <tr>
                                                        <td colspan="4" class="text-center text-base-content/50 py-6">
                                                            "Noch keine Schulden erfasst."
                                                        </td>
                                                    </tr>
                                                }.into_any()
                                            } else {
                                                monate.into_iter().map(|(monat, eintraege)| {
                                                    let monatssumme: f64 = eintraege.iter().map(|s| s.betrag).sum();
                                                    let zeilen = eintraege.into_iter().map(|s| {
                                                        let datum_str = s.datum.format("%d.%m.%Y").to_string();
                                                        let betrag_str = format!("CHF {:.2}", s.betrag);
                                                        let kontakt_str = s.kontakt.clone().unwrap_or_else(|| "—".to_string());
                                                        let zeile = s.clone();

                                                        view! {
                                                            <tr
                                                                class="cursor-pointer select-none hover:bg-base-200/60"
                                                                title="Doppelklick zum Bearbeiten"
                                                                on:dblclick=move |_| bearbeiten_oeffnen(zeile.clone())
                                                            >
                                                                <td class="whitespace-nowrap font-mono text-base-content/80">{datum_str}</td>
                                                                <td class="font-medium">{kontakt_str}</td>
                                                                <td>{s.grund}</td>
                                                                <td class="text-right whitespace-nowrap font-mono">{betrag_str}</td>
                                                            </tr>
                                                        }
                                                    }).collect_view();

                                                    view! {
                                                        <tr class="bg-base-200 font-semibold text-sm">
                                                            <td colspan="4">{monat.clone()}</td>
                                                        </tr>
                                                        {zeilen}
                                                        <tr class="font-bold text-xs bg-base-200/50">
                                                            <td colspan="3">{format!("Summe {monat}")}</td>
                                                            <td class="text-right text-primary whitespace-nowrap font-mono">{format!("CHF {:.2}", monatssumme)}</td>
                                                        </tr>
                                                    }
                                                }).collect_view().into_any()
                                            }}
                                        </tbody>
                                    </table>
                                }.into_any()
                            }
                            Err(err) => view! {
                                <div class="alert alert-error">
                                    <span>{format!("Fehler beim Laden der Schulden: {err}")}</span>
                                </div>
                            }.into_any(),
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
