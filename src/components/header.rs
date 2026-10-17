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
                    class="btn btn-square btn-ghost btn-sm text-white hover:bg-[#7A2048] lg:hidden"
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
                    class="btn btn-xs sm:btn-sm bg-white/10 hover:bg-white/20 text-white border-white/25"
                    on:click=move |_| on_logout.run(())
                >
                    "Abmelden"
                </button>
            </div>
        </header>
    }
}
