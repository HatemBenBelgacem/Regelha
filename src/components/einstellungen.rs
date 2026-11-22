use leptos::prelude::*;
use uuid::Uuid;

use crate::backend::kategorien::{
    add_kategorie, delete_kategorie, get_kategorien, update_kategorie, Kategorie,
};
use crate::components::modal::Modal;

/// Einstellungsseite. Enthält die Verwaltung der Kategorien für Fixkosten und Fixeinnahmen.
#[component]
pub fn EinstellungenSection() -> impl IntoView {
    let (name_input, set_name_input) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (modal_offen, set_modal_offen) = signal(false);
    // Id der Kategorie, die gerade bearbeitet wird (None = neue Kategorie)
    let (bearbeiten_id, set_bearbeiten_id) = signal(Option::<Uuid>::None);
    // Zweiter Klick auf "Löschen" bestätigt das Löschen
    let (loesch_bestaetigen, set_loesch_bestaetigen) = signal(false);

    let kategorien_resource = Resource::new(|| (), |_| get_kategorien());

    let create_action = Action::new(move |name: &String| {
        let name = name.clone();
        async move { add_kategorie(name).await }
    });

    let update_action = Action::new(move |(id, name): &(Uuid, String)| {
        let (id, name) = (*id, name.clone());
        async move { update_kategorie(id, name).await }
    });

    let delete_action = Action::new(move |id: &Uuid| {
        let id = *id;
        async move { delete_kategorie(id).await }
    });

    // Formular leeren und schliessen (nach Erfolg)
    let formular_schliessen = move || {
        set_name_input.set(String::new());
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
                    kategorien_resource.refetch();
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
                    kategorien_resource.refetch();
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
                    kategorien_resource.refetch();
                }
                Err(e) => {
                    set_loesch_bestaetigen.set(false);
                    set_error_msg.set(Some(e.to_string()));
                }
            }
        }
    });

    // Öffnet die Eingabemaske mit dem Namen einer bestehenden Kategorie (Doppelklick auf die Zeile)
    let bearbeiten_oeffnen = move |k: Kategorie| {
        set_error_msg.set(None);
        set_loesch_bestaetigen.set(false);
        set_bearbeiten_id.set(Some(k.id));
        set_name_input.set(k.name.clone());
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
        let name = name_input.get();
        if name.trim().is_empty() {
            set_error_msg.set(Some("Bitte einen Namen für die Kategorie eingeben.".to_string()));
            return;
        }
        set_error_msg.set(None);
        match bearbeiten_id.get() {
            Some(id) => {
                update_action.dispatch((id, name));
            }
            None => {
                create_action.dispatch(name);
            }
        }
    };

    view! {
        <section class="card bg-base-100 shadow-sm border border-base-300">
            <div class="card-body p-4 sm:p-6 space-y-6">
                <div>
                    <h2 class="card-title text-lg sm:text-xl">"Einstellungen"</h2>
                    <p class="text-xs sm:text-sm text-base-content/70">
                        "Grundlagen für deine Module verwalten"
                    </p>
                </div>

                // Kategorien: Verwaltung in eigener Tabelle, Zuordnung bei Fixkosten und Fixeinnahmen
                <div class="space-y-4">
                    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                        <div>
                            <h3 class="font-semibold text-base sm:text-lg">"Kategorien"</h3>
                            <p class="text-xs sm:text-sm text-base-content/70">
                                "Kategorien ordnen Fixkosten und Fixeinnahmen. Eine Kategorie, die noch verwendet wird, kann nicht gelöscht werden."
                            </p>
                        </div>
                        <button
                            type="button"
                            class="btn btn-primary self-start sm:self-auto"
                            on:click=move |_| {
                                set_bearbeiten_id.set(None);
                                set_error_msg.set(None);
                                set_loesch_bestaetigen.set(false);
                                set_name_input.set(String::new());
                                set_modal_offen.set(true);
                            }
                        >
                            "+ Neue Kategorie"
                        </button>
                    </div>

                    // Eingabemaske zum Erfassen, Umbenennen und Löschen von Kategorien
                    <Modal
                        offen=modal_offen
                        on_close=Callback::new(move |_| set_modal_offen.set(false))
                        titel=move || if bearbeiten_id.get().is_some() { "Kategorie bearbeiten".to_string() } else { "Neue Kategorie".to_string() }
                    >
                        <form on:submit=on_submit class="grid grid-cols-1 gap-3">
                            <div>
                                <label class="label py-1">
                                    <span class="label-text font-medium text-sm">"Name"</span>
                                </label>
                                <input
                                    type="text"
                                    placeholder="z.B. Wohnen, Versicherung, Abos"
                                    maxlength="60"
                                    class="input input-bordered w-full"
                                    prop:value=move || name_input.get()
                                    on:input=move |ev| set_name_input.set(event_target_value(&ev))
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

                    // Tabelle Kategorien (Doppelklick auf eine Zeile öffnet die Eingabemaske)
                    <div class="overflow-x-auto -mx-2 sm:mx-0">
                        <Suspense fallback=move || view! {
                            <div class="flex justify-center p-8">
                                <span class="loading loading-spinner loading-md text-primary"></span>
                            </div>
                        }>
                            {move || {
                                kategorien_resource.get().map(|res| match res {
                                    Ok(items) => {
                                        let gesamt = items.len();

                                        view! {
                                            <table class="table table-zebra table-xs w-full">
                                                <thead>
                                                    <tr class="text-xs uppercase text-base-content/70">
                                                        <th>"Name"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {if items.is_empty() {
                                                        view! {
                                                            <tr>
                                                                <td class="text-center text-base-content/50 py-6">
                                                                    "Noch keine Kategorien angelegt."
                                                                </td>
                                                            </tr>
                                                        }.into_any()
                                                    } else {
                                                        items.into_iter().map(|k| {
                                                            let zeile = k.clone();

                                                            view! {
                                                                <tr
                                                                    class="cursor-pointer select-none hover:bg-base-200/60"
                                                                    title="Doppelklick zum Bearbeiten"
                                                                    on:dblclick=move |_| bearbeiten_oeffnen(zeile.clone())
                                                                >
                                                                    <td class="font-medium">{k.name}</td>
                                                                </tr>
                                                            }
                                                        }).collect_view().into_any()
                                                    }}
                                                </tbody>
                                                <tfoot>
                                                    <tr class="font-bold text-xs bg-base-200/50">
                                                        <td>
                                                            <div class="flex justify-between gap-3">
                                                                <span>"Anzahl Kategorien"</span>
                                                                <span class="text-primary">{gesamt}</span>
                                                            </div>
                                                        </td>
                                                    </tr>
                                                </tfoot>
                                            </table>
                                        }.into_any()
                                    }
                                    Err(err) => view! {
                                        <div class="alert alert-error">
                                            <span>{format!("Fehler beim Laden der Kategorien: {err}")}</span>
                                        </div>
                                    }.into_any(),
                                })
                            }}
                        </Suspense>
                    </div>
                </div>
            </div>
        </section>
    }
}
