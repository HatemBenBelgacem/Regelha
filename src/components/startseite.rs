use leptos::prelude::*;

/// Ein Eintrag auf der Startseite: Pfad, Name, Beschreibung und SVG-Pfad des Icons.
struct Modul {
    pfad: &'static str,
    name: &'static str,
    beschreibung: &'static str,
    icon: &'static str,
}

const MODULE: &[Modul] = &[
    Modul {
        pfad: "/dashboard",
        name: "Dashboard",
        beschreibung: "Übersicht über Tages-, Monats- und prognostizierte Ausgaben",
        icon: "M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6",
    },
    Modul {
        pfad: "/buchungen",
        name: "Ein-/Ausgaben",
        beschreibung: "Einnahmen und Ausgaben erfassen",
        icon: "M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4",
    },
    Modul {
        pfad: "/fixkosten",
        name: "Fixkosten",
        beschreibung: "Wiederkehrende Kosten verwalten",
        icon: "M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2",
    },
    Modul {
        pfad: "/monatskosten",
        name: "Kosten pro Monat",
        beschreibung: "Kosten nach Monaten aufgeteilt",
        icon: "M3 10h18M3 14h18M10 3v18M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z",
    },
    Modul {
        pfad: "/schulden",
        name: "Schulden",
        beschreibung: "Schulden und Rückzahlungen pro Kontakt",
        icon: "M9 14l6-6m-5.5.5h.01m4.99 5h.01M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16l3.5-2 3.5 2 3.5-2 3.5 2zM10 8.5a.5.5 0 11-1 0 .5.5 0 011 0zm5 5a.5.5 0 11-1 0 .5.5 0 011 0z",
    },
    Modul {
        pfad: "/aufgaben",
        name: "Aufgaben",
        beschreibung: "Persönliche Aufgaben mit Datum und Status",
        icon: "M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4",
    },
    Modul {
        pfad: "/fasten",
        name: "Fasten",
        beschreibung: "Fastenzeiten erfassen und Dauer berechnen",
        icon: "M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z",
    },
    Modul {
        pfad: "/kontakte",
        name: "Kontakte",
        beschreibung: "Personen mit Vor- und Nachname",
        icon: "M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z",
    },
    Modul {
        pfad: "/chat",
        name: "Chat",
        beschreibung: "Direkt- und Gruppenchats",
        icon: "M8 10h.01M12 10h.01M16 10h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z",
    },
];

/// Startseite: listet alle Module als Kacheln auf. Wird beim Start der App unter "/" angezeigt.
#[component]
pub fn Startseite() -> impl IntoView {
    view! {
        <section class="card bg-base-100 shadow-sm border border-base-300">
            <div class="card-body p-4 sm:p-6 space-y-6">
                <div>
                    <h2 class="card-title text-lg sm:text-xl">"Module"</h2>
                    <p class="text-xs sm:text-sm text-base-content/70">
                        "Wähle ein Modul, um loszulegen"
                    </p>
                </div>

                <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-3 sm:gap-4">
                    {MODULE
                        .iter()
                        .map(|m| {
                            view! {
                                <a
                                    href=m.pfad
                                    class="flex items-center gap-4 p-4 rounded-box border border-slate-200 bg-white hover:border-[#66023C] hover:bg-slate-50 transition-colors"
                                >
                                    <div class="flex-none flex items-center justify-center w-12 h-12 rounded-xl bg-[#66023C]/10">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-[#66023C]" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d=m.icon />
                                        </svg>
                                    </div>
                                    <div class="min-w-0">
                                        <div class="font-semibold text-slate-800">{m.name}</div>
                                        <div class="text-xs sm:text-sm text-slate-500">{m.beschreibung}</div>
                                    </div>
                                </a>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
        </section>
    }
}
