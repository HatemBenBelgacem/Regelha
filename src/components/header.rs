use leptos::prelude::*;

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="navbar bg-base-100 border-b border-base-300 px-3 sm:px-6 sticky top-0 z-20 shadow-xs">
            <div class="flex-1 flex items-center gap-2 sm:gap-3">
                <label
                    for="app-sideboard-drawer"
                    class="btn btn-square btn-ghost btn-sm sm:btn-md lg:hidden"
                    title="Navigation öffnen"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="inline-block w-5 h-5 sm:w-6 sm:h-6 stroke-current">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path>
                    </svg>
                </label>
                <a href="/" class="text-lg sm:text-xl font-bold tracking-tight text-primary">"Budgetplaner"</a>
                <span class="badge badge-primary badge-sm badge-outline hidden sm:inline-flex">
                    "Fullstack Leptos"
                </span>
            </div>
            <div class="flex-none flex items-center gap-2">
                <a href="/buchungen" class="btn btn-primary btn-xs sm:btn-sm">
                    "+ Buchung"
                </a>
            </div>
        </header>
    }
}
