use chrono::Local;
use leptos::prelude::*;

use crate::backend::aufgaben::{
    add_aufgabe, delete_aufgabe, get_aufgaben, toggle_aufgabe_erledigt,
};

#[component]
pub fn AufgabenSection() -> impl IntoView {
    let today_iso = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let (datum_input, set_datum_input) = signal(today_iso);
    let (beschreibung, set_beschreibung) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    let aufgaben_resource = Resource::new(|| (), |_| get_aufgaben());

    let create_action = Action::new(move |(beschr, datum): &(String, String)| {
        let beschr = beschr.clone();
        let datum = Some(datum.clone());
        async move { add_aufgabe(beschr, datum).await }
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

    Effect::new(move |_| {
        if let Some(res) = create_action.value().get() {
            match res {
                Ok(_) => {
                    set_beschreibung.set(String::new());
                    set_error_msg.set(None);
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
        create_action.dispatch((text, datum_input.get()));
    };

    view! {
        <section class="card bg-base-100 shadow-sm border border-base-300">
            <div class="card-body p-4 sm:p-6 space-y-6">
                <div>
                    <h2 class="card-title text-lg sm:text-xl">"Aufgaben"</h2>
                    <p class="text-xs sm:text-sm text-base-content/70">
                        "Erfassung und Übersicht deiner persönlichen Aufgaben mit Beschreibung, Datum und Status"
                    </p>
                </div>

                // Formular zum Erfassen neuer Aufgaben (Standardwert erledigt = false)
                <form on:submit=on_submit class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-12 gap-3 bg-base-200/60 p-3 sm:p-4 rounded-box items-end">
                    <div class="sm:col-span-2 lg:col-span-7">
                        <label class="label py-1">
                            <span class="label-text font-medium text-xs sm:text-sm">"Beschreibung"</span>
                        </label>
                        <input
                            type="text"
                            placeholder="z.B. Rechnung überweisen, Steuererklärung vorbereiten..."
                            class="input input-bordered input-sm w-full"
                            prop:value=move || beschreibung.get()
                            on:input=move |ev| set_beschreibung.set(event_target_value(&ev))
                            required
                        />
                    </div>

                    <div class="sm:col-span-1 lg:col-span-3">
                        <label class="label py-1">
                            <span class="label-text font-medium text-xs sm:text-sm">"Datum"</span>
                        </label>
                        <input
                            type="date"
                            class="input input-bordered input-sm w-full"
                            prop:value=move || datum_input.get()
                            on:input=move |ev| set_datum_input.set(event_target_value(&ev))
                            required
                        />
                    </div>

                    <div class="sm:col-span-1 lg:col-span-2">
                        <button
                            type="submit"
                            class="btn btn-primary btn-sm w-full"
                            disabled=move || create_action.pending().get()
                        >
                            "Hinzufügen"
                        </button>
                    </div>
                </form>

                {move || error_msg.get().map(|msg| view! {
                    <div class="alert alert-warning py-2 text-sm">
                        <span>{msg}</span>
                    </div>
                })}

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
                                                    <th class="w-12 text-right"></th>
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
                                                                <td class="text-right">
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
