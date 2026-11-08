use chrono::Local;
use leptos::prelude::*;

use crate::backend::schulden::{add_schuld, delete_schuld, get_schulden, update_schuld, Schuld};
use crate::components::modal::Modal;

/// Heutiges Datum im Format JJJJ-MM-TT (für das Datumsfeld).
fn heute_iso() -> String {
    Local::now().date_naive().format("%Y-%m-%d").to_string()
}

#[component]
pub fn SchuldenSection() -> impl IntoView {
    let (datum_input, set_datum_input) = signal(heute_iso());
    let (grund_input, set_grund_input) = signal(String::new());
    let (betrag_input, set_betrag_input) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (modal_offen, set_modal_offen) = signal(false);
    // Id der Schuld, die gerade bearbeitet wird (None = neue Schuld)
    let (bearbeiten_id, set_bearbeiten_id) = signal(Option::<uuid::Uuid>::None);

    let schulden_resource = Resource::new(|| (), |_| get_schulden());

    let create_action = Action::new(move |(datum, grund, betrag): &(String, String, f64)| {
        let (datum, grund, betrag) = (datum.clone(), grund.clone(), *betrag);
        async move { add_schuld(datum, grund, betrag).await }
    });

    let update_action = Action::new(
        move |(id, datum, grund, betrag): &(uuid::Uuid, String, String, f64)| {
            let (id, datum, grund, betrag) = (*id, datum.clone(), grund.clone(), *betrag);
            async move { update_schuld(id, datum, grund, betrag).await }
        },
    );

    let delete_action = Action::new(move |id: &uuid::Uuid| {
        let id = *id;
        async move { delete_schuld(id).await }
    });

    // Formular leeren und schliessen (nach Erfolg)
    let formular_schliessen = move || {
        set_grund_input.set(String::new());
        set_betrag_input.set(String::new());
        set_error_msg.set(None);
        set_bearbeiten_id.set(None);
        set_modal_offen.set(false);
    };

    Effect::new(move |_| {
        if let Some(res) = create_action.value().get() {
            match res {
                Ok(_) => {
                    formular_schliessen();
                    schulden_resource.refetch();
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
                    schulden_resource.refetch();
                }
                Err(e) => set_error_msg.set(Some(e.to_string())),
            }
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(())) = delete_action.value().get() {
            schulden_resource.refetch();
        }
    });

    // Öffnet das Formular mit den Werten einer bestehenden Schuld (Doppelklick oder ✎)
    let bearbeiten_oeffnen = move |s: Schuld| {
        set_error_msg.set(None);
        set_bearbeiten_id.set(Some(s.id));
        set_datum_input.set(s.datum.format("%Y-%m-%d").to_string());
        set_grund_input.set(s.grund.clone());
        set_betrag_input.set(format!("{:.2}", s.betrag));
        set_modal_offen.set(true);
    };

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
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
                update_action.dispatch((id, datum_input.get(), grund, betrag));
            }
            None => {
                create_action.dispatch((datum_input.get(), grund, betrag));
            }
        }
    };

    view! {
        <section class="card bg-base-100 shadow-sm border border-base-300">
            <div class="card-body p-4 sm:p-6 space-y-6">
                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                    <div>
                        <h2 class="card-title text-lg sm:text-xl">"Schulden"</h2>
                        <p class="text-xs sm:text-sm text-base-content/70">
                            "Erfassung deiner Schulden mit Datum, Grund und Betrag in CHF"
                        </p>
                    </div>
                    <button
                        type="button"
                        class="btn btn-primary self-start sm:self-auto"
                        on:click=move |_| {
                            set_bearbeiten_id.set(None);
                            set_error_msg.set(None);
                            set_grund_input.set(String::new());
                            set_betrag_input.set(String::new());
                            set_datum_input.set(heute_iso());
                            set_modal_offen.set(true);
                        }
                    >
                        "+ Neue Schuld"
                    </button>
                </div>

                // Modalfenster zum Erfassen und Bearbeiten von Schulden
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
                                <span class="label-text font-medium text-sm">"Grund"</span>
                            </label>
                            <input
                                type="text"
                                placeholder="z.B. Darlehen von Freund, Restbetrag..."
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

                // Tabelle Schulden (Spalten: Datum, Grund, Betrag)
                <div class="overflow-x-auto -mx-2 sm:mx-0">
                    <Suspense fallback=move || view! {
                        <div class="flex justify-center p-8">
                            <span class="loading loading-spinner loading-md text-primary"></span>
                        </div>
                    }>
                        {move || {
                            schulden_resource.get().map(|res| match res {
                                Ok(items) => {
                                    let summe: f64 = items.iter().map(|s| s.betrag).sum();

                                    view! {
                                        <table class="table table-zebra table-xs w-full">
                                            <thead>
                                                <tr class="text-xs uppercase text-base-content/70">
                                                    <th class="w-32">"Datum"</th>
                                                    <th>"Grund"</th>
                                                    <th class="text-right">"Betrag (CHF)"</th>
                                                    <th class="w-20 text-right"></th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {if items.is_empty() {
                                                    view! {
                                                        <tr>
                                                            <td colspan="4" class="text-center text-base-content/50 py-6">
                                                                "Noch keine Schulden erfasst."
                                                            </td>
                                                        </tr>
                                                    }.into_any()
                                                } else {
                                                    items.into_iter().map(|s| {
                                                        let datum_str = s.datum.format("%d.%m.%Y").to_string();
                                                        let betrag_str = format!("CHF {:.2}", s.betrag);
                                                        let id = s.id;
                                                        let bearbeiten = s.clone();
                                                        let zeile_dbl = s.clone();

                                                        view! {
                                                            <tr class="select-none" title="Doppelklick zum Bearbeiten" on:dblclick=move |_| bearbeiten_oeffnen(zeile_dbl.clone())>
                                                                <td class="whitespace-nowrap font-mono text-base-content/80">{datum_str}</td>
                                                                <td class="font-medium">{s.grund}</td>
                                                                <td class="text-right whitespace-nowrap font-mono">{betrag_str}</td>
                                                                <td class="text-right whitespace-nowrap">
                                                                    <button
                                                                        type="button"
                                                                        class="btn btn-ghost"
                                                                        title="Schuld bearbeiten"
                                                                        aria-label="Schuld bearbeiten"
                                                                        on:click=move |_| bearbeiten_oeffnen(bearbeiten.clone())
                                                                    >
                                                                        "✎"
                                                                    </button>
                                                                    <button
                                                                        type="button"
                                                                        class="btn btn-ghost text-error"
                                                                        title="Schuld löschen"
                                                                        on:dblclick=move |ev| { ev.stop_propagation(); }
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
                                                <tr class="font-bold text-xs bg-base-200/50">
                                                    <td colspan="2">"Total Schulden"</td>
                                                    <td class="text-right text-primary whitespace-nowrap font-mono">{format!("CHF {:.2}", summe)}</td>
                                                    <td></td>
                                                </tr>
                                            </tfoot>
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
        </section>
    }
}
