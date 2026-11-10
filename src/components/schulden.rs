use chrono::{Datelike, Local};
use leptos::prelude::*;
use uuid::Uuid;

use crate::backend::kontakte::{anzeigename, get_kontakte};
use crate::backend::schulden::{
    add_schuld, add_schuldenposition, delete_schuld, delete_schuldenposition, get_schulden,
    get_schuldenpositionen, update_schuld, update_schuldenposition, Schuld, Schuldenposition,
};
use crate::components::modal::Modal;
use crate::components::rueckzahlungen::RueckzahlungTab;

/// Ergebnis von `get_schulden`, wie es die Teile der Seite gemeinsam nutzen.
type SchuldenResource = Resource<Result<Vec<Schuld>, ServerFnError>>;

/// Heutiges Datum im Format JJJJ-MM-TT (für das Datumsfeld).
fn heute_iso() -> String {
    Local::now().date_naive().format("%Y-%m-%d").to_string()
}

/// Betrag als Text, z.B. "CHF 120.50".
fn chf(betrag: f64) -> String {
    // + 0.0 macht aus -0.0 (leere Summe) eine normale 0.0
    format!("CHF {:.2}", betrag + 0.0)
}

/// Deutsche Monatsnamen für die Gruppierung (Index 0 = Januar).
const MONATE: [&str; 12] = [
    "Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober",
    "November", "Dezember",
];

/// Gruppiert die Positionen nach Monat, z.B. ("November 2026", [...]).
/// Die Liste kommt bereits nach Datum absteigend sortiert, deshalb reicht ein Durchgang.
fn nach_monat(items: Vec<Schuldenposition>) -> Vec<(String, Vec<Schuldenposition>)> {
    let mut gruppen: Vec<((i32, u32), Vec<Schuldenposition>)> = Vec::new();
    for p in items {
        let schluessel = (p.datum.year(), p.datum.month());
        match gruppen.last_mut() {
            Some((k, liste)) if *k == schluessel => liste.push(p),
            _ => gruppen.push((schluessel, vec![p])),
        }
    }
    gruppen
        .into_iter()
        .map(|((jahr, monat), liste)| {
            (format!("{} {}", MONATE[(monat - 1) as usize], jahr), liste)
        })
        .collect()
}

/// CSS-Klasse für den Restbetrag: rot solange etwas offen ist, sonst grün.
fn rest_klasse(rest: f64) -> &'static str {
    if rest > 0.005 {
        "text-error"
    } else {
        "text-success"
    }
}

/// Restbetrag als Text. Ein negativer Wert ist ein Guthaben.
fn rest_text(rest: f64) -> String {
    if rest < -0.005 {
        format!("Guthaben {}", chf(rest.abs()))
    } else {
        chf(rest.max(0.0))
    }
}

/// Card mit Total Schulden, Rückzahlungen und Restbetrag.
fn saldo_card(total_schulden: f64, total_rueckzahlungen: f64) -> impl IntoView {
    let rest = total_schulden - total_rueckzahlungen;

    view! {
        <div class="card bg-base-200 border border-base-300">
            <div class="card-body p-4">
                <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
                    <div>
                        <span class="text-xs uppercase tracking-wide text-base-content/70">"Total Schulden"</span>
                        <div class="text-xl font-bold font-mono">{chf(total_schulden)}</div>
                    </div>
                    <div>
                        <span class="text-xs uppercase tracking-wide text-base-content/70">"Rückzahlungen"</span>
                        <div class="text-xl font-bold font-mono text-success">{chf(total_rueckzahlungen)}</div>
                    </div>
                    <div>
                        <span class="text-xs uppercase tracking-wide text-base-content/70">"Restbetrag"</span>
                        <div class=format!("text-2xl font-bold font-mono {}", rest_klasse(rest))>
                            {rest_text(rest)}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

fn spinner() -> impl IntoView {
    view! {
        <div class="flex justify-center p-6">
            <span class="loading loading-spinner loading-md text-primary"></span>
        </div>
    }
}

/// Die Reiter in der Detailansicht einer Schuld.
#[derive(Clone, Copy, PartialEq)]
enum Reiter {
    Positionen,
    Rueckzahlungen,
}

fn tab_klasse(aktiv: bool) -> &'static str {
    if aktiv {
        "tab tab-active font-semibold"
    } else {
        "tab"
    }
}

/// Seite Schulden: Übersicht aller Schulden. Ein Klick auf eine Schuld öffnet die Detailansicht.
#[component]
pub fn SchuldenSection() -> impl IntoView {
    // Zähler: Wird er erhöht, laden alle Listen neu (nach Anlegen, Ändern oder Löschen)
    let stand = RwSignal::new(0_u32);
    // Id der geöffneten Schuld (None = Übersicht)
    let ausgewaehlt = RwSignal::new(Option::<Uuid>::None);
    // Aktiver Reiter der Detailansicht; liegt hier, damit er beim Neuladen erhalten bleibt
    let reiter = RwSignal::new(Reiter::Positionen);

    let schulden_res: SchuldenResource = Resource::new(move || stand.get(), |_| get_schulden());

    view! {
        <section class="card bg-base-100 shadow-sm border border-base-300">
            <div class="card-body p-4 sm:p-6 space-y-6">
                {move || match ausgewaehlt.get() {
                    Some(id) => view! {
                        <SchuldDetail schuld_id=id schulden_res=schulden_res stand=stand ausgewaehlt=ausgewaehlt reiter=reiter/>
                    }.into_any(),
                    None => view! {
                        <SchuldenUebersicht schulden_res=schulden_res stand=stand ausgewaehlt=ausgewaehlt reiter=reiter/>
                    }.into_any(),
                }}
            </div>
        </section>
    }
}

/// Übersicht: Gesamtsaldo und je eine Card pro Schuld (Kontakt).
#[component]
fn SchuldenUebersicht(
    schulden_res: SchuldenResource,
    stand: RwSignal<u32>,
    ausgewaehlt: RwSignal<Option<Uuid>>,
    reiter: RwSignal<Reiter>,
) -> impl IntoView {
    let modal_offen = RwSignal::new(false);

    view! {
        <div class="flex flex-wrap items-start justify-between gap-3">
            <div>
                <h2 class="card-title text-lg sm:text-xl">"Schulden"</h2>
                <p class="text-xs sm:text-sm text-base-content/70">
                    "Eine Schuld pro Kontakt. Antippen für Positionen und Rückzahlungen."
                </p>
            </div>
            <button type="button" class="btn btn-primary" on:click=move |_| modal_offen.set(true)>
                "+ Neue Schuld"
            </button>
        </div>

        <SchuldModal
            offen=modal_offen
            schuld_id=None
            start_kontakt=Signal::derive(String::new)
            schulden_res=schulden_res
            stand=stand
            ausgewaehlt=ausgewaehlt
        />

        {move || match schulden_res.get() {
            None => spinner().into_any(),
            Some(Err(e)) => view! {
                <div class="alert alert-error">
                    <span>{format!("Fehler beim Laden der Schulden: {e}")}</span>
                </div>
            }.into_any(),
            Some(Ok(liste)) => {
                let total_schulden: f64 = liste.iter().map(|s| s.total_schulden).sum();
                let total_rueck: f64 = liste.iter().map(|s| s.total_rueckzahlungen).sum();

                let karten = if liste.is_empty() {
                    view! {
                        <p class="text-center text-base-content/50 py-6">"Noch keine Schulden erfasst."</p>
                    }.into_any()
                } else {
                    view! {
                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                            {liste.into_iter().map(|s| {
                                let id = s.id;
                                let rest = s.restbetrag();
                                view! {
                                    <button
                                        type="button"
                                        class="card bg-base-100 border border-base-300 text-left hover:border-primary hover:shadow-md transition"
                                        on:click=move |_| {
                                            reiter.set(Reiter::Positionen);
                                            ausgewaehlt.set(Some(id));
                                        }
                                    >
                                        <div class="card-body p-4 gap-2">
                                            <div class="flex items-center justify-between gap-2">
                                                <span class="font-semibold">{s.name()}</span>
                                                <span class=format!("font-mono font-bold {}", rest_klasse(rest))>
                                                    {rest_text(rest)}
                                                </span>
                                            </div>
                                            <div class="flex justify-between text-xs text-base-content/70 font-mono">
                                                <span>{format!("Schulden {}", chf(s.total_schulden))}</span>
                                                <span>{format!("Zurück {}", chf(s.total_rueckzahlungen))}</span>
                                            </div>
                                        </div>
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                };

                view! {
                    {saldo_card(total_schulden, total_rueck)}
                    {karten}
                }.into_any()
            }
        }}
    }
}

/// Detailansicht einer Schuld: Card mit Saldo, darunter die Reiter Schuldenpositionen und Rückzahlungen.
#[component]
fn SchuldDetail(
    schuld_id: Uuid,
    schulden_res: SchuldenResource,
    stand: RwSignal<u32>,
    ausgewaehlt: RwSignal<Option<Uuid>>,
    reiter: RwSignal<Reiter>,
) -> impl IntoView {
    let modal_offen = RwSignal::new(false);

    // Die aktuelle Schuld aus der Liste (wird nach jeder Änderung neu berechnet)
    let schuld = Memo::new(move |_| {
        schulden_res
            .get()
            .and_then(|r| r.ok())
            .and_then(|liste| liste.into_iter().find(|s| s.id == schuld_id))
    });
    // Nur der Kontakt: so wird der Reiter Rückzahlungen nicht bei jeder Summenänderung neu aufgebaut
    let kontakt_id = Memo::new(move |_| schuld.get().and_then(|s| s.kontakt_id));
    let start_kontakt = Signal::derive(move || {
        kontakt_id.get().map(|id| id.to_string()).unwrap_or_default()
    });

    view! {
        <div class="flex flex-wrap items-center justify-between gap-3">
            <div class="flex items-center gap-2 min-w-0">
                <button
                    type="button"
                    class="btn btn-ghost btn-sm btn-circle"
                    aria-label="Zurück zur Übersicht"
                    on:click=move |_| ausgewaehlt.set(None)
                >
                    "←"
                </button>
                <div class="min-w-0">
                    <h2 class="card-title text-lg sm:text-xl truncate">
                        {move || schuld.get().map(|s| s.name()).unwrap_or_else(|| "Schuld".to_string())}
                    </h2>
                    <p class="text-xs sm:text-sm text-base-content/70">"Schuldenpositionen und Rückzahlungen"</p>
                </div>
            </div>
            <button type="button" class="btn btn-outline btn-sm" on:click=move |_| modal_offen.set(true)>
                "Bearbeiten"
            </button>
        </div>

        <SchuldModal
            offen=modal_offen
            schuld_id=Some(schuld_id)
            start_kontakt=start_kontakt
            schulden_res=schulden_res
            stand=stand
            ausgewaehlt=ausgewaehlt
        />

        {move || match schulden_res.get() {
            None => spinner().into_any(),
            Some(Err(e)) => view! {
                <div class="alert alert-error">
                    <span>{format!("Fehler beim Laden: {e}")}</span>
                </div>
            }.into_any(),
            Some(Ok(_)) => match schuld.get() {
                Some(s) => saldo_card(s.total_schulden, s.total_rueckzahlungen).into_any(),
                None => view! {
                    <div class="alert alert-warning">
                        <span>"Diese Schuld gibt es nicht mehr."</span>
                    </div>
                }.into_any(),
            },
        }}

        <div role="tablist" class="tabs tabs-border">
            <button
                type="button"
                role="tab"
                class=move || tab_klasse(reiter.get() == Reiter::Positionen)
                on:click=move |_| reiter.set(Reiter::Positionen)
            >
                "Schuldenpositionen"
            </button>
            <button
                type="button"
                role="tab"
                class=move || tab_klasse(reiter.get() == Reiter::Rueckzahlungen)
                on:click=move |_| reiter.set(Reiter::Rueckzahlungen)
            >
                "Rückzahlungen"
            </button>
        </div>

        {move || match reiter.get() {
            Reiter::Positionen => view! {
                <SchuldenpositionenTab schuld_id=schuld_id stand=stand/>
            }.into_any(),
            Reiter::Rueckzahlungen => match kontakt_id.get() {
                Some(k) => view! { <RueckzahlungTab stand=stand kontakt_id=k/> }.into_any(),
                None => view! {
                    <p class="text-sm text-base-content/60">
                        "Diese Schuld hat keinen Kontakt. Bitte zuerst einen Kontakt zuweisen."
                    </p>
                }.into_any(),
            },
        }}
    }
}

/// Eingabemaske für eine Schuld: nur der Kontakt. Beim Bearbeiten auch Löschen.
#[component]
fn SchuldModal(
    offen: RwSignal<bool>,
    /// None = neue Schuld, Some = bestehende Schuld bearbeiten
    schuld_id: Option<Uuid>,
    /// Vorbelegter Kontakt (Id als Text) beim Öffnen
    start_kontakt: Signal<String>,
    schulden_res: SchuldenResource,
    stand: RwSignal<u32>,
    ausgewaehlt: RwSignal<Option<Uuid>>,
) -> impl IntoView {
    let (kontakt_input, set_kontakt_input) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (loesch_bestaetigen, set_loesch_bestaetigen) = signal(false);

    let kontakte_resource = Resource::new(|| (), |_| get_kontakte());

    // Beim Öffnen die Felder zurücksetzen
    Effect::new(move |_| {
        if offen.get() {
            set_kontakt_input.set(start_kontakt.get_untracked());
            set_error_msg.set(None);
            set_loesch_bestaetigen.set(false);
        }
    });

    let create_action = Action::new(move |kontakt_id: &Uuid| {
        let kontakt_id = *kontakt_id;
        async move { add_schuld(kontakt_id).await }
    });
    let update_action = Action::new(move |(id, kontakt_id): &(Uuid, Uuid)| {
        let (id, kontakt_id) = (*id, *kontakt_id);
        async move { update_schuld(id, kontakt_id).await }
    });
    let delete_action = Action::new(move |id: &Uuid| {
        let id = *id;
        async move { delete_schuld(id).await }
    });

    Effect::new(move |_| {
        if let Some(res) = create_action.value().get() {
            match res {
                Ok(neue_id) => {
                    offen.set(false);
                    stand.update(|v| *v += 1);
                    // Neue Schuld gleich öffnen, damit Positionen erfasst werden können
                    ausgewaehlt.set(Some(neue_id));
                }
                Err(e) => set_error_msg.set(Some(e.to_string())),
            }
        }
    });

    Effect::new(move |_| {
        if let Some(res) = update_action.value().get() {
            match res {
                Ok(()) => {
                    offen.set(false);
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
                    offen.set(false);
                    ausgewaehlt.set(None);
                    stand.update(|v| *v += 1);
                }
                Err(e) => {
                    set_loesch_bestaetigen.set(false);
                    set_error_msg.set(Some(e.to_string()));
                }
            }
        }
    });

    let loeschen_klick = move |id: Uuid| {
        if loesch_bestaetigen.get() {
            delete_action.dispatch(id);
        } else {
            set_loesch_bestaetigen.set(true);
        }
    };

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let kontakt_id = match Uuid::parse_str(&kontakt_input.get()) {
            Ok(id) => id,
            Err(_) => {
                set_error_msg.set(Some("Bitte einen Kontakt auswählen.".to_string()));
                return;
            }
        };
        set_error_msg.set(None);
        match schuld_id {
            Some(id) => {
                update_action.dispatch((id, kontakt_id));
            }
            None => {
                create_action.dispatch(kontakt_id);
            }
        }
    };

    // Kontakte, die schon eine (andere) Schuld haben, werden nicht angeboten
    let freie_kontakte = move || {
        let belegt: Vec<Uuid> = schulden_res
            .get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
            .into_iter()
            .filter(|s| Some(s.id) != schuld_id)
            .filter_map(|s| s.kontakt_id)
            .collect();
        kontakte_resource
            .get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
            .into_iter()
            .filter(|k| !belegt.contains(&k.id))
            .collect::<Vec<_>>()
    };

    view! {
        <Modal
            offen=offen
            on_close=Callback::new(move |_| offen.set(false))
            titel=move || if schuld_id.is_some() { "Schuld bearbeiten".to_string() } else { "Neue Schuld".to_string() }
        >
            <form on:submit=on_submit class="grid grid-cols-1 gap-3">
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
                            freie_kontakte()
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
                        (kontakte_resource.get().is_some() && freie_kontakte().is_empty()).then(|| view! {
                            <p class="text-xs text-warning mt-1">
                                "Kein freier Kontakt vorhanden. Lege zuerst einen neuen Kontakt an."
                            </p>
                        })
                    }}
                </div>

                {move || error_msg.get().map(|msg| view! {
                    <div class="alert alert-warning py-2 text-sm">
                        <span>{msg}</span>
                    </div>
                })}

                {schuld_id.is_some().then(|| view! {
                    <p class="text-xs text-base-content/60">
                        "Beim Löschen werden auch alle Schuldenpositionen gelöscht. Die Rückzahlungen bleiben beim Kontakt."
                    </p>
                })}

                <div class="modal-action mt-2">
                    {schuld_id.map(|id| view! {
                        <button
                            type="button"
                            class="btn btn-outline btn-error mr-auto"
                            disabled=move || delete_action.pending().get()
                            on:click=move |_| loeschen_klick(id)
                        >
                            {move || if loesch_bestaetigen.get() { "Wirklich löschen?" } else { "Löschen" }}
                        </button>
                    })}
                    <button type="button" class="btn btn-ghost" on:click=move |_| offen.set(false)>
                        "Abbrechen"
                    </button>
                    <button
                        type="submit"
                        class="btn btn-primary"
                        disabled=move || create_action.pending().get() || update_action.pending().get()
                    >
                        {if schuld_id.is_some() { "Speichern" } else { "Hinzufügen" }}
                    </button>
                </div>
            </form>
        </Modal>
    }
}

/// Reiter Schuldenpositionen: Eingabemaske und Tabelle, nach Monat gruppiert mit Monatssumme.
#[component]
fn SchuldenpositionenTab(schuld_id: Uuid, stand: RwSignal<u32>) -> impl IntoView {
    let (datum_input, set_datum_input) = signal(heute_iso());
    let (betrag_input, set_betrag_input) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (modal_offen, set_modal_offen) = signal(false);
    // Id der Position, die gerade bearbeitet wird (None = neue Position)
    let (bearbeiten_id, set_bearbeiten_id) = signal(Option::<Uuid>::None);
    // Zweiter Klick auf "Löschen" bestätigt das Löschen
    let (loesch_bestaetigen, set_loesch_bestaetigen) = signal(false);

    let positionen_resource =
        Resource::new(move || stand.get(), move |_| get_schuldenpositionen(schuld_id));

    let create_action = Action::new(move |(datum, betrag): &(String, f64)| {
        let (datum, betrag) = (datum.clone(), *betrag);
        async move { add_schuldenposition(schuld_id, datum, betrag).await }
    });

    let update_action = Action::new(move |(id, datum, betrag): &(Uuid, String, f64)| {
        let (id, datum, betrag) = (*id, datum.clone(), *betrag);
        async move { update_schuldenposition(id, datum, betrag).await }
    });

    let delete_action = Action::new(move |id: &Uuid| {
        let id = *id;
        async move { delete_schuldenposition(id).await }
    });

    // Formular leeren und schliessen (nach Erfolg)
    let formular_schliessen = move || {
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

    // Öffnet die Eingabemaske mit den Werten einer bestehenden Position (Doppelklick auf die Zeile)
    let bearbeiten_oeffnen = move |p: Schuldenposition| {
        set_error_msg.set(None);
        set_loesch_bestaetigen.set(false);
        set_bearbeiten_id.set(Some(p.id));
        set_datum_input.set(p.datum.format("%Y-%m-%d").to_string());
        set_betrag_input.set(format!("{:.2}", p.betrag));
        set_modal_offen.set(true);
    };

    // Erster Klick fragt nach, zweiter Klick löscht
    let loeschen_klick = move |id: Uuid| {
        if loesch_bestaetigen.get() {
            delete_action.dispatch(id);
        } else {
            set_loesch_bestaetigen.set(true);
        }
    };

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
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
                update_action.dispatch((id, datum_input.get(), betrag));
            }
            None => {
                create_action.dispatch((datum_input.get(), betrag));
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
                        set_betrag_input.set(String::new());
                        set_datum_input.set(heute_iso());
                        set_modal_offen.set(true);
                    }
                >
                    "+ Neue Position"
                </button>
            </div>

            // Eingabemaske zum Erfassen, Bearbeiten und Löschen von Positionen
            <Modal
                offen=modal_offen
                on_close=Callback::new(move |_| set_modal_offen.set(false))
                titel=move || if bearbeiten_id.get().is_some() { "Position bearbeiten".to_string() } else { "Neue Position".to_string() }
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

            // Tabelle Positionen, nach Monat gruppiert (Doppelklick auf eine Zeile öffnet die Eingabemaske)
            <div class="overflow-x-auto -mx-2 sm:mx-0">
                <Suspense fallback=spinner>
                    {move || {
                        positionen_resource.get().map(|res| match res {
                            Ok(items) => {
                                let leer = items.is_empty();
                                let total: f64 = items.iter().map(|p| p.betrag).sum();
                                let monate = nach_monat(items);

                                view! {
                                    <table class="table table-zebra table-xs w-full">
                                        <thead>
                                            <tr class="text-xs uppercase text-base-content/70">
                                                <th class="w-32">"Datum"</th>
                                                <th class="text-right">"Betrag (CHF)"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {if leer {
                                                view! {
                                                    <tr>
                                                        <td colspan="2" class="text-center text-base-content/50 py-6">
                                                            "Noch keine Positionen erfasst."
                                                        </td>
                                                    </tr>
                                                }.into_any()
                                            } else {
                                                monate.into_iter().map(|(monat, eintraege)| {
                                                    let monatssumme: f64 = eintraege.iter().map(|p| p.betrag).sum();
                                                    let zeilen = eintraege.into_iter().map(|p| {
                                                        let datum_str = p.datum.format("%d.%m.%Y").to_string();
                                                        let betrag_str = chf(p.betrag);
                                                        view! {
                                                            <tr
                                                                class="cursor-pointer select-none hover:bg-base-200/60"
                                                                title="Doppelklick zum Bearbeiten"
                                                                on:dblclick=move |_| bearbeiten_oeffnen(p.clone())
                                                            >
                                                                <td class="whitespace-nowrap font-mono text-base-content/80">{datum_str}</td>
                                                                <td class="text-right whitespace-nowrap font-mono">{betrag_str}</td>
                                                            </tr>
                                                        }
                                                    }).collect_view();

                                                    view! {
                                                        <tr class="bg-base-200 font-semibold text-sm">
                                                            <td colspan="2">{monat.clone()}</td>
                                                        </tr>
                                                        {zeilen}
                                                        <tr class="font-bold text-xs bg-base-200/50">
                                                            <td>{format!("Summe {monat}")}</td>
                                                            <td class="text-right text-primary whitespace-nowrap font-mono">{chf(monatssumme)}</td>
                                                        </tr>
                                                    }
                                                }).collect_view().into_any()
                                            }}
                                        </tbody>
                                        {(!leer).then(|| view! {
                                            <tfoot>
                                                <tr class="font-bold text-xs">
                                                    <td>"Total"</td>
                                                    <td class="text-right whitespace-nowrap font-mono">{chf(total)}</td>
                                                </tr>
                                            </tfoot>
                                        })}
                                    </table>
                                }.into_any()
                            }
                            Err(err) => view! {
                                <div class="alert alert-error">
                                    <span>{format!("Fehler beim Laden der Positionen: {err}")}</span>
                                </div>
                            }.into_any(),
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
