use chrono::Local;
use leptos::prelude::*;

use crate::backend::kontakte::{anzeigename, get_kontakte};
use crate::backend::rueckzahlungen::{
    add_rueckzahlung, delete_rueckzahlung, get_rueckzahlungen, update_rueckzahlung,
    Rueckzahlung,
};
use crate::components::modal::Modal;

/// Heutiges Datum im Format JJJJ-MM-TT (für das Datumsfeld).
fn heute_iso() -> String {
    Local::now().date_naive().format("%Y-%m-%d").to_string()
}

/// Reiter Rückzahlungen (unter der Seite Schulden): Eingabemaske und Tabelle.
/// `stand` ist der gemeinsame Zähler der Seite. Jede Änderung erhöht ihn, damit auch die Card neu rechnet.
#[component]
pub fn RueckzahlungTab(stand: RwSignal<u32>) -> impl IntoView {
    let (datum_input, set_datum_input) = signal(heute_iso());
    // Id des gewählten Kontakts als Text (leer = noch kein Kontakt gewählt)
    let (kontakt_input, set_kontakt_input) = signal(String::new());
    let (betrag_input, set_betrag_input) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (modal_offen, set_modal_offen) = signal(false);
    // Id der Rückzahlung, die gerade bearbeitet wird (None = neue Rückzahlung)
    let (bearbeiten_id, set_bearbeiten_id) = signal(Option::<uuid::Uuid>::None);
    // Zweiter Klick auf "Löschen" bestätigt das Löschen
    let (loesch_bestaetigen, set_loesch_bestaetigen) = signal(false);

    let rueckzahlungen_resource = Resource::new(move || stand.get(), |_| get_rueckzahlungen());
    let kontakte_resource = Resource::new(|| (), |_| get_kontakte());

    let create_action = Action::new(
        move |(datum, betrag, kontakt_id): &(String, f64, uuid::Uuid)| {
            let (datum, betrag, kontakt_id) = (datum.clone(), *betrag, *kontakt_id);
            async move { add_rueckzahlung(datum, betrag, kontakt_id).await }
        },
    );

    let update_action = Action::new(
        move |(id, datum, betrag, kontakt_id): &(uuid::Uuid, String, f64, uuid::Uuid)| {
            let (id, datum, betrag, kontakt_id) = (*id, datum.clone(), *betrag, *kontakt_id);
            async move { update_rueckzahlung(id, datum, betrag, kontakt_id).await }
        },
    );

    let delete_action = Action::new(move |id: &uuid::Uuid| {
        let id = *id;
        async move { delete_rueckzahlung(id).await }
    });

    // Formular leeren und schliessen (nach Erfolg)
    let formular_schliessen = move || {
        set_kontakt_input.set(String::new());
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

    // Öffnet die Eingabemaske mit den Werten einer bestehenden Rückzahlung (Doppelklick auf die Zeile)
    let bearbeiten_oeffnen = move |r: Rueckzahlung| {
        set_error_msg.set(None);
        set_loesch_bestaetigen.set(false);
        set_bearbeiten_id.set(Some(r.id));
        set_datum_input.set(r.datum.format("%Y-%m-%d").to_string());
        set_kontakt_input.set(r.kontakt_id.to_string());
        set_betrag_input.set(format!("{:.2}", r.betrag));
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
                update_action.dispatch((id, datum_input.get(), betrag, kontakt_id));
            }
            None => {
                create_action.dispatch((datum_input.get(), betrag, kontakt_id));
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
                        set_betrag_input.set(String::new());
                        set_datum_input.set(heute_iso());
                        set_modal_offen.set(true);
                    }
                >
                    "+ Neue Rückzahlung"
                </button>
            </div>

            // Eingabemaske zum Erfassen, Bearbeiten und Löschen von Rückzahlungen
            <Modal
                offen=modal_offen
                on_close=Callback::new(move |_| set_modal_offen.set(false))
                titel=move || if bearbeiten_id.get().is_some() { "Rückzahlung bearbeiten".to_string() } else { "Neue Rückzahlung".to_string() }
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

            // Tabelle Rückzahlungen (nur Ansicht; Doppelklick auf eine Zeile öffnet die Eingabemaske)
            <div class="overflow-x-auto -mx-2 sm:mx-0">
                <Suspense fallback=move || view! {
                    <div class="flex justify-center p-8">
                        <span class="loading loading-spinner loading-md text-primary"></span>
                    </div>
                }>
                    {move || {
                        rueckzahlungen_resource.get().map(|res| match res {
                            Ok(items) => {
                                let leer = items.is_empty();

                                view! {
                                    <table class="table table-zebra table-xs w-full">
                                        <thead>
                                            <tr class="text-xs uppercase text-base-content/70">
                                                <th class="w-32">"Datum"</th>
                                                <th>"Kontakt"</th>
                                                <th class="text-right">"Betrag (CHF)"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {if leer {
                                                view! {
                                                    <tr>
                                                        <td colspan="3" class="text-center text-base-content/50 py-6">
                                                            "Noch keine Rückzahlungen erfasst."
                                                        </td>
                                                    </tr>
                                                }.into_any()
                                            } else {
                                                items.into_iter().map(|r| {
                                                    let datum_str = r.datum.format("%d.%m.%Y").to_string();
                                                    let betrag_str = format!("CHF {:.2}", r.betrag);
                                                    let zeile = r.clone();

                                                    view! {
                                                        <tr
                                                            class="cursor-pointer select-none hover:bg-base-200/60"
                                                            title="Doppelklick zum Bearbeiten"
                                                            on:dblclick=move |_| bearbeiten_oeffnen(zeile.clone())
                                                        >
                                                            <td class="whitespace-nowrap font-mono text-base-content/80">{datum_str}</td>
                                                            <td class="font-medium">{r.kontakt}</td>
                                                            <td class="text-right whitespace-nowrap font-mono">{betrag_str}</td>
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
                                    <span>{format!("Fehler beim Laden der Rückzahlungen: {err}")}</span>
                                </div>
                            }.into_any(),
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
