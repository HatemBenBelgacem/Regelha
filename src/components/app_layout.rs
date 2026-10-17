use leptos::prelude::*;
use leptos_router::nested_router::Outlet;

use crate::components::{header::Header, sideboard::Sideboard};

#[component]
pub fn AppLayout() -> impl IntoView {
    view! {
        <div class="min-h-screen flex flex-col bg-base-200 text-base-content">
            // 1. Header
            <Header/>

            // 2. Sideboard + 3. Content (Responsives Drawer-Layout)
            <div class="drawer lg:drawer-open flex-1">
                <input id="app-sideboard-drawer" type="checkbox" class="drawer-toggle" />

                // Content-Bereich
                <div class="drawer-content flex flex-col p-3 sm:p-6 lg:p-8 overflow-x-hidden">
                    <main class="container mx-auto max-w-6xl w-full space-y-6 sm:space-y-8">
                        <Outlet/>
                    </main>
                </div>

                // Sideboard (Navigationen)
                <div class="drawer-side z-30">
                    <label for="app-sideboard-drawer" aria-label="Navigation schließen" class="drawer-overlay"></label>
                    <Sideboard/>
                </div>
            </div>
        </div>
    }
}
