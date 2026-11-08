use chrono::Local;
use leptos::prelude::*;

use crate::backend::fasten::{add_fasten, delete_fasten, dauer_minuten, get_fasten, update_fasten, Fasten};
use crate::components::modal::Modal;

/// Heutiges Datum im Format JJJJ-MM-TT (für das Datumsfeld).
fn heute_iso() -> String {
    Local::now().date_naive().format("%Y-%m-%d").to_string()
}

/// Dauer als Text, z.B. "16 h 00 min" oder "30 h 15 min".
fn dauer_text(minuten: i64) -> String {
    format!("{} h {:02} min", minuten / 60, minuten % 60)
}

#[component]
pub fn FastenSection() -> impl IntoView {
    let (datum_input, set_datum_input) = signal(heute_iso());
    // Beginn und Ende als Zeitpunkt im Format JJJJ-MM-TTTHH:MM (Eingabe "datetime-local")
    let (beginn_input, set_beginn_input) = signal(String::new());
    let (ende_input, set_ende_input) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (modal_offen, set_modal_offen) = signal(false);
    // Id des Fastens, das gerade bearbeitet wird (None = neues Fasten)
    let (bearbeiten_id, set_bearbeiten_id) = signal(Option::<uuid::Uuid>::None);
    // Zweiter Klick auf "Löschen" bestätigt das Löschen
    let (loesch_bestaetigen, set_loesch_bestaetigen) = signal(false);

    let fasten_resource = Resource::new(|| (), |_| get_fasten());

    let create_action = Action::new(
        move |(datum, beginn, ende): &(String, String, String)| {
            let (datum, beginn, ende) = (datum.clone(), beginn.clone(), ende.clone());
            async move { add_fasten(datum, beginn, ende).await }
        },
    );

    let update_action = Action::new(
        move |(id, datum, beginn, ende): &(uuid::Uuid, String, String, String)| {
            let (id, datum, beginn, ende) = (*id, datum.clone(), beginn.clone(), ende.clone());
            async move { update_fasten(id, datum, beginn, ende).await }
        },
    );

    let delete_action = Action::new(move |id: &uuid::Uuid| {
        let id = *id;
        async move { delete_fasten(id).await }
    });

    // Formular leeren und schliessen (nach Erfolg)
    let formular_schliessen = move || {
        set_beginn_input.set(String::new());
        set_ende_input.set(String::new());
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
                    fasten_resource.refetch();
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
                    fasten_resource.refetch();
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
                    fasten_resource.refetch();
                }
                Err(e) => {
                    set_loesch_bestaetigen.set(false);
                    set_error_msg.set(Some(e.to_string()));
                }
            }
        }
    });

    // Öffnet die Eingabemaske mit den Werten eines bestehenden Fastens (Doppelklick auf die Zeile)
    let bearbeiten_oeffnen = move |f: Fasten| {
        set_error_msg.set(None);
        set_loesch_bestaetigen.set(false);
        set_bearbeiten_id.set(Some(f.id));
        set_datum_input.set(f.datum.format("%Y-%m-%d").to_string());
        set_beginn_input.set(f.beginn.format("%Y-%m-%dT%H:%M").to_string());
        set_ende_input.set(f.ende.format("%Y-%m-%dT%H:%M").to_string());
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
        let beginn = beginn_input.get();
        let ende = ende_input.get();
        if beginn.trim().is_empty() || ende.trim().is_empty() {
            set_error_msg.set(Some("Bitte Beginn und Ende eingeben.".to_string()));
            return;
        }
        // Die Zeitpunkte haben alle das Format JJJJ-MM-TTTHH:MM, deshalb stimmt der Textvergleich mit der Reihenfolge überein
        if ende <= beginn {
            set_error_msg.set(Some("Das Ende muss nach dem Beginn liegen.".to_string()));
            return;
        }
        set_error_msg.set(None);
        match bearbeiten_id.get() {
            Some(id) => {
                update_action.dispatch((id, datum_input.get(), beginn, ende));
            }
            None => {
                create_action.dispatch((datum_input.get(), beginn, ende));
            }
        }
    };

    view! {
        <section class="card bg-base-100 shadow-sm border border-base-300">
            <div class="card-body p-4 sm:p-6 space-y-6">
                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                    <div>
                        <h2 class="card-title text-lg sm:text-xl">"Fasten"</h2>
                        <p class="text-xs sm:text-sm text-base-content/70">
                            "Erfassung deiner Fastenzeiten mit Datum, Beginn und Ende. Die Dauer wird berechnet."
                        </p>
                    </div>
                    <button
                        type="button"
                        class="btn btn-primary self-start sm:self-auto"
                        on:click=move |_| {
                            set_bearbeiten_id.set(None);
                            set_error_msg.set(None);
                            set_loesch_bestaetigen.set(false);
                            set_beginn_input.set(String::new());
                            set_ende_input.set(String::new());
                            set_datum_input.set(heute_iso());
                            set_modal_offen.set(true);
                        }
                    >
                        "+ Neues Fasten"
                    </button>
                </div>

                // Eingabemaske zum Erfassen, Bearbeiten und Löschen von Fastenzeiten
                <Modal
                    offen=modal_offen
                    on_close=Callback::new(move |_| set_modal_offen.set(false))
                    titel=move || if bearbeiten_id.get().is_some() { "Fasten bearbeiten".to_string() } else { "Neues Fasten".to_string() }
                >
                    <form on:submit=on_submit class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <div class="sm:col-span-2">
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
                                <span class="label-text font-medium text-sm">"Beginn"</span>
                            </label>
                            <input
                                type="datetime-local"
                                class="input input-bordered w-full"
                                prop:value=move || beginn_input.get()
                                on:input=move |ev| set_beginn_input.set(event_target_value(&ev))
                                required
                            />
                        </div>

                        <div>
                            <label class="label py-1">
                                <span class="label-text font-medium text-sm">"Ende"</span>
                            </label>
                            <input
                                type="datetime-local"
                                class="input input-bordered w-full"
                                prop:value=move || ende_input.get()
                                on:input=move |ev| set_ende_input.set(event_target_value(&ev))
                                required
                            />
                        </div>

                        <p class="text-xs text-base-content/60 sm:col-span-2">
                            "Das Ende muss nach dem Beginn liegen."
                        </p>

                        {move || error_msg.get().map(|msg| view! {
                            <div class="alert alert-warning py-2 text-sm sm:col-span-2">
                                <span>{msg}</span>
                            </div>
                        })}

                        <div class="modal-action sm:col-span-2 mt-2">
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

                // Card mit der Anzahl Tage, darüber die Tabelle
                <div class="overflow-x-auto -mx-2 sm:mx-0">
                    <Suspense fallback=move || view! {
                        <div class="flex justify-center p-8">
                            <span class="loading loading-spinner loading-md text-primary"></span>
                        </div>
                    }>
                        {move || {
                            fasten_resource.get().map(|res| match res {
                                Ok(items) => {
                                    let anzahl_tage = items.len();
                                    let leer = items.is_empty();

                                    view! {
                                        <div class="card bg-base-200 border border-base-300 mb-3">
                                            <div class="card-body p-4 gap-1">
                                                <span class="text-xs uppercase tracking-wide text-base-content/70">"Anzahl Tage"</span>
                                                <span class="text-2xl font-bold text-primary font-mono">{anzahl_tage}</span>
                                            </div>
                                        </div>

                                        <table class="table table-zebra table-xs w-full">
                                            <thead>
                                                <tr class="text-xs uppercase text-base-content/70">
                                                    <th class="w-32">"Datum"</th>
                                                    <th class="w-40">"Beginn"</th>
                                                    <th class="w-40">"Ende"</th>
                                                    <th class="text-right">"Dauer"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {if leer {
                                                    view! {
                                                        <tr>
                                                            <td colspan="4" class="text-center text-base-content/50 py-6">
                                                                "Noch keine Fastenzeiten erfasst."
                                                            </td>
                                                        </tr>
                                                    }.into_any()
                                                } else {
                                                    items.into_iter().map(|f| {
                                                        let datum_str = f.datum.format("%d.%m.%Y").to_string();
                                                        let beginn_str = f.beginn.format("%d.%m.%Y %H:%M").to_string();
                                                        let ende_str = f.ende.format("%d.%m.%Y %H:%M").to_string();
                                                        let dauer = dauer_text(dauer_minuten(f.beginn, f.ende));
                                                        let zeile = f.clone();

                                                        view! {
                                                            <tr
                                                                class="cursor-pointer select-none hover:bg-base-200/60"
                                                                title="Doppelklick zum Bearbeiten"
                                                                on:dblclick=move |_| bearbeiten_oeffnen(zeile.clone())
                                                            >
                                                                <td class="whitespace-nowrap font-mono text-base-content/80">{datum_str}</td>
                                                                <td class="whitespace-nowrap font-mono">{beginn_str}</td>
                                                                <td class="whitespace-nowrap font-mono">{ende_str}</td>
                                                                <td class="text-right whitespace-nowrap font-semibold">{dauer}</td>
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
                                        <span>{format!("Fehler beim Laden der Fastenzeiten: {err}")}</span>
                                    </div>
                                }.into_any(),
                            })
                        }}
                    </Suspense>
                </div>
            </div>
        </section>
    }
}
