use leptos::prelude::*;
use leptos_router::nested_router::Outlet;

use crate::{
    backend::auth::{get_current_user, logout_user, AuthUser},
    components::{header::Header, login::LoginMaske, sideboard::Sideboard},
};

#[component]
pub fn AppLayout() -> impl IntoView {
    let user_resource = Resource::new(|| (), |_| get_current_user());
    let (current_user_override, set_current_user_override) = signal(Option::<Option<AuthUser>>::None);

    let logout_action = Action::new(move |_: &()| async move { logout_user().await });

    Effect::new(move |_| {
        if let Some(Ok(())) = logout_action.value().get() {
            set_current_user_override.set(Some(None));
        }
    });

    let on_authenticated = Callback::new(move |user: AuthUser| {
        set_current_user_override.set(Some(Some(user)));
    });

    let on_logout = Callback::new(move |_: ()| {
        logout_action.dispatch(());
    });

    view! {
        <Suspense fallback=move || view! {
            <div class="min-h-screen flex items-center justify-center bg-[#F8FAFC]">
                <span class="loading loading-spinner loading-lg text-primary"></span>
            </div>
        }>
            {move || {
                let effective_user = match current_user_override.get() {
                    Some(override_val) => Some(override_val),
                    None => user_resource.get().map(|res| res.ok().flatten()),
                };

                effective_user.map(|user_opt| match user_opt {
                    Some(user) => view! {
                        <div class="min-h-screen flex flex-col bg-[#F8FAFC] text-base-content">
                            // 1. Header inkl. Menüband
                            <Header user=user on_logout=on_logout/>

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
                    }.into_any(),
                    None => view! {
                        <LoginMaske on_authenticated=on_authenticated/>
                    }.into_any(),
                })
            }}
        </Suspense>
    }
}
