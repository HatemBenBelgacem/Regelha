use leptos::prelude::*;

use crate::backend::kontakte::{
    add_kontakt, delete_kontakt, get_kontakte, update_kontakt, Kontakt,
};
use crate::components::modal::Modal;

#[component]
pub fn KontakteSection() -> impl IntoView {
    let (vorname_input, set_vorname_input) = signal(String::new());
    let (nachname_input, set_nachname_input) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (modal_offen, set_modal_offen) = signal(false);
    // Id des Kontakts, der gerade bearbeitet wird (None = neuer Kontakt)
    let (bearbeiten_id, set_bearbeiten_id) = signal(Option::<uuid::Uuid>::None);
    // Zweiter Klick auf "Löschen" bestätigt das Löschen
    let (loesch_bestaetigen, set_loesch_bestaetigen) = signal(false);

    let kontakte_resource = Resource::new(|| (), |_| get_kontakte());

    let create_action = Action::new(move |(vorname, nachname): &(String, String)| {
        let (vorname, nachname) = (vorname.clone(), nachname.clone());
        async move { add_kontakt(vorname, nachname).await }
    });

    let update_action = Action::new(move |(id, vorname, nachname): &(uuid::Uuid, String, String)| {
        let (id, vorname, nachname) = (*id, vorname.clone(), nachname.clone());
        async move { update_kontakt(id, vorname, nachname).await }
    });

    let delete_action = Action::new(move |id: &uuid::Uuid| {
        let id = *id;
        async move { delete_kontakt(id).await }
    });

    // Formular leeren und schliessen (nach Erfolg)
    let formular_schliessen = move || {
        set_vorname_input.set(String::new());
        set_nachname_input.set(String::new());
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
                    kontakte_resource.refetch();
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
                    kontakte_resource.refetch();
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
                    kontakte_resource.refetch();
                }
                Err(e) => {
                    set_loesch_bestaetigen.set(false);
                    set_error_msg.set(Some(e.to_string()));
                }
            }
        }
    });

    // Öffnet die Eingabemaske mit den Werten eines bestehenden Kontakts (Klick auf die Zeile)
    let bearbeiten_oeffnen = move |k: Kontakt| {
        set_error_msg.set(None);
        set_loesch_bestaetigen.set(false);
        set_bearbeiten_id.set(Some(k.id));
        set_vorname_input.set(k.vorname.clone());
        set_nachname_input.set(k.nachname.clone());
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
        let vorname = vorname_input.get();
        if vorname.trim().is_empty() {
            set_error_msg.set(Some("Bitte einen Vornamen eingeben.".to_string()));
            return;
        }
        set_error_msg.set(None);
        let nachname = nachname_input.get();
        match bearbeiten_id.get() {
            Some(id) => {
                update_action.dispatch((id, vorname, nachname));
            }
            None => {
                create_action.dispatch((vorname, nachname));
            }
        }
    };

    view! {
        <section class="card bg-base-100 shadow-sm border border-base-300">
            <div class="card-body p-4 sm:p-6 space-y-6">
                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                    <div>
                        <h2 class="card-title text-lg sm:text-xl">"Kontakte"</h2>
                        <p class="text-xs sm:text-sm text-base-content/70">
                            "Deine persönlichen Kontakte mit Vor- und Nachname"
                        </p>
                    </div>
                    <button
                        type="button"
                        class="btn btn-primary self-start sm:self-auto"
                        on:click=move |_| {
                            set_bearbeiten_id.set(None);
                            set_error_msg.set(None);
                            set_loesch_bestaetigen.set(false);
                            set_vorname_input.set(String::new());
                            set_nachname_input.set(String::new());
                            set_modal_offen.set(true);
                        }
                    >
                        "+ Neuer Kontakt"
                    </button>
                </div>

                // Eingabemaske zum Erfassen, Bearbeiten und Löschen von Kontakten
                <Modal
                    offen=modal_offen
                    on_close=Callback::new(move |_| set_modal_offen.set(false))
                    titel=move || if bearbeiten_id.get().is_some() { "Kontakt bearbeiten".to_string() } else { "Neuer Kontakt".to_string() }
                >
                    <form on:submit=on_submit class="grid grid-cols-1 gap-3">
                        <div>
                            <label class="label py-1">
                                <span class="label-text font-medium text-sm">"Vorname"</span>
                            </label>
                            <input
                                type="text"
                                placeholder="z.B. Max"
                                class="input input-bordered w-full"
                                prop:value=move || vorname_input.get()
                                on:input=move |ev| set_vorname_input.set(event_target_value(&ev))
                                required
                            />
                        </div>

                        <div>
                            <label class="label py-1">
                                <span class="label-text font-medium text-sm">"Nachname"</span>
                            </label>
                            <input
                                type="text"
                                placeholder="z.B. Muster"
                                class="input input-bordered w-full"
                                prop:value=move || nachname_input.get()
                                on:input=move |ev| set_nachname_input.set(event_target_value(&ev))
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

                // Tabelle Kontakte (nur Ansicht; Klick auf eine Zeile öffnet die Eingabemaske)
                <div class="overflow-x-auto -mx-2 sm:mx-0">
                    <Suspense fallback=move || view! {
                        <div class="flex justify-center p-8">
                            <span class="loading loading-spinner loading-md text-primary"></span>
                        </div>
                    }>
                        {move || {
                            kontakte_resource.get().map(|res| match res {
                                Ok(items) => {
                                    let gesamt = items.len();

                                    view! {
                                        <table class="table table-zebra table-xs w-full">
                                            <thead>
                                                <tr class="text-xs uppercase text-base-content/70">
                                                    <th>"Vorname"</th>
                                                    <th>"Nachname"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {if items.is_empty() {
                                                    view! {
                                                        <tr>
                                                            <td colspan="2" class="text-center text-base-content/50 py-6">
                                                                "Noch keine Kontakte erfasst."
                                                            </td>
                                                        </tr>
                                                    }.into_any()
                                                } else {
                                                    items.into_iter().map(|k| {
                                                        let zeile = k.clone();

                                                        view! {
                                                            <tr
                                                                class="cursor-pointer select-none hover:bg-base-200/60"
                                                                title="Zum Bearbeiten klicken"
                                                                on:click=move |_| bearbeiten_oeffnen(zeile.clone())
                                                            >
                                                                <td class="font-medium">{k.vorname}</td>
                                                                <td>{k.nachname}</td>
                                                            </tr>
                                                        }
                                                    }).collect_view().into_any()
                                                }}
                                            </tbody>
                                            <tfoot>
                                                <tr class="font-bold text-xs bg-base-200/50">
                                                    <td>"Anzahl Kontakte"</td>
                                                    <td class="text-right text-primary">{gesamt}</td>
                                                </tr>
                                            </tfoot>
                                        </table>
                                    }.into_any()
                                }
                                Err(err) => view! {
                                    <div class="alert alert-error">
                                        <span>{format!("Fehler beim Laden der Kontakte: {err}")}</span>
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
