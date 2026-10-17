use leptos::prelude::*;
use leptos_router::hooks::use_url;

#[component]
pub fn Sideboard() -> impl IntoView {
    let url = use_url();
    // Nur das Icon und der Text des aktiven Menüpunkts werden lila (#66023C / #7A2048) eingefärbt
    let nav_link_class = move |path: &str| {
        if url.with(|u| u.path() == path) {
            "flex items-center gap-3 px-3 py-2 rounded-lg font-semibold text-[#66023C] bg-transparent hover:bg-slate-100/70 transition-colors"
        } else {
            "flex items-center gap-3 px-3 py-2 rounded-lg font-normal text-slate-600 bg-transparent hover:text-[#7A2048] hover:bg-slate-100/60 transition-colors"
        }
    };

    let nav_icon_class = move |path: &str| {
        if url.with(|u| u.path() == path) {
            "h-5 w-5 text-[#66023C]"
        } else {
            "h-5 w-5 text-slate-400"
        }
    };

    view! {
        <aside class="w-72 sm:w-64 min-h-full bg-white border-r border-slate-200 flex flex-col justify-between p-4">
            <div class="space-y-4">
                // Mobile Header im Drawer mit Schließen-Button
                <div class="flex items-center justify-between px-2 pt-1 lg:hidden border-b border-slate-100 pb-3">
                    <span class="font-bold text-lg text-[#66023C]">"Regelha"</span>
                    <label
                        for="app-sideboard-drawer"
                        class="btn btn-sm btn-circle btn-ghost"
                        aria-label="Menü schließen"
                    >
                        "✕"
                    </label>
                </div>

                <ul class="menu menu-md w-full gap-1 p-0">
                    // Gruppierung unter "Finanzen" (Dashboard, Ein-/Ausgaben, Fixkosten)
                    <li>
                        <details open>
                            <summary class="font-semibold text-xs uppercase tracking-wider text-slate-500 hover:bg-transparent">
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-slate-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg>
                                "Finanzen"
                            </summary>
                            <ul class="mt-1 space-y-1">
                                <li>
                                    <a href="/" class=move || nav_link_class("/")>
                                        <svg xmlns="http://www.w3.org/2000/svg" class=move || nav_icon_class("/") fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
                                        </svg>
                                        <span>"Dashboard"</span>
                                    </a>
                                </li>
                                <li>
                                    <a href="/buchungen" class=move || nav_link_class("/buchungen")>
                                        <svg xmlns="http://www.w3.org/2000/svg" class=move || nav_icon_class("/buchungen") fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
                                        </svg>
                                        <span>"Ein-/Ausgaben"</span>
                                    </a>
                                </li>
                                <li>
                                    <a href="/fixkosten" class=move || nav_link_class("/fixkosten")>
                                        <svg xmlns="http://www.w3.org/2000/svg" class=move || nav_icon_class("/fixkosten") fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
                                        </svg>
                                        <span>"Fixkosten"</span>
                                    </a>
                                </li>
                            </ul>
                        </details>
                    </li>

                    // Aufgaben
                    <li class="mt-2">
                        <a href="/aufgaben" class=move || nav_link_class("/aufgaben")>
                            <svg xmlns="http://www.w3.org/2000/svg" class=move || nav_icon_class("/aufgaben") fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
                            </svg>
                            <span>"Aufgaben"</span>
                        </a>
                    </li>

                    // Chat
                    <li class="mt-1">
                        <a href="/chat" class=move || nav_link_class("/chat")>
                            <svg xmlns="http://www.w3.org/2000/svg" class=move || nav_icon_class("/chat") fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 10h.01M12 10h.01M16 10h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
                            </svg>
                            <span>"Chat"</span>
                        </a>
                    </li>
                </ul>
            </div>

            <div class="p-3 rounded-box bg-[#F8FAFC] border border-slate-200/80 text-xs text-slate-600 space-y-1">
                <div class="font-semibold text-[#66023C]">"Regelha"</div>
                <p>"Übersicht über Tages-, Monats- und prognostizierte Ausgaben."</p>
            </div>
        </aside>
    }
}
