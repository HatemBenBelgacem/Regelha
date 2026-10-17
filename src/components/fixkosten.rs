use leptos::prelude::*;

use crate::backend::budget::{add_fixkosten, delete_fixkosten, get_fixkosten};

#[component]
pub fn FixkostenSection() -> impl IntoView {
    let (buchungstext, set_buchungstext) = signal(String::new());
    let (betrag_input, set_betrag_input) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    let fixkosten_resource = Resource::new(|| (), |_| get_fixkosten());

    let create_action = Action::new(move |(text, betrag): &(String, f64)| {
        let text = text.clone();
        let betrag = *betrag;
        async move { add_fixkosten(text, betrag).await }
    });

    let delete_action = Action::new(move |id: &uuid::Uuid| {
        let id = *id;
        async move { delete_fixkosten(id).await }
    });

    Effect::new(move |_| {
        if let Some(res) = create_action.value().get() {
            match res {
                Ok(_) => {
                    set_buchungstext.set(String::new());
                    set_betrag_input.set(String::new());
                    set_error_msg.set(None);
                    fixkosten_resource.refetch();
                }
                Err(e) => set_error_msg.set(Some(e.to_string())),
            }
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(())) = delete_action.value().get() {
            fixkosten_resource.refetch();
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let text = buchungstext.get();
        let raw_amount = betrag_input.get().replace(',', ".");
        match raw_amount.trim().parse::<f64>() {
            Ok(val) if val > 0.0 => {
                set_error_msg.set(None);
                create_action.dispatch((text, val));
            }
            _ => {
                set_error_msg.set(Some(
                    "Bitte einen gültigen positiven Betrag eingeben (z.B. 950.00).".to_string(),
                ));
            }
        }
    };

    view! {
        <section class="card bg-base-100 shadow-sm border border-base-300">
            <div class="card-body p-4 sm:p-6 space-y-6">
                <div>
                    <h2 class="card-title text-lg sm:text-xl">"Fixkosten"</h2>
                    <p class="text-xs sm:text-sm text-base-content/70">
                        "Erfassung wiederkehrender monatlicher Fixkosten (Miete, Versicherungen, Abos)"
                    </p>
                </div>

                // Formular zum Erfassen neuer Fixkosten (Responsives Grid)
                <form on:submit=on_submit class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-12 gap-3 bg-base-200/60 p-3 sm:p-4 rounded-box items-end">
                    <div class="sm:col-span-2 lg:col-span-7">
                        <label class="label py-1">
                            <span class="label-text font-medium text-xs sm:text-sm">"Buchungstext"</span>
                        </label>
                        <input
                            type="text"
                            placeholder="z.B. Warmmiete, Strom, Haftpflicht..."
                            class="input input-bordered input-sm w-full"
                            prop:value=move || buchungstext.get()
                            on:input=move |ev| set_buchungstext.set(event_target_value(&ev))
                            required
                        />
                    </div>

                    <div class="sm:col-span-1 lg:col-span-3">
                        <label class="label py-1">
                            <span class="label-text font-medium text-xs sm:text-sm">"Betrag (€)"</span>
                        </label>
                        <input
                            type="text"
                            inputmode="decimal"
                            placeholder="0.00"
                            class="input input-bordered input-sm w-full"
                            prop:value=move || betrag_input.get()
                            on:input=move |ev| set_betrag_input.set(event_target_value(&ev))
                            required
                        />
                    </div>

                    <div class="sm:col-span-1 lg:col-span-2">
                        <button
                            type="submit"
                            class="btn btn-secondary btn-sm w-full"
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

                // Tabelle Fixkosten (Spalten: Buchungstext, Betrag)
                <div class="overflow-x-auto -mx-2 sm:mx-0">
                    <Suspense fallback=move || view! {
                        <div class="flex justify-center p-8">
                            <span class="loading loading-spinner loading-md text-secondary"></span>
                        </div>
                    }>
                        {move || {
                            fixkosten_resource.get().map(|res| match res {
                                Ok(items) => {
                                    let gesamt_fixkosten: f64 = items.iter().map(|f| f.betrag).sum();

                                    view! {
                                        <table class="table table-zebra table-sm sm:table-md w-full">
                                            <thead>
                                                <tr class="text-xs sm:text-sm uppercase text-base-content/70">
                                                    <th>"Buchungstext"</th>
                                                    <th class="text-right">"Betrag"</th>
                                                    <th class="w-12 text-right"></th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {if items.is_empty() {
                                                    view! {
                                                        <tr>
                                                            <td colspan="3" class="text-center text-base-content/50 py-6">
                                                                "Noch keine Fixkosten erfasst."
                                                            </td>
                                                        </tr>
                                                    }.into_any()
                                                } else {
                                                    items.into_iter().map(|f| {
                                                        let id = f.id;
                                                        view! {
                                                            <tr>
                                                                <td class="font-medium">{f.buchungstext}</td>
                                                                <td class="text-right font-semibold text-warning whitespace-nowrap">
                                                                    {format!("{:.2} €", f.betrag)}
                                                                </td>
                                                                <td class="text-right">
                                                                    <button
                                                                        type="button"
                                                                        class="btn btn-ghost btn-xs text-error"
                                                                        title="Fixkosten löschen"
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
                                                    <td>"Summe Fixkosten"</td>
                                                    <td class="text-right text-warning whitespace-nowrap">
                                                        {format!("{:.2} €", gesamt_fixkosten)}
                                                    </td>
                                                    <td></td>
                                                </tr>
                                            </tfoot>
                                        </table>
                                    }.into_any()
                                }
                                Err(err) => view! {
                                    <div class="alert alert-error">
                                        <span>{format!("Fehler beim Laden der Fixkosten: {err}")}</span>
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
