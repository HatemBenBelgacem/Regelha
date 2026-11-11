use leptos::prelude::*;

use crate::backend::auth::AuthUser;

#[component]
pub fn Header(user: AuthUser, on_logout: Callback<()>) -> impl IntoView {
    view! {
        <header class="navbar min-h-14 bg-[#66023C] text-white border-b border-[#7A2048] px-3 sm:px-6 sticky top-0 z-20 shadow-sm">
            // Links: Mobile-Menü-Button, App-Titel & Währungs-Badge
            <div class="flex-1 flex items-center gap-2 sm:gap-3">
                <label
                    for="app-sideboard-drawer"
                    class="btn btn-square btn-ghost text-white hover:bg-[#7A2048] lg:hidden"
                    title="Navigation öffnen"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="inline-block w-5 h-5 stroke-current">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path>
                    </svg>
                </label>
                <a href="/" class="text-lg sm:text-xl font-bold tracking-tight text-white">"Regelha"</a>
                <span class="badge badge-xs sm:badge-sm bg-[#7A2048] text-white border-white/30">
                    "CHF"
                </span>
            </div>

            // Rechts: Angemeldeter Benutzer & Abmelden-Button
            <div class="flex-none flex items-center gap-2 sm:gap-3">
                <div class="hidden sm:flex items-center gap-2 px-3 py-1 rounded-full bg-[#7A2048] text-white text-xs font-medium border border-white/15">
                    <span class="w-2 h-2 rounded-full bg-emerald-400"></span>
                    <span>{user.benutzername}</span>
                </div>
                <button
                    type="button"
                    class="btn btn-square bg-white/10 hover:bg-white/20 text-white border-white/25"
                    title="Abmelden"
                    aria-label="Abmelden"
                    on:click=move |_| on_logout.run(())
                >
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="inline-block w-5 h-5 stroke-current">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.75 9V5.25A2.25 2.25 0 0 0 13.5 3h-6a2.25 2.25 0 0 0-2.25 2.25v13.5A2.25 2.25 0 0 0 7.5 21h6a2.25 2.25 0 0 0 2.25-2.25V15M12 9l-3 3m0 0 3 3m-3-3h12.75"></path>
                    </svg>
                </button>
            </div>
        </header>
    }
}
