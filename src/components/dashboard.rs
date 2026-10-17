use leptos::prelude::*;

use crate::backend::budget::get_dashboard_kennzahlen;

#[component]
pub fn BudgetDashboard() -> impl IntoView {
    let kennzahlen_resource = Resource::new(|| (), |_| get_dashboard_kennzahlen());

    view! {
        <div class="space-y-6">
            // Kopfzeile des Dashboards
            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                <div>
                    <h1 class="text-2xl sm:text-3xl font-bold tracking-tight">"Dashboard"</h1>
                    <p class="text-sm text-base-content/70">
                        "Ausgaben-Analyse, Tagesschnitt und Hochrechnung bis Monatsende"
                    </p>
                </div>
                <div class="flex flex-wrap gap-2">
                    <a href="/buchungen" class="btn btn-primary btn-sm">
                        "Ein-/Ausgaben verwalten"
                    </a>
                    <a href="/fixkosten" class="btn btn-outline btn-sm">
                        "Fixkosten verwalten"
                    </a>
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
                                                    {format!("{:.2} €", k.ausgaben_heute)}
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
                                                    {format!("{:.2} €", k.ausgaben_aktueller_monat)}
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
                                                    {format!("{:.2} €", k.durchschnitt_aktueller_monat)}
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
                                                    <span class="badge badge-warning badge-outline badge-sm">
                                                        {format!("{} Tage", k.tage_im_monat)}
                                                    </span>
                                                </div>
                                                <div class="text-2xl sm:text-3xl font-extrabold text-warning mt-2">
                                                    {format!("{:.2} €", k.voraussichtliche_ausgaben_monatsende)}
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
                                                        <div class="text-lg font-bold text-warning">
                                                            {format!("{:.2} €", k.voraussichtliche_gesamtausgaben_inkl_fixkosten)}
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
                                                            {format!("+ {:.2} €", k.einnahmen_aktueller_monat)}
                                                        </div>
                                                    </div>
                                                    <div class="p-3 rounded-box bg-base-200/70">
                                                        <div class="text-xs text-base-content/60">"Fixkosten (Monat)"</div>
                                                        <div class="text-lg sm:text-xl font-bold text-warning">
                                                            {format!("- {:.2} €", k.summe_fixkosten)}
                                                        </div>
                                                    </div>
                                                    <div class="p-3 rounded-box bg-base-200/70">
                                                        <div class="text-xs text-base-content/60">"Erwarteter Saldo"</div>
                                                        <div class=restbudget_class>
                                                            {if restbudget_prognose >= 0.0 {
                                                                format!("+ {:.2} €", restbudget_prognose)
                                                            } else {
                                                                format!("- {:.2} €", restbudget_prognose.abs())
                                                            }}
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
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
