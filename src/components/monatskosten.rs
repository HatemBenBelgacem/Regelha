use leptos::prelude::*;

use crate::backend::budget::get_monatskosten;

/// Formatiert einen Saldo mit Vorzeichen (z.B. "+ CHF 12.00" / "- CHF 3.50").
fn saldo_text(saldo: f64) -> String {
    if saldo >= 0.0 {
        format!("+ CHF {:.2}", saldo)
    } else {
        format!("- CHF {:.2}", saldo.abs())
    }
}

fn saldo_class(saldo: f64) -> &'static str {
    if saldo >= 0.0 {
        "text-right font-semibold text-success whitespace-nowrap"
    } else {
        "text-right font-semibold text-error whitespace-nowrap"
    }
}

#[component]
pub fn MonatskostenSection() -> impl IntoView {
    let monate_resource = Resource::new(|| (), |_| get_monatskosten());

    view! {
        <section class="card bg-base-100 shadow-sm border border-base-300">
            <div class="card-body p-4 sm:p-6 space-y-6">
                <div>
                    <h2 class="card-title text-lg sm:text-xl">"Kosten pro Monat"</h2>
                    <p class="text-xs sm:text-sm text-base-content/70">
                        "Monatliche Übersicht deiner Einnahmen, Ausgaben und Fixkosten in CHF"
                    </p>
                </div>

                <div class="overflow-x-auto rounded-box border border-base-300">
                    <Suspense fallback=move || view! {
                        <div class="flex justify-center p-8">
                            <span class="loading loading-spinner loading-md text-primary"></span>
                        </div>
                    }>
                        {move || {
                            monate_resource.get().map(|res| match res {
                                Ok(monate) => {
                                    let summe_einnahmen: f64 = monate.iter().map(|m| m.einnahmen).sum();
                                    let summe_ausgaben: f64 = monate.iter().map(|m| m.ausgaben).sum();
                                    let summe_fixkosten: f64 = monate.iter().map(|m| m.fixkosten).sum();
                                    let summe_total: f64 = monate.iter().map(|m| m.total_kosten).sum();
                                    let summe_saldo = summe_einnahmen - summe_total;
                                    let anzahl_monate = monate.len().max(1) as f64;
                                    let durchschnitt_total = summe_total / anzahl_monate;

                                    view! {
                                        <table class="table table-zebra table-sm sm:table-md w-full">
                                            <thead>
                                                <tr class="text-xs sm:text-sm uppercase text-base-content/70">
                                                    <th>"Monat"</th>
                                                    <th class="text-right">"Einnahmen"</th>
                                                    <th class="text-right">"Ausgaben"</th>
                                                    <th class="text-right">"Fixkosten"</th>
                                                    <th class="text-right">"Total Kosten"</th>
                                                    <th class="text-right">"Saldo"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {monate.into_iter().map(|m| {
                                                    view! {
                                                        <tr>
                                                            <td class="whitespace-nowrap font-medium">
                                                                <div class="flex flex-col">
                                                                    <span>{m.monat_name}</span>
                                                                    <span class="text-xs font-normal text-base-content/50">
                                                                        {format!("{} Buchungen", m.anzahl_buchungen)}
                                                                    </span>
                                                                </div>
                                                            </td>
                                                            <td class="text-right text-success whitespace-nowrap">
                                                                {format!("CHF {:.2}", m.einnahmen)}
                                                            </td>
                                                            <td class="text-right text-error whitespace-nowrap">
                                                                {format!("CHF {:.2}", m.ausgaben)}
                                                            </td>
                                                            <td class="text-right text-warning whitespace-nowrap">
                                                                {format!("CHF {:.2}", m.fixkosten)}
                                                            </td>
                                                            <td class="text-right font-bold whitespace-nowrap">
                                                                {format!("CHF {:.2}", m.total_kosten)}
                                                            </td>
                                                            <td class=saldo_class(m.saldo)>{saldo_text(m.saldo)}</td>
                                                        </tr>
                                                    }
                                                }).collect_view()}
                                            </tbody>
                                            <tfoot>
                                                <tr class="font-bold text-sm sm:text-base bg-base-200/50">
                                                    <td>
                                                        <div class="flex flex-col">
                                                            <span>"Total"</span>
                                                            <span class="text-xs font-normal text-base-content/70">
                                                                {format!("Ø Kosten: CHF {:.2} / Monat", durchschnitt_total)}
                                                            </span>
                                                        </div>
                                                    </td>
                                                    <td class="text-right text-success whitespace-nowrap">
                                                        {format!("CHF {:.2}", summe_einnahmen)}
                                                    </td>
                                                    <td class="text-right text-error whitespace-nowrap">
                                                        {format!("CHF {:.2}", summe_ausgaben)}
                                                    </td>
                                                    <td class="text-right text-warning whitespace-nowrap">
                                                        {format!("CHF {:.2}", summe_fixkosten)}
                                                    </td>
                                                    <td class="text-right whitespace-nowrap">
                                                        {format!("CHF {:.2}", summe_total)}
                                                    </td>
                                                    <td class=saldo_class(summe_saldo)>{saldo_text(summe_saldo)}</td>
                                                </tr>
                                            </tfoot>
                                        </table>
                                    }.into_any()
                                }
                                Err(err) => view! {
                                    <div class="alert alert-error">
                                        <span>{format!("Fehler beim Laden der Monatsübersicht: {err}")}</span>
                                    </div>
                                }.into_any(),
                            })
                        }}
                    </Suspense>
                </div>

                <p class="text-xs text-base-content/60">
                    "Total Kosten = Ausgaben + Fixkosten. Fixkosten zählen ab ihrem Beginnmonat, solange sie nicht beendet sind. Vierteljährliche, halbjährliche und jährliche Zahlungen werden auf den Monat umgerechnet."
                </p>
            </div>
        </section>
    }
}
