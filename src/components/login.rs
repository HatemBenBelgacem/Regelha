use leptos::prelude::*;

use crate::backend::auth::{login_user, register_user, AuthUser};

#[component]
pub fn LoginMaske(on_authenticated: Callback<AuthUser>) -> impl IntoView {
    let (is_register_mode, set_is_register_mode) = signal(false);
    let (benutzername, set_benutzername) = signal(String::new());
    let (passwort, set_passwort) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    let auth_action = Action::new(move |(reg, user, pw): &(bool, String, String)| {
        let is_reg = *reg;
        let user = user.clone();
        let pw = pw.clone();
        async move {
            if is_reg {
                register_user(user, pw).await
            } else {
                login_user(user, pw).await
            }
        }
    });

    Effect::new(move |_| {
        if let Some(res) = auth_action.value().get() {
            match res {
                Ok(user) => {
                    set_error_msg.set(None);
                    on_authenticated.run(user);
                }
                Err(e) => {
                    let msg = e
                        .to_string()
                        .replace("error running server function: ", "");
                    set_error_msg.set(Some(msg));
                }
            }
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error_msg.set(None);
        auth_action.dispatch((
            is_register_mode.get(),
            benutzername.get(),
            passwort.get(),
        ));
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-base-200 p-4">
            <div class="card w-full max-w-md bg-base-100 shadow-lg border border-base-300">
                <div class="card-body p-6 sm:p-8 space-y-5">
                    <div class="text-center space-y-1">
                        <div class="inline-flex items-center justify-center w-12 h-12 rounded-2xl bg-primary/10 text-primary mb-2">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                            </svg>
                        </div>
                        <h1 class="text-2xl font-bold tracking-tight text-base-content">"Budgetplaner"</h1>
                        <p class="text-xs sm:text-sm text-base-content/60">
                            "Finanzverwaltung in Schweizer Franken (CHF)"
                        </p>
                    </div>

                    // Modus-Umschalter: Anmelden / Registrieren
                    <div role="tablist" class="tabs tabs-box grid grid-cols-2">
                        <button
                            type="button"
                            role="tab"
                            class=move || format!("tab font-medium {}", if !is_register_mode.get() { "tab-active" } else { "" })
                            on:click=move |_| {
                                set_is_register_mode.set(false);
                                set_error_msg.set(None);
                            }
                        >
                            "Anmelden"
                        </button>
                        <button
                            type="button"
                            role="tab"
                            class=move || format!("tab font-medium {}", if is_register_mode.get() { "tab-active" } else { "" })
                            on:click=move |_| {
                                set_is_register_mode.set(true);
                                set_error_msg.set(None);
                            }
                        >
                            "Registrieren"
                        </button>
                    </div>

                    {move || error_msg.get().map(|msg| view! {
                        <div class="alert alert-error py-2.5 text-sm">
                            <span>{msg}</span>
                        </div>
                    })}

                    <form on:submit=on_submit class="space-y-4">
                        <div class="form-control">
                            <label class="label py-1">
                                <span class="label-text font-medium text-sm">"Benutzername"</span>
                            </label>
                            <input
                                type="text"
                                placeholder="z.B. admin"
                                autocomplete="username"
                                class="input input-bordered w-full"
                                prop:value=move || benutzername.get()
                                on:input=move |ev| set_benutzername.set(event_target_value(&ev))
                                required
                            />
                        </div>

                        <div class="form-control">
                            <label class="label py-1">
                                <span class="label-text font-medium text-sm">"Passwort"</span>
                            </label>
                            <input
                                type="password"
                                placeholder="••••••••"
                                autocomplete="current-password"
                                class="input input-bordered w-full"
                                prop:value=move || passwort.get()
                                on:input=move |ev| set_passwort.set(event_target_value(&ev))
                                required
                            />
                        </div>

                        <button
                            type="submit"
                            class="btn btn-primary w-full mt-2"
                            disabled=move || auth_action.pending().get()
                        >
                            {move || if auth_action.pending().get() {
                                "Bitte warten..."
                            } else if is_register_mode.get() {
                                "Konto erstellen & anmelden"
                            } else {
                                "Anmelden"
                            }}
                        </button>
                    </form>

                    <div class="p-3 rounded-box bg-base-200/80 text-xs text-base-content/70 flex items-center justify-between">
                        <div>
                            <span class="font-semibold text-base-content">"Standard-Zugang: "</span>
                            <code>"admin"</code>" / "<code>"admin123"</code>
                        </div>
                        <button
                            type="button"
                            class="btn btn-ghost btn-xs text-primary"
                            on:click=move |_| {
                                set_is_register_mode.set(false);
                                set_benutzername.set("admin".to_string());
                                set_passwort.set("admin123".to_string());
                            }
                        >
                            "Ausfüllen"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
