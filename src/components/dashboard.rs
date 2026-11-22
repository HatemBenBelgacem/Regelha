use leptos::prelude::*;

use crate::backend::budget::{get_dashboard_kennzahlen, KategorieSumme};

#[component]
pub fn BudgetDashboard() -> impl IntoView {
    let kennzahlen_resource = Resource::new(|| (), |_| get_dashboard_kennzahlen());

    view! {
        <div class="space-y-6">
            // Kopfzeile des Dashboards
            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
                <div>
                    <h1 class="text-2xl sm:text-3xl font-bold tracking-tight">"Dashboard"</h1>
                    <p class="text-sm text-base-content/70">
                        "Ausgaben-Analyse, Tagesschnitt und Hochrechnung bis Monatsende (in CHF)"
                    </p>
                </div>
            </div>

            <Suspense fallback=move || view! {
                <div class="flex justify-center p-12">
                    <span class="loading loading-spinner loading-lg text-primary"></span>
                </div>
            }>
                {move || {
                    kennzahlen_resource.get().map(|res| match res {
                        Ok(k) => {
                            let fortschritt_prozent = ((k.tag_im_monat as f64) / (k.tage_im_monat as f64) * 100.0).round() as u32;
                            let restbudget_prognose = k.einnahmen_aktueller_monat - k.voraussichtliche_gesamtausgaben_inkl_fixkosten;
                            let restbudget_class = if restbudget_prognose >= 0.0 {
                                "text-xl sm:text-2xl font-bold text-success"
                            } else {
                                "text-xl sm:text-2xl font-bold text-error"
                            };

                            view! {
                                <div class="space-y-6">
                                    // 4 Haupt-KPI-Karten (Responsives Grid: 1 Spalte Mobile, 2 Tablet, 4 Desktop)
                                    <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-4">
                                        // 1. Ausgaben des aktuellen Tages
                                        <div class="card bg-base-100 shadow-sm border border-base-300">
                                            <div class="card-body p-4 sm:p-5">
                                                <div class="flex items-center justify-between">
                                                    <span class="text-xs font-semibold uppercase tracking-wider text-base-content/60">
                                                        "Ausgaben heute"
                                                    </span>
                                                    <span class="badge badge-ghost badge-sm">{k.heute_datum.clone()}</span>
                                                </div>
                                                <div class="text-2xl sm:text-3xl font-extrabold text-error mt-2">
                                                    {format!("CHF {:.2}", k.ausgaben_heute)}
                                                </div>
                                                <p class="text-xs text-base-content/60 mt-1">
                                                    "Summe der Ausgaben des aktuellen Tages"
                                                </p>
                                            </div>
                                        </div>

                                        // 2. Ausgaben des aktuellen Monats
                                        <div class="card bg-base-100 shadow-sm border border-base-300">
                                            <div class="card-body p-4 sm:p-5">
                                                <div class="flex items-center justify-between">
                                                    <span class="text-xs font-semibold uppercase tracking-wider text-base-content/60">
                                                        "Ausgaben Monat"
                                                    </span>
                                                    <span class="badge badge-primary badge-outline badge-sm">{k.monat_name.clone()}</span>
                                                </div>
                                                <div class="text-2xl sm:text-3xl font-extrabold text-primary mt-2">
                                                    {format!("CHF {:.2}", k.ausgaben_aktueller_monat)}
                                                </div>
                                                <p class="text-xs text-base-content/60 mt-1">
                                                    {format!("Bisherige Ausgaben (Tag 1 bis {})", k.tag_im_monat)}
                                                </p>
                                            </div>
                                        </div>

                                        // 3. Durchschnittliche Ausgaben des aktuellen Monats (Total Ausgaben / Anzahl Tage bis heute)
                                        <div class="card bg-base-100 shadow-sm border border-base-300">
                                            <div class="card-body p-4 sm:p-5">
                                                <div class="flex items-center justify-between">
                                                    <span class="text-xs font-semibold uppercase tracking-wider text-base-content/60">
                                                        "Ø Ausgaben (Monat)"
                                                    </span>
                                                    <span class="badge badge-secondary badge-outline badge-sm">
                                                        {format!("÷ {} Tage", k.tag_im_monat)}
                                                    </span>
                                                </div>
                                                <div class="text-2xl sm:text-3xl font-extrabold text-secondary mt-2">
                                                    {format!("CHF {:.2}", k.durchschnitt_aktueller_monat)}
                                                </div>
                                                <p class="text-xs text-base-content/60 mt-1">
                                                    "Total Ausgaben ÷ Anzahl Tage bis heute"
                                                </p>
                                            </div>
                                        </div>

                                        // 4. Voraussichtliche Ausgaben bis Ende Monat
                                        <div class="card bg-base-100 shadow-sm border border-base-300">
                                            <div class="card-body p-4 sm:p-5">
                                                <div class="flex items-center justify-between">
                                                    <span class="text-xs font-semibold uppercase tracking-wider text-base-content/60">
                                                        "Prognose Monatsende"
                                                    </span>
                                                    <span class="badge badge-warning badge-outline badge-sm text-black">
                                                        {format!("{} Tage", k.tage_im_monat)}
                                                    </span>
                                                </div>
                                                <div class="text-2xl sm:text-3xl font-extrabold text-black mt-2">
                                                    {format!("CHF {:.2}", k.voraussichtliche_ausgaben_monatsende)}
                                                </div>
                                                <p class="text-xs text-base-content/60 mt-1">
                                                    {format!("Voraussichtliche Ausgaben bis Ende {}", k.monat_name)}
                                                </p>
                                            </div>
                                        </div>
                                    </div>

                                    // Zweite Reihe: Monatsfortschritt & Gesamt-Finanzüberblick (inkl. Fixkosten & Einnahmen)
                                    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
                                        <div class="card bg-base-100 shadow-sm border border-base-300">
                                            <div class="card-body p-4 sm:p-6 space-y-4">
                                                <div class="flex items-center justify-between">
                                                    <h2 class="card-title text-base sm:text-lg">"Monatsfortschritt & Hochrechnung"</h2>
                                                    <span class="text-xs sm:text-sm font-medium text-base-content/70">
                                                        {format!("Tag {} von {} ({}%)", k.tag_im_monat, k.tage_im_monat, fortschritt_prozent)}
                                                    </span>
                                                </div>
                                                <progress
                                                    class="progress progress-primary w-full"
                                                    value=k.tag_im_monat
                                                    max=k.tage_im_monat
                                                ></progress>
                                                <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-2 text-sm">
                                                    <div class="p-3 rounded-box bg-base-200/70">
                                                        <div class="text-xs text-base-content/60">"Verbleibende Tage im Monat"</div>
                                                        <div class="text-lg font-bold">
                                                            {format!("{} Tage", k.tage_im_monat.saturating_sub(k.tag_im_monat))}
                                                        </div>
                                                    </div>
                                                    <div class="p-3 rounded-box bg-base-200/70">
                                                        <div class="text-xs text-base-content/60">"Prognose inkl. Fixkosten"</div>
                                                        <div class="text-lg font-bold text-black">
                                                            {format!("CHF {:.2}", k.voraussichtliche_gesamtausgaben_inkl_fixkosten)}
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        <div class="card bg-base-100 shadow-sm border border-base-300">
                                            <div class="card-body p-4 sm:p-6 space-y-4">
                                                <h2 class="card-title text-base sm:text-lg">"Monatsbilanz (Finanzen)"</h2>
                                                <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
                                                    <div class="p-3 rounded-box bg-base-200/70">
                                                        <div class="text-xs text-base-content/60">"Einnahmen (Monat)"</div>
                                                        <div class="text-lg sm:text-xl font-bold text-success">
                                                            {format!("+ CHF {:.2}", k.einnahmen_aktueller_monat)}
                                                        </div>
                                                    </div>
                                                    <div class="p-3 rounded-box bg-base-200/70">
                                                        <div class="text-xs text-base-content/60">"Fixkosten (Monat)"</div>
                                                        <div class="text-lg sm:text-xl font-bold text-black">
                                                            {format!("- CHF {:.2}", k.summe_fixkosten)}
                                                        </div>
                                                    </div>
                                                    <div class="p-3 rounded-box bg-base-200/70">
                                                        <div class="text-xs text-base-content/60">"Erwarteter Saldo"</div>
                                                        <div class=restbudget_class>
                                                            {if restbudget_prognose >= 0.0 {
                                                                format!("+ CHF {:.2}", restbudget_prognose)
                                                            } else {
                                                                format!("- CHF {:.2}", restbudget_prognose.abs())
                                                            }}
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    // Dritte Reihe: Fixkosten je Kategorie als Säulendiagramm
                                    <div class="card bg-base-100 shadow-sm border border-base-300">
                                        <div class="card-body p-4 sm:p-6">
                                            {kategorie_saeulen(k.fixkosten_pro_kategorie.clone(), k.monat_name.clone())}
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        }
                        Err(err) => view! {
                            <div class="alert alert-error">
                                <span>{format!("Fehler beim Laden des Dashboards: {err}")}</span>
                            </div>
                        }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}

/// Obere Grenze der Y-Achse: die nächste glatte Zahl über dem größten Wert (z.B. 980 -> 1000, 8,5 -> 10).
fn achsen_obergrenze(max: f64) -> f64 {
    if max <= 0.0 {
        return 1.0;
    }
    let groessenordnung = 10f64.powf(max.log10().floor());
    [1.0, 2.0, 2.5, 5.0, 10.0]
        .into_iter()
        .map(|faktor| faktor * groessenordnung)
        .find(|kandidat| *kandidat >= max)
        .unwrap_or(max)
}

/// Ganze Zahl mit Apostroph als Tausendertrennzeichen (Schweizer Schreibweise), z.B. 1'000.
fn ganzzahl_text(wert: f64) -> String {
    let ziffern = (wert.round() as i64).abs().to_string();
    let mut text = String::new();
    for (i, c) in ziffern.chars().enumerate() {
        if i > 0 && (ziffern.len() - i).is_multiple_of(3) {
            text.push('\'');
        }
        text.push(c);
    }
    text
}

/// Säulendiagramm: Fixkosten je Kategorie im Monat, Skala ab 0.
/// Die Tabelle darunter enthält dieselben Werte. Der Tooltip erscheint bei Maus und Tastatur.
fn kategorie_saeulen(summen: Vec<KategorieSumme>, monat_name: String) -> impl IntoView {
    if summen.is_empty() {
        return view! {
            <div>
                <h3 class="card-title text-base sm:text-lg">"Fixkosten nach Kategorie"</h3>
                <p class="text-sm text-base-content/60 py-6 text-center">"Keine Fixkosten in diesem Monat."</p>
            </div>
        }
        .into_any();
    }

    let max = summen.iter().map(|s| s.betrag).fold(0.0, f64::max);
    let obergrenze = achsen_obergrenze(max);
    let ticks: Vec<f64> = (0..=4).map(|i| obergrenze * i as f64 / 4.0).collect();
    let gesamt: f64 = summen.iter().map(|s| s.betrag).sum();
    // Jede Kategorie braucht genug Breite für ihren Namen, sonst scrollt die Karte waagerecht
    let mindestbreite = format!("min-width: {}px", summen.len() * 96 + 56);
    let tabelle = summen.clone();
    // Position in Prozent der Plotfläche (0 = Basislinie, 100 = Oberkante der Skala)
    let position = |betrag: f64| betrag / obergrenze * 100.0;

    view! {
        // <div role="figure"> statt <figure>: DaisyUI formatiert figure in .card als Flex-Zeile (für Bilder)
        <div role="figure" class="space-y-3">
            <div>
                <h3 class="card-title text-base sm:text-lg">"Fixkosten nach Kategorie"</h3>
                <p class="text-xs sm:text-sm text-base-content/70">
                    {format!("Monatsbetrag je Kategorie im {monat_name} in CHF. Fixeinnahmen sind nicht enthalten.")}
                </p>
            </div>

            // Oben Platz für den Tooltip der höchsten Säule (overflow-x-auto schneidet sonst auch senkrecht ab)
            <div class="overflow-x-auto pt-14 -mt-8">
                <div class="space-y-2" style=mindestbreite>
                    // Plot: Y-Achse mit Werten, Gitterlinien und Säulen
                    <div class="flex gap-2">
                        <div class="relative w-12 h-56 sm:h-64 shrink-0">
                            {ticks.iter().map(|t| {
                                let pos = position(*t);
                                view! {
                                    <span
                                        class="absolute right-0 translate-y-1/2 text-xs text-base-content/60 tabular-nums"
                                        style=format!("bottom: {pos}%")
                                    >
                                        {ganzzahl_text(*t)}
                                    </span>
                                }
                            }).collect_view()}
                        </div>

                        <div class="relative flex-1 h-56 sm:h-64">
                            {ticks.iter().map(|t| {
                                let pos = position(*t);
                                let klasse = if *t == 0.0 {
                                    "absolute inset-x-0 border-t border-slate-300"
                                } else {
                                    "absolute inset-x-0 border-t border-base-300"
                                };
                                view! { <div class=klasse style=format!("bottom: {pos}%")></div> }
                            }).collect_view()}

                            <div class="absolute inset-0 flex gap-3 px-1">
                                {summen.iter().enumerate().map(|(i, s)| {
                                    let pos = position(s.betrag);
                                    let wert = format!("CHF {:.2}", s.betrag);
                                    let beschriftung = format!("{}: {}", s.name, wert);
                                    let name = s.name.clone();
                                    let betrag_text = format!("{:.2}", s.betrag);
                                    view! {
                                        // Die ganze Spalte ist der Zeigebereich, nicht nur die 24px-Säule
                                        <div tabindex="0" role="img" aria-label=beschriftung class="group relative flex-1 h-full">
                                            <div
                                                class="absolute bottom-0 left-1/2 -translate-x-1/2 w-6 rounded-t bg-regelha-diagramm transition-opacity group-hover:opacity-80 group-focus-visible:opacity-80"
                                                style=format!("height: {pos}%")
                                            ></div>
                                            // Wert nur an der größten Säule, die übrigen stehen im Tooltip und in der Tabelle
                                            {(i == 0).then(|| view! {
                                                <span
                                                    class="absolute left-1/2 -translate-x-1/2 text-xs font-semibold tabular-nums whitespace-nowrap"
                                                    style=format!("bottom: calc({pos}% + 4px)")
                                                >
                                                    {betrag_text}
                                                </span>
                                            })}
                                            <div
                                                class="pointer-events-none invisible group-hover:visible group-focus-visible:visible absolute left-1/2 -translate-x-1/2 z-10 whitespace-nowrap rounded-md border border-base-300 bg-base-100 px-2 py-1 text-xs shadow-md"
                                                style=format!("bottom: calc({pos}% + 8px)")
                                            >
                                                <div class="text-base-content/70">{name}</div>
                                                <div class="font-bold tabular-nums">{wert}</div>
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    </div>

                    // Kategorienamen unter den Säulen (gleiche Spaltenaufteilung wie die Plotfläche)
                    <div class="flex gap-2">
                        <div class="w-12 shrink-0"></div>
                        <div class="flex-1 flex gap-3 px-1">
                            {summen.iter().map(|s| view! {
                                <div class="flex-1 min-w-0 text-center text-xs text-base-content/80 hyphens-auto break-words">{s.name.clone()}</div>
                            }).collect_view()}
                        </div>
                    </div>
                </div>
            </div>

            <details class="text-sm">
                <summary class="cursor-pointer text-xs font-medium text-primary">"Als Tabelle anzeigen"</summary>
                <table class="table table-xs w-full mt-2">
                    <thead>
                        <tr class="text-xs uppercase text-base-content/70">
                            <th>"Kategorie"</th>
                            <th class="text-right">"Monatsbetrag (CHF)"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {tabelle.into_iter().map(|s| view! {
                            <tr>
                                <td class="font-medium">{s.name}</td>
                                <td class="text-right tabular-nums">{format!("{:.2}", s.betrag)}</td>
                            </tr>
                        }).collect_view()}
                    </tbody>
                    <tfoot>
                        <tr class="font-bold text-xs bg-base-200/50">
                            <td>"Summe"</td>
                            <td class="text-right tabular-nums">{format!("{:.2}", gesamt)}</td>
                        </tr>
                    </tfoot>
                </table>
            </details>
        </div>
    }
    .into_any()
}
