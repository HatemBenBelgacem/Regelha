use chrono::Local;
use leptos::prelude::*;

use crate::backend::budget::{add_buchung, delete_buchung, get_buchungen, BuchungsArt};

#[component]
pub fn BuchungenSection() -> impl IntoView {
    let today_iso = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let (datum_input, set_datum_input) = signal(today_iso);
    let (buchungstext, set_buchungstext) = signal(String::new());
    let (betrag_input, set_betrag_input) = signal(String::new());
    let (art, set_art) = signal(BuchungsArt::Eingang);
    let (filter_art, set_filter_art) = signal(Option::<BuchungsArt>::None);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    let buchungen_resource = Resource::new(|| (), |_| get_buchungen());

    let create_action = Action::new(
        move |(text, betrag, buchungs_art, datum): &(String, f64, BuchungsArt, String)| {
            let text = text.clone();
            let betrag = *betrag;
            let buchungs_art = *buchungs_art;
            let datum = Some(datum.clone());
            async move { add_buchung(text, betrag, buchungs_art, datum).await }
        },
    );

    let delete_action = Action::new(move |id: &uuid::Uuid| {
        let id = *id;
        async move { delete_buchung(id).await }
    });

    Effect::new(move |_| {
        if let Some(res) = create_action.value().get() {
            match res {
                Ok(_) => {
                    set_buchungstext.set(String::new());
                    set_betrag_input.set(String::new());
                    set_error_msg.set(None);
                    buchungen_resource.refetch();
                }
                Err(e) => set_error_msg.set(Some(e.to_string())),
            }
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(())) = delete_action.value().get() {
            buchungen_resource.refetch();
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let text = buchungstext.get();
        let raw_amount = betrag_input.get().replace(',', ".");
        match raw_amount.trim().parse::<f64>() {
            Ok(val) if val > 0.0 => {
                set_error_msg.set(None);
                create_action.dispatch((text, val, art.get(), datum_input.get()));
            }
            _ => {
                set_error_msg.set(Some(
                    "Bitte einen gültigen positiven Betrag in CHF eingeben (z.B. 120.50)."
                        .to_string(),
                ));
            }
        }
    };

    view! {
        <section class="card bg-base-100 shadow-sm border border-base-300">
            <div class="card-body p-4 sm:p-6 space-y-6">
                <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
                    <div>
                        <h2 class="card-title text-lg sm:text-xl">"Ein-/Ausgaben (Buchungstabelle)"</h2>
                        <p class="text-xs sm:text-sm text-base-content/70">
                            "Erfassung und Übersicht deiner persönlichen Zahlungseingänge und Zahlungsausgänge in CHF"
                        </p>
                    </div>

                    // Filter: Alle / Zahlungseingänge / Zahlungsausgänge
                    <div class="join flex flex-wrap">
                        <button
                            type="button"
                            class=move || format!("btn btn-xs sm:btn-sm join-item {}", if filter_art.get().is_none() { "btn-primary" } else { "btn-ghost" })
                            on:click=move |_| set_filter_art.set(None)
                        >
                            "Alle"
                        </button>
                        <button
                            type="button"
                            class=move || format!("btn btn-xs sm:btn-sm join-item {}", if filter_art.get() == Some(BuchungsArt::Eingang) { "btn-success text-success-content" } else { "btn-ghost" })
                            on:click=move |_| set_filter_art.set(Some(BuchungsArt::Eingang))
                        >
                            "Eingänge"
                        </button>
                        <button
                            type="button"
                            class=move || format!("btn btn-xs sm:btn-sm join-item {}", if filter_art.get() == Some(BuchungsArt::Ausgang) { "btn-error text-error-content" } else { "btn-ghost" })
                            on:click=move |_| set_filter_art.set(Some(BuchungsArt::Ausgang))
                        >
                            "Ausgänge"
                        </button>
                    </div>
                </div>

                // Formular zum Erfassen neuer Buchungen (inkl. Datum, Buchungstext, Betrag)
                <form on:submit=on_submit class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-12 gap-3 bg-base-200/60 p-3 sm:p-4 rounded-box items-end">
                    <div class="sm:col-span-1 lg:col-span-3">
                        <label class="label py-1">
                            <span class="label-text font-medium text-xs sm:text-sm">"Buchungsart"</span>
                        </label>
                        <div class="join w-full">
                            <button
                                type="button"
                                class=move || format!("btn btn-sm join-item flex-1 {}", if art.get() == BuchungsArt::Eingang { "btn-success" } else { "btn-outline" })
                                on:click=move |_| set_art.set(BuchungsArt::Eingang)
                            >
                                "+ Eingang"
                            </button>
                            <button
                                type="button"
                                class=move || format!("btn btn-sm join-item flex-1 {}", if art.get() == BuchungsArt::Ausgang { "btn-error" } else { "btn-outline" })
                                on:click=move |_| set_art.set(BuchungsArt::Ausgang)
                            >
                                "- Ausgang"
                            </button>
                        </div>
                    </div>

                    <div class="sm:col-span-1 lg:col-span-2">
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

                    <div class="sm:col-span-2 lg:col-span-3">
                        <label class="label py-1">
                            <span class="label-text font-medium text-xs sm:text-sm">"Buchungstext"</span>
                        </label>
                        <input
                            type="text"
                            placeholder="z.B. Lohn, Einkauf, Rückerstattung..."
                            class="input input-bordered input-sm w-full"
                            prop:value=move || buchungstext.get()
                            on:input=move |ev| set_buchungstext.set(event_target_value(&ev))
                            required
                        />
                    </div>

                    <div class="sm:col-span-1 lg:col-span-2">
                        <label class="label py-1">
                            <span class="label-text font-medium text-xs sm:text-sm">"Betrag (CHF)"</span>
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
                            class="btn btn-primary btn-sm w-full"
                            disabled=move || create_action.pending().get()
                        >
                            "Verbuchen"
                        </button>
                    </div>
                </form>

                {move || error_msg.get().map(|msg| view! {
                    <div class="alert alert-warning py-2 text-sm">
                        <span>{msg}</span>
                    </div>
                })}

                // Buchungstabelle (Spalten: Datum, Buchungstext, Benutzer, Betrag)
                <div class="overflow-x-auto -mx-2 sm:mx-0">
                    <Suspense fallback=move || view! {
                        <div class="flex justify-center p-8">
                            <span class="loading loading-spinner loading-md text-primary"></span>
                        </div>
                    }>
                        {move || {
                            buchungen_resource.get().map(|res| match res {
                                Ok(items) => {
                                    let filtered: Vec<_> = items
                                        .into_iter()
                                        .filter(|b| match filter_art.get() {
                                            Some(target) => b.art == target,
                                            None => true,
                                        })
                                        .collect();

                                    let summe_eingang: f64 = filtered
                                        .iter()
                                        .filter(|b| b.art == BuchungsArt::Eingang)
                                        .map(|b| b.betrag)
                                        .fold(0.0, |acc, x| acc + x);
                                    let summe_ausgang: f64 = filtered
                                        .iter()
                                        .filter(|b| b.art == BuchungsArt::Ausgang)
                                        .map(|b| b.betrag)
                                        .fold(0.0, |acc, x| acc + x);
                                    let saldo = summe_eingang - summe_ausgang;
                                    let saldo_class = if saldo >= 0.0 {
                                        "text-right text-success whitespace-nowrap"
                                    } else {
                                        "text-right text-error whitespace-nowrap"
                                    };

                                    view! {
                                        <table class="table table-zebra table-sm sm:table-md w-full">
                                            <thead>
                                                <tr class="text-xs sm:text-sm uppercase text-base-content/70">
                                                    <th class="w-32">"Datum"</th>
                                                    <th>"Buchungstext"</th>
                                                    <th class="w-40">"Benutzer"</th>
                                                    <th class="text-right">"Betrag (CHF)"</th>
                                                    <th class="w-12 text-right"></th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {if filtered.is_empty() {
                                                    view! {
                                                        <tr>
                                                            <td colspan="5" class="text-center text-base-content/50 py-6">
                                                                "Keine Buchungen vorhanden."
                                                            </td>
                                                        </tr>
                                                    }.into_any()
                                                } else {
                                                    filtered.into_iter().map(|b| {
                                                        let is_eingang = b.art == BuchungsArt::Eingang;
                                                        let badge_class = if is_eingang {
                                                            "badge badge-success badge-xs badge-outline mr-2"
                                                        } else {
                                                            "badge badge-error badge-xs badge-outline mr-2"
                                                        };
                                                        let badge_label = if is_eingang { "Eingang" } else { "Ausgang" };
                                                        let amount_class = if is_eingang {
                                                            "text-right font-semibold text-success whitespace-nowrap"
                                                        } else {
                                                            "text-right font-semibold text-error whitespace-nowrap"
                                                        };
                                                        let formatted_amount = if is_eingang {
                                                            format!("+ CHF {:.2}", b.betrag)
                                                        } else {
                                                            format!("- CHF {:.2}", b.betrag)
                                                        };
                                                        let datum_str = b.erstellt_am.format("%d.%m.%Y").to_string();
                                                        let benutzer_name = b.benutzer;
                                                        let id = b.id;

                                                        view! {
                                                            <tr>
                                                                <td class="whitespace-nowrap text-base-content/80 font-mono text-xs sm:text-sm">
                                                                    {datum_str}
                                                                </td>
                                                                <td class="font-medium">
                                                                    <div class="flex flex-wrap items-center gap-1">
                                                                        <span class=badge_class>{badge_label}</span>
                                                                        <span>{b.buchungstext}</span>
                                                                    </div>
                                                                </td>
                                                                <td>
                                                                    <span class="badge badge-ghost badge-sm font-medium">
                                                                        {benutzer_name}
                                                                    </span>
                                                                </td>
                                                                <td class=amount_class>{formatted_amount}</td>
                                                                <td class="text-right">
                                                                    <button
                                                                        type="button"
                                                                        class="btn btn-ghost btn-xs text-error"
                                                                        title="Eintrag löschen"
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
                                                    <td colspan="3">
                                                        <div class="flex flex-col sm:flex-row sm:gap-2">
                                                            <span>"Saldo"</span>
                                                            <span class="text-xs sm:text-sm font-normal text-base-content/70">
                                                                {format!("(Eingänge: +CHF {:.2} | Ausgänge: -CHF {:.2})", summe_eingang, summe_ausgang)}
                                                            </span>
                                                        </div>
                                                    </td>
                                                    <td class=saldo_class>
                                                        {if saldo >= 0.0 {
                                                            format!("+ CHF {:.2}", saldo)
                                                        } else {
                                                            format!("- CHF {:.2}", saldo.abs())
                                                        }}
                                                    </td>
                                                    <td></td>
                                                </tr>
                                            </tfoot>
                                        </table>
                                    }.into_any()
                                }
                                Err(err) => view! {
                                    <div class="alert alert-error">
                                        <span>{format!("Fehler beim Laden der Buchungen: {err}")}</span>
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
