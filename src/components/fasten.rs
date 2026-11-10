use chrono::NaiveDateTime;
use leptos::prelude::*;

use crate::backend::fasten::{add_fasten, delete_fasten, dauer_minuten, get_fasten, update_fasten, Fasten};
use crate::components::modal::Modal;

/// Dauer als Text, z.B. "16 h 00 min" oder "30 h 15 min".
fn dauer_text(minuten: i64) -> String {
    format!("{} h {:02} min", minuten / 60, minuten % 60)
}

/// Laufende Dauer mit Sekunden, z.B. "5 h 03 min 12 s".
fn laufend_text(sekunden: i64) -> String {
    let sekunden = sekunden.max(0);
    format!("{} h {:02} min {:02} s", sekunden / 3600, sekunden % 3600 / 60, sekunden % 60)
}

/// Aktuelle Ortszeit des Browsers (Beginn und Ende werden ebenfalls als Ortszeit erfasst).
#[cfg(feature = "hydrate")]
fn browser_jetzt() -> Option<NaiveDateTime> {
    let d = js_sys::Date::new_0();
    chrono::NaiveDate::from_ymd_opt(d.get_full_year() as i32, d.get_month() + 1, d.get_date())?
        .and_hms_opt(d.get_hours(), d.get_minutes(), d.get_seconds())
}

#[component]
pub fn FastenSection() -> impl IntoView {
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

    // Aktuelle Zeit für laufende Fasten. Der Ticker läuft nur im Browser (kein Datenverkehr):
    // Auf dem Server und beim ersten Zeichnen ist der Wert None, damit die Hydration übereinstimmt.
    let (jetzt, set_jetzt) = signal(Option::<NaiveDateTime>::None);
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        set_jetzt.set(browser_jetzt());
        let handle = set_interval_with_handle(
            move || set_jetzt.set(browser_jetzt()),
            std::time::Duration::from_secs(1),
        );
        on_cleanup(move || {
            if let Ok(h) = handle {
                h.clear();
            }
        });
    });
    #[cfg(not(feature = "hydrate"))]
    let _ = set_jetzt;

    let create_action = Action::new(move |(beginn, ende): &(String, String)| {
        let (beginn, ende) = (beginn.clone(), ende.clone());
        async move { add_fasten(beginn, ende).await }
    });

    let update_action = Action::new(move |(id, beginn, ende): &(uuid::Uuid, String, String)| {
        let (id, beginn, ende) = (*id, beginn.clone(), ende.clone());
        async move { update_fasten(id, beginn, ende).await }
    });

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
        set_beginn_input.set(f.beginn.format("%Y-%m-%dT%H:%M").to_string());
        set_ende_input.set(f.ende.map(|e| e.format("%Y-%m-%dT%H:%M").to_string()).unwrap_or_default());
        set_modal_offen.set(true);
    };

    // Laufendes Fasten jetzt beenden (Ende = aktuelle Zeit im Browser)
    let beenden_klick = move |f: Fasten| {
        if let Some(n) = jetzt.get_untracked().filter(|n| *n > f.beginn) {
            update_action.dispatch((
                f.id,
                f.beginn.format("%Y-%m-%dT%H:%M:%S").to_string(),
                n.format("%Y-%m-%dT%H:%M:%S").to_string(),
            ));
        }
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
        if beginn.trim().is_empty() {
            set_error_msg.set(Some("Bitte einen Beginn eingeben.".to_string()));
            return;
        }
        // Die Zeitpunkte haben alle das Format JJJJ-MM-TTTHH:MM, deshalb stimmt der Textvergleich mit der Reihenfolge überein.
        // Ein leeres Ende heisst: das Fasten läuft noch.
        if !ende.trim().is_empty() && ende <= beginn {
            set_error_msg.set(Some("Das Ende muss nach dem Beginn liegen.".to_string()));
            return;
        }
        set_error_msg.set(None);
        match bearbeiten_id.get() {
            Some(id) => {
                update_action.dispatch((id, beginn, ende));
            }
            None => {
                create_action.dispatch((beginn, ende));
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
                            "Erfassung deiner Fastenzeiten mit Beginn und Ende. Die Dauer wird berechnet und läuft live mit, solange kein Ende gesetzt ist."
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
                                <span class="label-text font-medium text-sm">"Ende (optional)"</span>
                            </label>
                            <input
                                type="datetime-local"
                                class="input input-bordered w-full"
                                prop:value=move || ende_input.get()
                                on:input=move |ev| set_ende_input.set(event_target_value(&ev))
                            />
                        </div>

                        <p class="text-xs text-base-content/60 sm:col-span-2">
                            "Ende leer lassen, wenn das Fasten noch läuft. Das Ende muss nach dem Beginn liegen."
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

                // Card mit der Anzahl Tage, darunter die Fastenzeiten als Cards
                <div>
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

                                        <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-3 sm:gap-4">
                                            {if leer {
                                                view! {
                                                    <p class="col-span-full text-center text-base-content/50 py-6">
                                                        "Noch keine Fastenzeiten erfasst."
                                                    </p>
                                                }.into_any()
                                            } else {
                                                items.into_iter().map(|f| {
                                                    let beginn_str = f.beginn.format("%d.%m.%Y %H:%M").to_string();
                                                    let laeuft = f.ende.is_none();
                                                    let zeile = f.clone();
                                                    let zum_bearbeiten = f.clone();
                                                    let zum_beenden = f.clone();
                                                    let beginn = f.beginn;

                                                    // Ende: festes Datum oder "noch offen" (Fasten läuft)
                                                    let ende_text = match f.ende {
                                                        Some(ende) => ende.format("%d.%m.%Y %H:%M").to_string(),
                                                        None => "noch offen".to_string(),
                                                    };

                                                    // Dauer: fest berechnet oder live im Browser (jede Sekunde)
                                                    let dauer_anzeige = match f.ende {
                                                        Some(ende) => view! { <span>{dauer_text(dauer_minuten(beginn, ende))}</span> }.into_any(),
                                                        None => view! {
                                                            <span class="text-success">
                                                                {move || match jetzt.get() {
                                                                    Some(n) => laufend_text(n.signed_duration_since(beginn).num_seconds()),
                                                                    None => "…".to_string(),
                                                                }}
                                                            </span>
                                                        }.into_any(),
                                                    };

                                                    view! {
                                                        <div
                                                            class="card bg-base-200 border border-base-300 shadow-sm cursor-pointer select-none hover:border-primary/60 transition-colors"
                                                            title="Doppelklick zum Bearbeiten"
                                                            on:dblclick=move |_| bearbeiten_oeffnen(zeile.clone())
                                                        >
                                                            <div class="card-body p-4 gap-3">
                                                                <div class="flex items-start justify-between gap-2">
                                                                    <div>
                                                                        <p class="text-xs uppercase tracking-wide text-base-content/70">"Dauer"</p>
                                                                        <p class="text-2xl font-bold font-mono text-primary">{dauer_anzeige}</p>
                                                                    </div>
                                                                    {laeuft.then(|| view! {
                                                                        <span class="badge badge-success badge-sm text-white">"läuft"</span>
                                                                    })}
                                                                </div>

                                                                <div class="text-sm space-y-1">
                                                                    <div class="flex justify-between gap-2">
                                                                        <span class="text-xs uppercase tracking-wide text-base-content/70 self-center">"Beginn"</span>
                                                                        <span class="font-mono">{beginn_str}</span>
                                                                    </div>
                                                                    <div class="flex justify-between gap-2">
                                                                        <span class="text-xs uppercase tracking-wide text-base-content/70 self-center">"Ende"</span>
                                                                        <span class="font-mono">{ende_text}</span>
                                                                    </div>
                                                                </div>

                                                                <div class="card-actions justify-end">
                                                                    {laeuft.then(move || view! {
                                                                        <button
                                                                            type="button"
                                                                            class="btn btn-sm btn-outline btn-primary"
                                                                            disabled=move || update_action.pending().get()
                                                                            on:click=move |_| beenden_klick(zum_beenden.clone())
                                                                            on:dblclick=|ev| ev.stop_propagation()
                                                                        >
                                                                            "Beenden"
                                                                        </button>
                                                                    })}
                                                                    <button
                                                                        type="button"
                                                                        class="btn btn-sm btn-ghost"
                                                                        on:click=move |_| bearbeiten_oeffnen(zum_bearbeiten.clone())
                                                                        on:dblclick=|ev| ev.stop_propagation()
                                                                    >
                                                                        "Bearbeiten"
                                                                    </button>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    }
                                                }).collect_view().into_any()
                                            }}
                                        </div>
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
