use leptos::prelude::*;
use leptos_router::hooks::use_url;

#[component]
pub fn Sideboard() -> impl IntoView {
    let url = use_url();
    let is_active = move |path: &str| {
        if url.with(|u| u.path() == path) {
            "active font-semibold bg-primary/10 text-primary"
        } else {
            ""
        }
    };

    view! {
        <aside class="w-72 sm:w-64 min-h-full bg-base-100 border-r border-base-300 flex flex-col justify-between p-4">
            <div class="space-y-4">
                // Mobile Header im Drawer mit Schließen-Button
                <div class="flex items-center justify-between px-2 pt-1 lg:hidden border-b border-base-200 pb-3">
                    <span class="font-bold text-lg text-primary">"Budgetplaner"</span>
                    <label
                        for="app-sideboard-drawer"
                        class="btn btn-sm btn-circle btn-ghost"
                        aria-label="Menü schließen"
                    >
                        "✕"
                    </label>
                </div>

                <ul class="menu menu-md w-full gap-1 p-0">
                    // Dashboard (ersetzt "Alle Tabellen")
                    <li>
                        <a href="/" class=move || is_active("/")>
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
                            </svg>
                            "Dashboard"
                        </a>
                    </li>

                    // Gruppierung unter "Finanzen"
                    <li class="mt-2">
                        <details open>
                            <summary class="font-semibold text-xs uppercase tracking-wider text-base-content/60">
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg>
                                "Finanzen"
                            </summary>
                            <ul class="mt-1 space-y-1">
                                <li>
                                    <a href="/buchungen" class=move || is_active("/buchungen")>
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
                                        </svg>
                                        "Ein-/Ausgaben"
                                    </a>
                                </li>
                                <li>
                                    <a href="/fixkosten" class=move || is_active("/fixkosten")>
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
                                        </svg>
                                        "Fixkosten"
                                    </a>
                                </li>
                            </ul>
                        </details>
                    </li>
                </ul>
            </div>

            <div class="p-3 rounded-box bg-base-200 text-xs text-base-content/70 space-y-1">
                <div class="font-semibold text-base-content">"Budgetplaner"</div>
                <p>"Übersicht über Tages-, Monats- und prognostizierte Ausgaben."</p>
            </div>
        </aside>
    }
}
