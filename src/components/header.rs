use leptos::prelude::*;
use leptos_router::hooks::use_url;

use crate::backend::auth::AuthUser;

#[component]
pub fn Header(user: AuthUser, on_logout: Callback<()>) -> impl IntoView {
    let url = use_url();
    let ribbon_btn_class = move |path: &str| {
        if url.with(|u| u.path() == path) {
            "btn btn-xs sm:btn-sm btn-primary shadow-xs"
        } else {
            "btn btn-xs sm:btn-sm btn-ghost text-base-content/80 hover:text-base-content"
        }
    };

    view! {
        <header class="bg-base-100 border-b border-base-300 sticky top-0 z-20 shadow-xs">
            // Obere Zeile: Markenname, Währungs-Badge & Benutzer-Aktionen (ohne "+ Buchung")
            <div class="navbar min-h-14 px-3 sm:px-6">
                <div class="flex-1 flex items-center gap-2 sm:gap-3">
                    <label
                        for="app-sideboard-drawer"
                        class="btn btn-square btn-ghost btn-sm lg:hidden"
                        title="Navigation öffnen"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="inline-block w-5 h-5 stroke-current">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path>
                        </svg>
                    </label>
                    <a href="/" class="text-lg sm:text-xl font-bold tracking-tight text-primary">"Budgetplaner"</a>
                    <span class="badge badge-primary badge-xs sm:badge-sm badge-outline">
                        "CHF"
                    </span>
                </div>

                // Rechts: Angemeldeter Benutzer & Abmelden-Button
                <div class="flex-none flex items-center gap-2 sm:gap-3">
                    <div class="hidden sm:flex items-center gap-2 px-2.5 py-1 rounded-full bg-base-200 text-xs font-medium">
                        <span class="w-2 h-2 rounded-full bg-success"></span>
                        <span>{user.benutzername}</span>
                    </div>
                    <button
                        type="button"
                        class="btn btn-outline btn-error btn-xs sm:btn-sm"
                        on:click=move |_| on_logout.run(())
                    >
                        "Abmelden"
                    </button>
                </div>
            </div>

            // Sauberes Menüband (Ribbon) für schnellen Zugriff auf alle Funktionen
            <div class="bg-base-200/60 border-t border-base-200 px-3 sm:px-6 py-1.5 overflow-x-auto">
                <div class="flex items-center gap-2 sm:gap-4 min-w-max">
                    // Gruppe: Übersicht
                    <div class="flex items-center gap-1.5">
                        <span class="text-[11px] font-semibold uppercase tracking-wider text-base-content/50 px-1">
                            "Übersicht:"
                        </span>
                        <a href="/" class=move || ribbon_btn_class("/")>
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
                            </svg>
                            "Dashboard"
                        </a>
                    </div>

                    <div class="divider divider-horizontal mx-0 my-1"></div>

                    // Gruppe: Finanzen
                    <div class="flex items-center gap-1.5">
                        <span class="text-[11px] font-semibold uppercase tracking-wider text-base-content/50 px-1">
                            "Finanzen:"
                        </span>
                        <a href="/buchungen" class=move || ribbon_btn_class("/buchungen")>
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
                            </svg>
                            "Ein-/Ausgaben"
                        </a>
                        <a href="/fixkosten" class=move || ribbon_btn_class("/fixkosten")>
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
                            </svg>
                            "Fixkosten"
                        </a>
                    </div>
                </div>
            </div>
        </header>
    }
}
