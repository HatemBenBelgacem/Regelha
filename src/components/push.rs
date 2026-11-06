use leptos::prelude::*;

// Die Funktionen aus public/push.js. Sie gibt es nur im Browser.
#[cfg(feature = "hydrate")]
mod js {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        /// Status: 'aktiv', 'inaktiv', 'verweigert' oder 'nicht_unterstuetzt'
        #[wasm_bindgen(js_name = regelhaPushStatus)]
        pub fn status() -> js_sys::Promise;

        /// Gerät anmelden: 'ok', 'verweigert', 'nicht_unterstuetzt' oder 'fehler:<Text>'
        #[wasm_bindgen(js_name = regelhaPushAbonnieren)]
        pub fn abonnieren() -> js_sys::Promise;
    }
}

/// Schalter für Push-Benachrichtigungen in der Seitenleiste.
/// Solange der Status nicht geprüft ist, wird nichts angezeigt (Server und Browser bleiben gleich).
#[component]
pub fn PushBenachrichtigungen() -> impl IntoView {
    let status = RwSignal::new(String::new());
    let meldung = RwSignal::new(Option::<String>::None);
    let laeuft = RwSignal::new(false);

    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            let antwort = wasm_bindgen_futures::JsFuture::from(js::status()).await;
            let text = antwort
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "nicht_unterstuetzt".to_string());
            status.set(text);
        });
    });

    let aktivieren = move |_| {
        if laeuft.get() {
            return;
        }
        laeuft.set(true);
        meldung.set(None);
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            let antwort = wasm_bindgen_futures::JsFuture::from(js::abonnieren()).await;
            let text = antwort
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "fehler:Unbekannter Fehler".to_string());
            laeuft.set(false);
            match text.as_str() {
                "ok" => status.set("aktiv".to_string()),
                "verweigert" => status.set("verweigert".to_string()),
                "nicht_unterstuetzt" => status.set("nicht_unterstuetzt".to_string()),
                _ => meldung.set(Some(text.trim_start_matches("fehler:").to_string())),
            }
        });
    };

    view! {
        {move || match status.get().as_str() {
            "inaktiv" => view! {
                <button
                    type="button"
                    class="btn btn-outline w-full"
                    disabled=move || laeuft.get()
                    on:click=aktivieren
                >
                    "Benachrichtigungen aktivieren"
                </button>
            }
            .into_any(),
            "aktiv" => view! {
                <p class="text-xs text-success text-center">"Benachrichtigungen sind aktiv"</p>
            }
            .into_any(),
            "verweigert" => view! {
                <p class="text-xs text-warning">
                    "Benachrichtigungen sind blockiert. Erlaube sie in den Einstellungen des Browsers."
                </p>
            }
            .into_any(),
            _ => ().into_any(),
        }}
        {move || meldung.get().map(|m| view! { <p class="text-xs text-error">{m}</p> })}
    }
}
