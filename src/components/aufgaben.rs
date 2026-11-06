use chrono::Local;
use leptos::prelude::*;

use crate::components::modal::Modal;
use crate::backend::aufgaben::{
    add_aufgabe, delete_aufgabe, get_aufgaben, toggle_aufgabe_erledigt, update_aufgabe,
};

/// Heutiges Datum im Format JJJJ-MM-TT (für das Datumsfeld).
fn heute_iso() -> String {
    Local::now().date_naive().format("%Y-%m-%d").to_string()
}

#[component]
pub fn AufgabenSection() -> impl IntoView {
    let (datum_input, set_datum_input) = signal(heute_iso());
    let (beschreibung, set_beschreibung) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (modal_offen, set_modal_offen) = signal(false);
    // Id der Aufgabe, die gerade bearbeitet wird (None = neue Aufgabe)
    let (bearbeiten_id, set_bearbeiten_id) = signal(Option::<uuid::Uuid>::None);

    let aufgaben_resource = Resource::new(|| (), |_| get_aufgaben());

    let create_action = Action::new(move |(beschr, datum): &(String, String)| {
        let beschr = beschr.clone();
        let datum = Some(datum.clone());
        async move { add_aufgabe(beschr, datum).await }
    });

    let update_action = Action::new(move |(id, beschr, datum): &(uuid::Uuid, String, String)| {
        let (id, beschr, datum) = (*id, beschr.clone(), datum.clone());
        async move { update_aufgabe(id, beschr, datum).await }
    });

    let toggle_action = Action::new(move |(id, erledigt): &(uuid::Uuid, bool)| {
        let id = *id;
        let erledigt = *erledigt;
        async move { toggle_aufgabe_erledigt(id, erledigt).await }
    });

    let delete_action = Action::new(move |id: &uuid::Uuid| {
        let id = *id;
        async move { delete_aufgabe(id).await }
    });

    // Formular leeren und schliessen (nach Erfolg)
    let formular_schliessen = move || {
        set_beschreibung.set(String::new());
        set_error_msg.set(None);
        set_bearbeiten_id.set(None);
        set_modal_offen.set(false);
    };

    Effect::new(move |_| {
        if let Some(res) = create_action.value().get() {
            match res {
                Ok(_) => {
                    formular_schliessen();
                    aufgaben_resource.refetch();
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
                    aufgaben_resource.refetch();
                }
                Err(e) => set_error_msg.set(Some(e.to_string())),
            }
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(())) = toggle_action.value().get() {
            aufgaben_resource.refetch();
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(())) = delete_action.value().get() {
            aufgaben_resource.refetch();
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let text = beschreibung.get();
        if text.trim().is_empty() {
            set_error_msg.set(Some("Bitte eine Beschreibung eingeben.".to_string()));
            return;
        }
        set_error_msg.set(None);
        match bearbeiten_id.get() {
            Some(id) => {
                update_action.dispatch((id, text, datum_input.get()));
            }
            None => {
                create_action.dispatch((text, datum_input.get()));
            }
        }
    };

    view! {
        <section class="card bg-base-100 shadow-sm border border-base-300">
            <div class="card-body p-4 sm:p-6 space-y-6">
                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                    <div>
                        <h2 class="card-title text-lg sm:text-xl">"Aufgaben"</h2>
                        <p class="text-xs sm:text-sm text-base-content/70">
                            "Erfassung und Übersicht deiner persönlichen Aufgaben mit Beschreibung, Datum und Status"
                        </p>
                    </div>
                    <button
                        type="button"
                        class="btn btn-primary btn-sm self-start sm:self-auto"
                        on:click=move |_| {
                            set_bearbeiten_id.set(None);
                            set_error_msg.set(None);
                            set_beschreibung.set(String::new());
                            set_datum_input.set(heute_iso());
                            set_modal_offen.set(true);
                        }
                    >
                        "+ Neue Aufgabe"
                    </button>
                </div>

                // Modalfenster zum Erfassen und Bearbeiten von Aufgaben
                <Modal
                    offen=modal_offen
                    on_close=Callback::new(move |_| set_modal_offen.set(false))
                    titel=move || if bearbeiten_id.get().is_some() { "Aufgabe bearbeiten".to_string() } else { "Neue Aufgabe".to_string() }
                >
                    <form on:submit=on_submit class="grid grid-cols-1 gap-3">
                        <div>
                            <label class="label py-1">
                                <span class="label-text font-medium text-sm">"Beschreibung"</span>
                            </label>
                            <input
                                type="text"
                                placeholder="z.B. Rechnung überweisen, Steuererklärung vorbereiten..."
                                class="input input-bordered w-full"
                                prop:value=move || beschreibung.get()
                                on:input=move |ev| set_beschreibung.set(event_target_value(&ev))
                                required
                            />
                        </div>

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

                        {move || error_msg.get().map(|msg| view! {
                            <div class="alert alert-warning py-2 text-sm">
                                <span>{msg}</span>
                            </div>
                        })}

                        <div class="modal-action mt-2">
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

                // Tabelle Aufgaben (Spalten: Erledigt, Datum, Beschreibung, Benutzer)
                <div class="overflow-x-auto -mx-2 sm:mx-0">
                    <Suspense fallback=move || view! {
                        <div class="flex justify-center p-8">
                            <span class="loading loading-spinner loading-md text-primary"></span>
                        </div>
                    }>
                        {move || {
                            aufgaben_resource.get().map(|res| match res {
                                Ok(items) => {
                                    let gesamt = items.len();
                                    let offene = items.iter().filter(|a| !a.erledigt).count();

                                    view! {
                                        <table class="table table-zebra table-sm sm:table-md w-full">
                                            <thead>
                                                <tr class="text-xs sm:text-sm uppercase text-base-content/70">
                                                    <th class="w-16 text-center">"Erledigt"</th>
                                                    <th class="w-32">"Datum"</th>
                                                    <th>"Beschreibung"</th>
                                                    <th class="w-40">"Benutzer"</th>
                                                    <th class="w-20 text-right"></th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {if items.is_empty() {
                                                    view! {
                                                        <tr>
                                                            <td colspan="5" class="text-center text-base-content/50 py-6">
                                                                "Noch keine Aufgaben erfasst."
                                                            </td>
                                                        </tr>
                                                    }.into_any()
                                                } else {
                                                    items.into_iter().map(|a| {
                                                        let datum_str = a.datum.format("%d.%m.%Y").to_string();
                                                        let id = a.id;
                                                        let is_erledigt = a.erledigt;
                                                        let bearbeiten = a.clone();

                                                        let datum_class = if is_erledigt {
                                                            "whitespace-nowrap font-mono text-xs sm:text-sm line-through text-base-content/40"
                                                        } else {
                                                            "whitespace-nowrap font-mono text-xs sm:text-sm text-base-content/80"
                                                        };

                                                        let beschreibung_class = if is_erledigt {
                                                            "font-medium line-through text-base-content/40"
                                                        } else {
                                                            "font-medium text-base-content"
                                                        };

                                                        view! {
                                                            <tr>
                                                                <td class="text-center">
                                                                    <input
                                                                        type="checkbox"
                                                                        class="checkbox checkbox-primary checkbox-sm"
                                                                        prop:checked=is_erledigt
                                                                        on:change=move |ev| {
                                                                            let checked = event_target_checked(&ev);
                                                                            toggle_action.dispatch((id, checked));
                                                                        }
                                                                    />
                                                                </td>
                                                                <td class=datum_class>{datum_str}</td>
                                                                <td class=beschreibung_class>{a.beschreibung}</td>
                                                                <td>
                                                                    <span class="badge badge-ghost badge-sm font-medium">
                                                                        {a.benutzer}
                                                                    </span>
                                                                </td>
                                                                <td class="text-right whitespace-nowrap">
                                                                    <button
                                                                        type="button"
                                                                        class="btn btn-ghost btn-xs"
                                                                        title="Aufgabe bearbeiten"
                                                                        aria-label="Aufgabe bearbeiten"
                                                                        on:click=move |_| {
                                                                            set_error_msg.set(None);
                                                                            set_bearbeiten_id.set(Some(id));
                                                                            set_beschreibung.set(bearbeiten.beschreibung.clone());
                                                                            set_datum_input.set(bearbeiten.datum.format("%Y-%m-%d").to_string());
                                                                            set_modal_offen.set(true);
                                                                        }
                                                                    >
                                                                        "✎"
                                                                    </button>
                                                                    <button
                                                                        type="button"
                                                                        class="btn btn-ghost btn-xs text-error"
                                                                        title="Aufgabe löschen"
                                                                        on:click=move |_| { delete_action.dispatch(id); }
                                                                    >
                                                                        "✕"
                                                                    </button>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }).collect_view().into_any()
                                                }}
                                            </tbody>
                                            <tfoot>
                                                <tr class="font-bold text-sm sm:text-base bg-base-200/50">
                                                    <td colspan="3">"Status"</td>
                                                    <td class="text-primary">
                                                        {format!("{offene} offen / {gesamt} gesamt")}
                                                    </td>
                                                    <td></td>
                                                </tr>
                                            </tfoot>
                                        </table>
                                    }.into_any()
                                }
                                Err(err) => view! {
                                    <div class="alert alert-error">
                                        <span>{format!("Fehler beim Laden der Aufgaben: {err}")}</span>
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
