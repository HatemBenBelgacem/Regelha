use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;

use leptos::{prelude::*, task::spawn_local};
use uuid::Uuid;

use crate::backend::chat::{
    chat_live_abfrage, create_group_chat, get_chat_nachrichten, get_chat_uebersicht,
    open_direct_chat, send_chat_nachricht, set_chat_tippstatus, ChatNachricht, ChatRaum,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct PendingAnhang {
    name: String,
    typ: String,
    daten: String,
}

#[cfg(feature = "hydrate")]
mod file_bridge {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        type JsFunction;

        #[wasm_bindgen(constructor, js_class = "Function")]
        fn new_with_args(args: &str, body: &str) -> JsFunction;

        #[wasm_bindgen(method, js_name = call)]
        fn call2(
            this: &JsFunction,
            context: &JsValue,
            ev: &JsValue,
            callback: &Closure<dyn FnMut(String, String, String)>,
        );
    }

    const READ_FILES_HELPER: &str = r#"
        function readFilesList(fileList, cb) {
            if (!fileList || fileList.length === 0) return;
            for (let i = 0; i < fileList.length; i++) {
                const file = fileList[i];
                if (!file) continue;
                const reader = new FileReader();
                const name = file.name || ("Anhang-" + (i + 1));
                const mime = file.type || "application/octet-stream";
                reader.onload = function(e) {
                    if (e.target && typeof e.target.result === "string") {
                        cb(name, mime, e.target.result);
                    }
                };
                reader.readAsDataURL(file);
            }
        }
    "#;

    pub fn handle_drop_event(
        ev: JsValue,
        callback: &Closure<dyn FnMut(String, String, String)>,
    ) {
        let body = format!(
            "{READ_FILES_HELPER}\n\
            ev.preventDefault();\n\
            ev.stopPropagation();\n\
            if (ev.dataTransfer && ev.dataTransfer.files) {{\n\
                readFilesList(ev.dataTransfer.files, callback);\n\
            }}"
        );
        let f = JsFunction::new_with_args("ev, callback", &body);
        f.call2(&JsValue::NULL, &ev, callback);
    }

    pub fn handle_paste_event(
        ev: JsValue,
        callback: &Closure<dyn FnMut(String, String, String)>,
    ) {
        let body = format!(
            "{READ_FILES_HELPER}\n\
            if (!ev.clipboardData) return;\n\
            if (ev.clipboardData.files && ev.clipboardData.files.length > 0) {{\n\
                ev.preventDefault();\n\
                readFilesList(ev.clipboardData.files, callback);\n\
                return;\n\
            }}\n\
            if (ev.clipboardData.items) {{\n\
                const files = [];\n\
                for (let i = 0; i < ev.clipboardData.items.length; i++) {{\n\
                    const item = ev.clipboardData.items[i];\n\
                    if (item.kind === 'file') {{\n\
                        const file = item.getAsFile();\n\
                        if (file) files.push(file);\n\
                    }}\n\
                }}\n\
                if (files.length > 0) {{\n\
                    ev.preventDefault();\n\
                    readFilesList(files, callback);\n\
                }}\n\
            }}"
        );
        let f = JsFunction::new_with_args("ev, callback", &body);
        f.call2(&JsValue::NULL, &ev, callback);
    }

    pub fn handle_file_input_event(
        ev: JsValue,
        callback: &Closure<dyn FnMut(String, String, String)>,
    ) {
        let body = format!(
            "{READ_FILES_HELPER}\n\
            const target = ev.target;\n\
            if (target && target.files) {{\n\
                readFilesList(target.files, callback);\n\
                target.value = '';\n\
            }}"
        );
        let f = JsFunction::new_with_args("ev, callback", &body);
        f.call2(&JsValue::NULL, &ev, callback);
    }
}

/// Signale, die von der Live-Abfrage aktualisiert werden.
#[derive(Clone, Copy)]
struct LiveSignale {
    nachrichten: RwSignal<Vec<ChatNachricht>>,
    tippende: RwSignal<Vec<String>>,
}

/// Fügt Nachrichten an, die noch nicht in der Liste sind (Duplikate über die ID erkannt).
fn nachrichten_anhaengen(liste: &mut Vec<ChatNachricht>, neue: Vec<ChatNachricht>) {
    for m in neue {
        if !liste.iter().any(|x| x.id == m.id) {
            liste.push(m);
        }
    }
}

/// Long-Polling-Schleife: wartet serverseitig auf neue Nachrichten oder Tipp-Signale im Chat
/// und schreibt sie direkt in die Signale. Endet, sobald sich die Generation in `lauf` ändert
/// (anderer Chat ausgewählt oder Komponente entfernt).
fn live_abfrage(
    lauf: Arc<AtomicU64>,
    generation: u64,
    chat_id: Uuid,
    letzte_id: Option<Uuid>,
    version: Option<u64>,
    tipp_signatur: String,
    signale: LiveSignale,
) {
    spawn_local(async move {
        let res = chat_live_abfrage(chat_id, letzte_id, version, tipp_signatur.clone()).await;
        if lauf.load(Ordering::SeqCst) != generation {
            return;
        }
        match res {
            Ok(status) => {
                let letzte_id = status.neue_nachrichten.last().map(|m| m.id).or(letzte_id);
                if !status.neue_nachrichten.is_empty() {
                    signale
                        .nachrichten
                        .try_update(|l| nachrichten_anhaengen(l, status.neue_nachrichten));
                }
                let signatur = status.tippende.join("|");
                if signatur != tipp_signatur {
                    signale.tippende.try_set(status.tippende);
                }
                live_abfrage(
                    lauf,
                    generation,
                    chat_id,
                    letzte_id,
                    Some(status.nachrichten_version),
                    signatur,
                    signale,
                );
            }
            // Netzwerk-/Serverfehler: nach kurzer Pause erneut versuchen
            Err(_) => set_timeout(
                move || {
                    live_abfrage(
                        lauf,
                        generation,
                        chat_id,
                        letzte_id,
                        version,
                        tipp_signatur,
                        signale,
                    )
                },
                Duration::from_secs(3),
            ),
        }
    });
}

fn initiale(name: &str) -> String {
    name.chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string()
}

fn tipp_text(namen: &[String]) -> String {
    match namen {
        [] => String::new(),
        [a] => format!("{a} schreibt"),
        [a, b] => format!("{a} und {b} schreiben"),
        _ => "Mehrere Personen schreiben".to_string(),
    }
}

const LISTEN_ZEILE_AKTIV: &str = "w-full text-left px-3 py-3 md:py-2 min-h-16 md:min-h-0 rounded-xl bg-[#66023C]/10 text-[#66023C] font-semibold transition-colors flex items-center gap-3";
const LISTEN_ZEILE: &str = "w-full text-left px-3 py-3 md:py-2 min-h-16 md:min-h-0 rounded-xl hover:bg-slate-100 active:bg-slate-200 text-slate-700 transition-colors flex items-center gap-3";

#[component]
pub fn ChatSection() -> impl IntoView {
    let uebersicht_resource = Resource::new(|| (), |_| get_chat_uebersicht());

    let (active_chat, set_active_chat) = signal(Option::<ChatRaum>::None);
    let active_chat_id = Memo::new(move |_| active_chat.get().map(|c| c.id));
    let my_id = Memo::new(move |_| {
        uebersicht_resource
            .get()
            .and_then(|r| r.ok())
            .map(|u| u.aktueller_benutzer_id)
    });

    // Auf Smartphones (< md) wird entweder die Liste oder der Chat angezeigt
    let (mobile_show_list, set_mobile_show_list) = signal(true);
    let (nachricht_text, set_nachricht_text) = signal(String::new());
    let (pending_anhaenge, set_pending_anhaenge) = signal(Vec::<PendingAnhang>::new());
    let (is_dragging, set_is_dragging) = signal(false);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    // Nachrichten & Tipp-Anzeige des aktiven Chats (live aktualisiert)
    let nachrichten = RwSignal::new(Vec::<ChatNachricht>::new());
    let tippende = RwSignal::new(Vec::<String>::new());
    let geladen_fuer = RwSignal::new(Option::<Uuid>::None);
    let signale = LiveSignale {
        nachrichten,
        tippende,
    };
    // Zeitpunkt (ms) der letzten "tippt"-Meldung an den Server, 0 = keine aktiv
    let letzte_tipp_meldung = StoredValue::new(0_i64);
    let liste_ref = NodeRef::<leptos::html::Div>::new();

    // Gruppenchat-Erstellung
    let (show_group_form, set_show_group_form) = signal(false);
    let (group_name, set_group_name) = signal(String::new());
    let (selected_group_members, set_selected_group_members) = signal(Vec::<Uuid>::new());
    let (oeffne_partner, set_oeffne_partner) = signal(Option::<Uuid>::None);

    // Generation der Live-Abfrage: jede Änderung beendet die laufende Schleife
    let live_lauf = Arc::new(AtomicU64::new(0));

    on_cleanup({
        let lauf = live_lauf.clone();
        move || {
            lauf.fetch_add(1, Ordering::SeqCst);
        }
    });

    // Chat gewechselt: Nachrichten laden und anschließend die Live-Abfrage starten
    Effect::new({
        let lauf = live_lauf.clone();
        move |_| {
            let cid = active_chat_id.get();
            let generation = lauf.fetch_add(1, Ordering::SeqCst) + 1;
            nachrichten.set(Vec::new());
            tippende.set(Vec::new());
            geladen_fuer.set(None);
            letzte_tipp_meldung.set_value(0);

            let Some(cid) = cid else { return };
            let lauf = lauf.clone();
            spawn_local(async move {
                let res = get_chat_nachrichten(cid).await;
                if lauf.load(Ordering::SeqCst) != generation {
                    return;
                }
                match res {
                    Ok(list) => {
                        let letzte_id = list.last().map(|m| m.id);
                        nachrichten.try_set(list);
                        geladen_fuer.try_set(Some(cid));
                        live_abfrage(lauf, generation, cid, letzte_id, None, String::new(), signale);
                    }
                    Err(e) => {
                        geladen_fuer.try_set(Some(cid));
                        set_error_msg.try_set(Some(format!("Fehler beim Laden der Nachrichten: {e}")));
                    }
                }
            });
        }
    });

    // Bei neuen Nachrichten / Tipp-Anzeige ans Ende scrollen
    Effect::new(move |_| {
        nachrichten.with(|l| l.len());
        tippende.with(|t| t.len());
        geladen_fuer.get();
        request_animation_frame(move || {
            if let Some(el) = liste_ref.get_untracked() {
                el.set_scroll_top(el.scroll_height());
            }
        });
    });

    // Wenn die Übersicht geladen ist und noch kein Chat ausgewählt wurde, ersten vorhandenen Chat auswählen
    Effect::new(move |_| {
        if active_chat.get().is_none() {
            if let Some(Ok(ref ueb)) = uebersicht_resource.get() {
                if let Some(first_chat) = ueb.chats.first() {
                    set_active_chat.set(Some(first_chat.clone()));
                }
            }
        }
    });

    let open_direct_action = Action::new(move |partner_id: &Uuid| {
        let partner_id = *partner_id;
        async move { open_direct_chat(partner_id).await }
    });

    Effect::new(move |_| {
        if let Some(res) = open_direct_action.value().get() {
            set_oeffne_partner.set(None);
            match res {
                Ok(raum) => {
                    let ist_neu = uebersicht_resource
                        .get_untracked()
                        .and_then(|r| r.ok())
                        .map(|u| !u.chats.iter().any(|c| c.id == raum.id))
                        .unwrap_or(true);
                    set_active_chat.set(Some(raum));
                    set_mobile_show_list.set(false);
                    set_error_msg.set(None);
                    if ist_neu {
                        uebersicht_resource.refetch();
                    }
                }
                Err(e) => set_error_msg.set(Some(e.to_string())),
            }
        }
    });

    let create_group_action = Action::new(move |(name, members): &(String, Vec<Uuid>)| {
        let name = name.clone();
        let members = members.clone();
        async move { create_group_chat(name, members).await }
    });

    Effect::new(move |_| {
        if let Some(res) = create_group_action.value().get() {
            match res {
                Ok(raum) => {
                    set_group_name.set(String::new());
                    set_selected_group_members.set(Vec::new());
                    set_show_group_form.set(false);
                    set_active_chat.set(Some(raum));
                    set_mobile_show_list.set(false);
                    set_error_msg.set(None);
                    uebersicht_resource.refetch();
                }
                Err(e) => set_error_msg.set(Some(e.to_string())),
            }
        }
    });

    let send_action = Action::new(
        move |(chat_id, text, anhaenge): &(Uuid, String, Vec<PendingAnhang>)| {
            let chat_id = *chat_id;
            let text = text.clone();
            let anhaenge = anhaenge.clone();
            async move {
                let mut gesendet = Vec::new();
                if anhaenge.is_empty() {
                    gesendet.push(send_chat_nachricht(chat_id, text, None, None, None).await?);
                } else {
                    for (idx, a) in anhaenge.into_iter().enumerate() {
                        let msg_text = if idx == 0 {
                            text.clone()
                        } else {
                            String::new()
                        };
                        gesendet.push(
                            send_chat_nachricht(
                                chat_id,
                                msg_text,
                                Some(a.name),
                                Some(a.typ),
                                Some(a.daten),
                            )
                            .await?,
                        );
                    }
                }
                Ok::<Vec<ChatNachricht>, ServerFnError>(gesendet)
            }
        },
    );

    Effect::new(move |_| {
        if let Some(res) = send_action.value().get() {
            match res {
                Ok(gesendet) => {
                    set_nachricht_text.set(String::new());
                    set_pending_anhaenge.set(Vec::new());
                    set_error_msg.set(None);
                    // Der Server hat den Tipp-Status mit dem Senden bereits zurückgesetzt
                    letzte_tipp_meldung.set_value(0);
                    // Eigene Nachrichten sofort anzeigen (die Live-Abfrage liefert sie ggf. nochmals – Duplikate werden ignoriert)
                    let aktiv = active_chat_id.get_untracked();
                    let gesendet: Vec<ChatNachricht> = gesendet
                        .into_iter()
                        .filter(|m| Some(m.chat_id) == aktiv)
                        .collect();
                    nachrichten.update(|l| nachrichten_anhaengen(l, gesendet));
                }
                Err(e) => set_error_msg.set(Some(e.to_string())),
            }
        }
    });

    let melde_tippen = move |tippt: bool| {
        let Some(cid) = active_chat_id.get_untracked() else {
            return;
        };
        spawn_local(async move {
            let _ = set_chat_tippstatus(cid, tippt).await;
        });
    };

    let on_text_input = move |ev: leptos::ev::Event| {
        let wert = event_target_value(&ev);
        let leer = wert.trim().is_empty();
        set_nachricht_text.set(wert);

        let jetzt = chrono::Utc::now().timestamp_millis();
        let zuletzt = letzte_tipp_meldung.get_value();
        if leer {
            if zuletzt != 0 {
                letzte_tipp_meldung.set_value(0);
                melde_tippen(false);
            }
        } else if jetzt - zuletzt > 2500 {
            // Signal regelmäßig erneuern, solange getippt wird (läuft serverseitig nach 5 s ab)
            letzte_tipp_meldung.set_value(jetzt);
            melde_tippen(true);
        }
    };

    #[cfg(feature = "hydrate")]
    let add_anhang = move |name: String, typ: String, daten: String| {
        set_pending_anhaenge.update(|list| {
            list.push(PendingAnhang { name, typ, daten });
        });
    };

    let on_drop_files = move |ev: leptos::ev::DragEvent| {
        ev.prevent_default();
        set_is_dragging.set(false);
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::{closure::Closure, JsValue};
            let cb = Closure::wrap(Box::new(move |name: String, typ: String, daten: String| {
                add_anhang(name, typ, daten);
            })
                as Box<dyn FnMut(String, String, String)>);
            file_bridge::handle_drop_event(JsValue::from(ev), &cb);
            cb.forget();
        }
    };

    let on_paste_files = move |ev: leptos::ev::ClipboardEvent| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::{closure::Closure, JsValue};
            let cb = Closure::wrap(Box::new(move |name: String, typ: String, daten: String| {
                add_anhang(name, typ, daten);
            })
                as Box<dyn FnMut(String, String, String)>);
            file_bridge::handle_paste_event(JsValue::from(ev), &cb);
            cb.forget();
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = ev;
    };

    let on_file_input_change = move |ev: leptos::ev::Event| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::{closure::Closure, JsValue};
            let cb = Closure::wrap(Box::new(move |name: String, typ: String, daten: String| {
                add_anhang(name, typ, daten);
            })
                as Box<dyn FnMut(String, String, String)>);
            file_bridge::handle_file_input_event(JsValue::from(ev), &cb);
            cb.forget();
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = ev;
    };

    let on_send_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let Some(current_chat) = active_chat.get() else {
            set_error_msg.set(Some(
                "Bitte zuerst einen Chatteilnehmer oder Gruppenchat auswählen.".to_string(),
            ));
            return;
        };
        let text = nachricht_text.get();
        let anhaenge = pending_anhaenge.get();
        if text.trim().is_empty() && anhaenge.is_empty() {
            return;
        }
        set_error_msg.set(None);
        send_action.dispatch((current_chat.id, text, anhaenge));
    };

    let on_create_group_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let name = group_name.get();
        if name.trim().is_empty() {
            set_error_msg.set(Some(
                "Bitte einen Namen für den Gruppenchat eingeben.".to_string(),
            ));
            return;
        }
        create_group_action.dispatch((name, selected_group_members.get()));
    };

    view! {
        <section class="w-full max-w-none flex-1 flex flex-col gap-3 sm:gap-4">
            // Kopfbereich – auf Smartphones in der Chatansicht ausgeblendet (mehr Platz für Nachrichten)
            <div class=move || if mobile_show_list.get() { "block" } else { "hidden md:block" }>
                <h1 class="text-2xl lg:text-3xl font-bold tracking-tight">"Chat"</h1>
                <p class="hidden sm:block text-sm text-base-content/70">
                    "Direkt- und Gruppenchats mit Datei- & Bildupload per Drag & Drop oder Copy & Paste"
                </p>
            </div>

            {move || error_msg.get().map(|msg| view! {
                <div role="alert" class="alert alert-warning py-2 text-sm flex items-center justify-between gap-2">
                    <span>{msg}</span>
                    <button
                        type="button"
                        class="btn btn-ghost btn-sm btn-square min-h-11 min-w-11 md:min-h-0 md:min-w-0"
                        aria-label="Hinweis schließen"
                        on:click=move |_| set_error_msg.set(None)
                    >
                        "✕"
                    </button>
                </div>
            })}

            // Haupt-Container: Smartphone = Liste ODER Chat, ab md = beide Spalten nebeneinander
            <div class=move || {
                if mobile_show_list.get() {
                    "w-full grid grid-cols-1 md:grid-cols-12 gap-3 sm:gap-4 h-[calc(100dvh-8.5rem)] sm:h-[calc(100dvh-10rem)] md:h-[calc(100dvh-10rem)] lg:h-[calc(100dvh-11rem)] min-h-[360px] md:min-h-[520px]"
                } else {
                    "w-full grid grid-cols-1 md:grid-cols-12 gap-3 sm:gap-4 h-[calc(100dvh-4.5rem)] sm:h-[calc(100dvh-6rem)] md:h-[calc(100dvh-10rem)] lg:h-[calc(100dvh-11rem)] min-h-[360px] md:min-h-[520px]"
                }
            }>
                // LINKE SPALTE: Gruppenchats & Chatteilnehmer
                <div class=move || {
                    if mobile_show_list.get() {
                        "flex md:col-span-5 lg:col-span-4 xl:col-span-3 card bg-base-100 shadow-sm border border-base-300 flex-col h-full overflow-hidden"
                    } else {
                        "hidden md:flex md:col-span-5 lg:col-span-4 xl:col-span-3 card bg-base-100 shadow-sm border border-base-300 flex-col h-full overflow-hidden"
                    }
                }>
                    <div class="flex-1 overflow-y-auto p-2 sm:p-3 space-y-4">
                        // 1. Bereich: Gruppenchats + Button zum Erstellen
                        <div class="space-y-1.5">
                            <div class="flex items-center justify-between gap-2 px-1 pt-1">
                                <h2 class="text-xs font-bold uppercase tracking-wider text-slate-500">
                                    "Gruppenchats"
                                </h2>
                                <button
                                    type="button"
                                    class="btn btn-md md:btn-sm min-h-11 md:min-h-0 rounded-full bg-[#66023C] hover:bg-[#7A2048] text-white border-none px-4"
                                    on:click=move |_| set_show_group_form.update(|v| *v = !*v)
                                >
                                    {move || if show_group_form.get() { "Abbrechen" } else { "+ Gruppe" }}
                                </button>
                            </div>

                            // Formular zum Erstellen eines neuen Gruppenchats
                            {move || show_group_form.get().then(|| view! {
                                <form
                                    on:submit=on_create_group_submit
                                    class="p-3 rounded-xl bg-slate-50 border border-slate-200 space-y-3"
                                >
                                    <label class="block space-y-1">
                                        <span class="text-xs font-semibold text-slate-600">"Gruppenname"</span>
                                        <input
                                            type="text"
                                            placeholder="z.B. Haushalt, Projekt, Familie..."
                                            class="input input-bordered input-md md:input-sm w-full text-base md:text-sm"
                                            prop:value=move || group_name.get()
                                            on:input=move |ev| set_group_name.set(event_target_value(&ev))
                                            required
                                        />
                                    </label>

                                    <div>
                                        <div class="text-xs font-semibold mb-1 text-slate-600">
                                            "Teilnehmer auswählen:"
                                        </div>
                                        <div class="max-h-48 md:max-h-32 overflow-y-auto pr-1">
                                            {move || uebersicht_resource.get().and_then(|r| r.ok()).map(|ueb| {
                                                let my_id = ueb.aktueller_benutzer_id;
                                                ueb.alle_teilnehmer.into_iter().map(|t| {
                                                    let uid = t.id;
                                                    let is_self = uid == my_id;
                                                    let checked = move || is_self || selected_group_members.get().contains(&uid);
                                                    view! {
                                                        <label class="flex items-center gap-3 text-sm md:text-xs cursor-pointer py-2 md:py-1 px-1 rounded-lg hover:bg-white">
                                                            <input
                                                                type="checkbox"
                                                                class="checkbox checkbox-md md:checkbox-xs checkbox-primary"
                                                                prop:checked=checked
                                                                disabled=is_self
                                                                on:change=move |ev| {
                                                                    let is_checked = event_target_checked(&ev);
                                                                    set_selected_group_members.update(|list| {
                                                                        if is_checked {
                                                                            if !list.contains(&uid) {
                                                                                list.push(uid);
                                                                            }
                                                                        } else {
                                                                            list.retain(|x| *x != uid);
                                                                        }
                                                                    });
                                                                }
                                                            />
                                                            <span class="truncate">
                                                                {t.benutzername}
                                                                {if is_self { " (Du)" } else { "" }}
                                                            </span>
                                                        </label>
                                                    }
                                                }).collect_view()
                                            })}
                                        </div>
                                    </div>

                                    <button
                                        type="submit"
                                        class="btn btn-primary btn-md md:btn-sm min-h-11 md:min-h-0 w-full"
                                        disabled=move || create_group_action.pending().get()
                                    >
                                        "Gruppenchat erstellen"
                                    </button>
                                </form>
                            })}

                            <Suspense fallback=move || view! {
                                <div class="text-xs text-slate-400 py-2 px-1">"Lade Chats..."</div>
                            }>
                                {move || uebersicht_resource.get().map(|res| match res {
                                    Ok(ueb) => {
                                        let gruppen: Vec<ChatRaum> = ueb
                                            .chats
                                            .into_iter()
                                            .filter(|c| c.ist_gruppe)
                                            .collect();

                                        if gruppen.is_empty() {
                                            view! {
                                                <div class="text-xs text-slate-400 italic py-1 px-1">
                                                    "Noch kein Gruppenchat vorhanden."
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <ul class="space-y-0.5">
                                                    {gruppen.into_iter().map(|g| {
                                                        let g_clone = g.clone();
                                                        let gid = g.id;
                                                        let member_names = g
                                                            .teilnehmer
                                                            .iter()
                                                            .map(|t| t.benutzername.as_str())
                                                            .collect::<Vec<_>>()
                                                            .join(", ");

                                                        let btn_class = move || {
                                                            if active_chat_id.get() == Some(gid) {
                                                                LISTEN_ZEILE_AKTIV
                                                            } else {
                                                                LISTEN_ZEILE
                                                            }
                                                        };

                                                        view! {
                                                            <li>
                                                                <button
                                                                    type="button"
                                                                    class=btn_class
                                                                    on:click=move |_| {
                                                                        set_active_chat.set(Some(g_clone.clone()));
                                                                        set_mobile_show_list.set(false);
                                                                    }
                                                                >
                                                                    <div class="w-11 h-11 md:w-9 md:h-9 rounded-full bg-[#7A2048]/15 text-[#66023C] flex items-center justify-center shrink-0">
                                                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                                                            <path stroke-linecap="round" stroke-linejoin="round" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0z" />
                                                                        </svg>
                                                                    </div>
                                                                    <div class="min-w-0 flex-1">
                                                                        <div class="text-base md:text-sm truncate">{g.name}</div>
                                                                        <div class="text-xs md:text-[11px] text-slate-400 font-normal truncate">
                                                                            {member_names}
                                                                        </div>
                                                                    </div>
                                                                    <span class="badge badge-ghost badge-sm shrink-0">
                                                                        {format!("{}", g.teilnehmer.len())}
                                                                    </span>
                                                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 text-slate-300 shrink-0 md:hidden" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                                                        <path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7" />
                                                                    </svg>
                                                                </button>
                                                            </li>
                                                        }
                                                    }).collect_view()}
                                                </ul>
                                            }.into_any()
                                        }
                                    }
                                    Err(_) => view! { <div></div> }.into_any(),
                                })}
                            </Suspense>
                        </div>

                        <div class="border-t border-slate-100"></div>

                        // 2. Bereich: Alle Chatteilnehmer (Klick öffnet den Direktchat)
                        <div class="space-y-1.5">
                            <div class="flex items-center justify-between gap-2 px-1">
                                <h2 class="text-xs font-bold uppercase tracking-wider text-slate-500">
                                    "Chatteilnehmer"
                                </h2>
                                <span class="text-[11px] text-slate-400">
                                    "Antippen für Direktchat"
                                </span>
                            </div>

                            <Suspense fallback=move || view! {
                                <div class="flex justify-center p-4">
                                    <span class="loading loading-spinner loading-sm text-primary"></span>
                                </div>
                            }>
                                {move || uebersicht_resource.get().map(|res| match res {
                                    Ok(ueb) => {
                                        let my_id = ueb.aktueller_benutzer_id;
                                        view! {
                                            <ul class="space-y-0.5">
                                                {ueb.alle_teilnehmer.into_iter().map(|teilnehmer| {
                                                    let tid = teilnehmer.id;
                                                    let tname = teilnehmer.benutzername.clone();
                                                    let initial = initiale(&tname);
                                                    let is_self = tid == my_id;

                                                    let is_direct_partner = move || {
                                                        active_chat
                                                            .get()
                                                            .map(|c| {
                                                                !c.ist_gruppe && (
                                                                    (is_self && c.teilnehmer.len() == 1 && c.teilnehmer[0].id == my_id)
                                                                    || (!is_self && c.teilnehmer.iter().any(|m| m.id == tid))
                                                                )
                                                            })
                                                            .unwrap_or(false)
                                                    };

                                                    let row_class = move || {
                                                        if is_direct_partner() { LISTEN_ZEILE_AKTIV } else { LISTEN_ZEILE }
                                                    };
                                                    let wird_geoeffnet = move || {
                                                        oeffne_partner.get() == Some(tid) && open_direct_action.pending().get()
                                                    };

                                                    view! {
                                                        <li>
                                                            <button
                                                                type="button"
                                                                class=row_class
                                                                on:click=move |_| {
                                                                    set_oeffne_partner.set(Some(tid));
                                                                    open_direct_action.dispatch(tid);
                                                                }
                                                            >
                                                                <div class="w-11 h-11 md:w-9 md:h-9 rounded-full bg-[#66023C] text-white flex items-center justify-center text-sm font-bold shrink-0">
                                                                    {initial}
                                                                </div>
                                                                <div class="min-w-0 flex-1 text-base md:text-sm truncate">
                                                                    {tname}
                                                                    {if is_self { " (Du)" } else { "" }}
                                                                </div>
                                                                {move || if wird_geoeffnet() {
                                                                    view! { <span class="loading loading-spinner loading-sm text-[#66023C] shrink-0"></span> }.into_any()
                                                                } else {
                                                                    view! {
                                                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 text-slate-300 shrink-0 md:hidden" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                                                            <path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7" />
                                                                        </svg>
                                                                    }.into_any()
                                                                }}
                                                            </button>
                                                        </li>
                                                    }
                                                }).collect_view()}
                                            </ul>
                                        }.into_any()
                                    }
                                    Err(e) => view! {
                                        <div class="alert alert-error text-xs">
                                            <span>{format!("Fehler: {e}")}</span>
                                        </div>
                                    }.into_any(),
                                })}
                            </Suspense>
                        </div>
                    </div>
                </div>

                // RECHTE SPALTE: Nachrichtenverlauf + Drag & Drop / Copy & Paste Datei-Upload
                <div
                    class=move || {
                        let mobile_vis = if mobile_show_list.get() { "hidden md:flex" } else { "flex" };
                        let border_style = if is_dragging.get() {
                            "border-2 border-dashed border-[#66023C] bg-[#66023C]/5"
                        } else {
                            "border border-base-300"
                        };
                        format!(
                            "{mobile_vis} md:col-span-7 lg:col-span-8 xl:col-span-9 card bg-base-100 shadow-sm {border_style} flex-col h-full overflow-hidden transition-all"
                        )
                    }
                    on:dragover=move |ev| {
                        ev.prevent_default();
                        set_is_dragging.set(true);
                    }
                    on:dragleave=move |_| {
                        set_is_dragging.set(false);
                    }
                    on:drop=on_drop_files
                >
                    // Kopfbereich des ausgewählten Chats
                    <div class="flex items-center gap-2 sm:gap-3 px-2 sm:px-4 py-2 sm:py-3 border-b border-slate-200 shrink-0">
                        <button
                            type="button"
                            class="btn btn-ghost btn-square btn-md min-h-11 min-w-11 md:hidden text-[#66023C] -ml-1"
                            on:click=move |_| set_mobile_show_list.set(true)
                            aria-label="Zurück zur Chatliste"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.25">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M15 19l-7-7 7-7" />
                            </svg>
                        </button>
                        {move || match active_chat.get() {
                            Some(chat) => {
                                let teilnehmer_liste = chat
                                    .teilnehmer
                                    .iter()
                                    .map(|t| t.benutzername.clone())
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                let avatar = if chat.ist_gruppe {
                                    "👥".to_string()
                                } else {
                                    let me = my_id.get_untracked();
                                    chat.teilnehmer
                                        .iter()
                                        .find(|t| Some(t.id) != me)
                                        .or(chat.teilnehmer.first())
                                        .map(|t| initiale(&t.benutzername))
                                        .unwrap_or_else(|| "?".to_string())
                                };
                                view! {
                                    <div class="w-10 h-10 rounded-full bg-[#66023C] text-white flex items-center justify-center text-sm font-bold shrink-0">
                                        {avatar}
                                    </div>
                                    <div class="min-w-0 flex-1">
                                        <div class="flex items-center gap-2">
                                            <h2 class="text-base lg:text-lg font-bold text-[#66023C] truncate">
                                                {chat.name}
                                            </h2>
                                            <span class="hidden sm:inline-flex badge badge-sm badge-ghost shrink-0">
                                                {if chat.ist_gruppe { "Gruppenchat" } else { "Direktchat" }}
                                            </span>
                                        </div>
                                        {move || {
                                            let t = tippende.get();
                                            if t.is_empty() {
                                                view! {
                                                    <p class="text-xs text-slate-500 truncate">{teilnehmer_liste.clone()}</p>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <p class="text-xs font-medium text-[#7A2048] truncate">
                                                        {format!("{} …", tipp_text(&t))}
                                                    </p>
                                                }.into_any()
                                            }
                                        }}
                                    </div>
                                }.into_any()
                            }
                            None => view! {
                                <div class="min-w-0">
                                    <h2 class="text-base font-semibold text-slate-500">"Kein Chat ausgewählt"</h2>
                                    <p class="text-xs text-slate-400">
                                        "Wähle einen Chatteilnehmer oder erstelle einen Gruppenchat."
                                    </p>
                                </div>
                            }.into_any(),
                        }}
                    </div>

                    // Drag & Drop Hinweis-Banner wenn Datei gezogen wird
                    {move || is_dragging.get().then(|| view! {
                        <div class="mx-3 mt-3 p-3 rounded-xl bg-[#66023C]/10 border border-[#66023C] text-[#66023C] text-center text-sm font-semibold shrink-0">
                            "📂 Bilder oder Dokumente hier loslassen, um sie anzuhängen"
                        </div>
                    })}

                    // Nachrichtenliste
                    <div node_ref=liste_ref class="flex-1 overflow-y-auto min-h-0 px-3 sm:px-5 py-4 bg-[#F8FAFC]/60">
                        {move || {
                            let Some(cid) = active_chat_id.get() else {
                                return view! {
                                    <div class="h-full flex items-center justify-center text-slate-400 text-sm py-12 text-center px-4">
                                        "Klicke auf einen Chatteilnehmer oder Gruppenchat, um Nachrichten anzuzeigen."
                                    </div>
                                }.into_any();
                            };
                            if geladen_fuer.get() != Some(cid) {
                                return view! {
                                    <div class="flex justify-center p-8">
                                        <span class="loading loading-spinner loading-md text-primary"></span>
                                    </div>
                                }.into_any();
                            }
                            if nachrichten.with(|l| l.is_empty()) && tippende.with(|t| t.is_empty()) {
                                return view! {
                                    <div class="h-full flex flex-col items-center justify-center text-slate-400 text-sm py-12 space-y-1 text-center px-4">
                                        <p class="font-medium">"Noch keine Nachrichten in diesem Chat."</p>
                                        <p class="text-xs">"Schreibe eine Nachricht oder ziehe Bilder & Dokumente hierher."</p>
                                    </div>
                                }.into_any();
                            }
                            view! {
                                <div class="space-y-3">
                                    <For
                                        each=move || nachrichten.get()
                                        key=|m| m.id
                                        children=move |m| view! { <NachrichtBubble m=m my_id=my_id/> }
                                    />
                                </div>
                            }.into_any()
                        }}

                        // Tipp-Anzeige ("… schreibt") mit animierten Punkten
                        {move || {
                            let t = tippende.get();
                            (!t.is_empty()).then(|| view! {
                                <div class="flex items-end gap-2 mt-3" aria-live="polite">
                                    <div class="rounded-2xl rounded-bl-md bg-white border border-slate-200 shadow-xs px-4 py-3 flex items-center gap-1">
                                        <span class="w-2 h-2 rounded-full bg-[#66023C]/70 animate-bounce [animation-delay:-0.3s]"></span>
                                        <span class="w-2 h-2 rounded-full bg-[#66023C]/70 animate-bounce [animation-delay:-0.15s]"></span>
                                        <span class="w-2 h-2 rounded-full bg-[#66023C]/70 animate-bounce"></span>
                                    </div>
                                    <span class="text-xs text-slate-500 pb-1">{format!("{} …", tipp_text(&t))}</span>
                                </div>
                            })
                        }}
                    </div>

                    // Vorschau-Leiste für angehängte Bilder & Dokumente (vor dem Senden)
                    {move || {
                        let list = pending_anhaenge.get();
                        (!list.is_empty()).then(|| view! {
                            <div class="flex flex-wrap gap-2 px-3 pt-2 max-h-32 overflow-y-auto shrink-0">
                                {list.into_iter().enumerate().map(|(idx, item)| {
                                    let is_img = item.typ.starts_with("image/");
                                    view! {
                                        <div class="flex items-center gap-2 bg-white pl-2.5 pr-1 py-1 rounded-xl border border-slate-200 shadow-2xs text-xs">
                                            {if is_img {
                                                view! {
                                                    <img src=item.daten class="w-8 h-8 object-cover rounded shrink-0" alt=item.name.clone() />
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <span class="text-base shrink-0">"📎"</span>
                                                }.into_any()
                                            }}
                                            <span class="max-w-[140px] sm:max-w-[200px] truncate font-medium text-slate-700">
                                                {item.name}
                                            </span>
                                            <button
                                                type="button"
                                                class="btn btn-ghost btn-sm btn-square min-h-10 min-w-10 md:min-h-0 md:min-w-0 text-error"
                                                aria-label="Anhang entfernen"
                                                on:click=move |_| {
                                                    set_pending_anhaenge.update(|v| {
                                                        if idx < v.len() {
                                                            v.remove(idx);
                                                        }
                                                    });
                                                }
                                            >
                                                "✕"
                                            </button>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        })
                    }}

                    // Eingabezeile
                    <form on:submit=on_send_submit class="px-2 sm:px-4 py-2 sm:py-3 border-t border-slate-200 bg-base-100 shrink-0">
                        <div class="flex items-center gap-2">
                            <label
                                class="btn btn-ghost btn-circle btn-md md:btn-sm min-h-11 min-w-11 md:min-h-0 md:min-w-0 text-slate-500 hover:bg-[#66023C]/10 hover:text-[#66023C] cursor-pointer shrink-0"
                                title="Bilder oder Dokumente anhängen"
                                aria-label="Bilder oder Dokumente anhängen"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="w-6 h-6 md:w-5 md:h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13" />
                                </svg>
                                <input
                                    type="file"
                                    multiple
                                    class="hidden"
                                    on:change=on_file_input_change
                                />
                            </label>

                            <input
                                type="text"
                                placeholder="Nachricht schreiben…"
                                class="input input-bordered input-md md:input-sm min-h-11 md:min-h-0 rounded-full flex-1 min-w-0 text-base md:text-sm px-4 focus:border-[#66023C] focus:outline-[#66023C]/30"
                                prop:value=move || nachricht_text.get()
                                on:input=on_text_input
                                on:paste=on_paste_files
                                disabled=move || active_chat.get().is_none()
                            />

                            <button
                                type="submit"
                                class="btn btn-primary btn-circle sm:btn-wide sm:w-auto sm:rounded-full btn-md md:btn-sm min-h-11 min-w-11 md:min-h-0 md:min-w-0 sm:px-4 shrink-0"
                                aria-label="Nachricht senden"
                                disabled=move || active_chat.get().is_none() || send_action.pending().get()
                            >
                                {move || if send_action.pending().get() {
                                    view! { <span class="loading loading-spinner loading-sm"></span> }.into_any()
                                } else {
                                    view! {
                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 md:w-4 md:h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                            <path stroke-linecap="round" stroke-linejoin="round" d="M12 19V5m0 0l-6 6m6-6l6 6" />
                                        </svg>
                                    }.into_any()
                                }}
                                <span class="hidden sm:inline">"Senden"</span>
                            </button>
                        </div>
                        <p class="hidden lg:block text-[11px] text-slate-400 px-2 pt-1.5">
                            "Tipp: Bilder & Dokumente per Drag & Drop in das Chatfenster ziehen oder mit Cmd+V / Strg+V einfügen."
                        </p>
                    </form>
                </div>
            </div>
        </section>
    }
}

/// Eine einzelne Nachricht als Sprechblase (eigene rechts in Markenfarbe, fremde links).
#[component]
fn NachrichtBubble(m: ChatNachricht, my_id: Memo<Option<Uuid>>) -> impl IntoView {
    let absender = m.benutzer_id;
    let is_mine = move || my_id.get() == Some(absender);

    let is_image = m
        .anhang_typ
        .as_deref()
        .map(|t| t.starts_with("image/"))
        .unwrap_or(false);

    view! {
        <div class=move || if is_mine() { "flex flex-col items-end" } else { "flex flex-col items-start" }>
            {move || (!is_mine()).then(|| view! {
                <span class="text-[11px] font-semibold text-slate-500 mb-0.5 px-2">{m.benutzername.clone()}</span>
            })}
            <div class=move || {
                if is_mine() {
                    "max-w-[85%] sm:max-w-[75%] xl:max-w-[65%] rounded-2xl rounded-br-md px-3.5 py-2 bg-[#66023C] text-white shadow-xs space-y-1.5"
                } else {
                    "max-w-[85%] sm:max-w-[75%] xl:max-w-[65%] rounded-2xl rounded-bl-md px-3.5 py-2 bg-white text-slate-800 border border-slate-200 shadow-xs space-y-1.5"
                }
            }>
                {(!m.inhalt.is_empty()).then(|| view! {
                    <p class="text-[15px] md:text-sm leading-snug whitespace-pre-wrap break-words">{m.inhalt}</p>
                })}

                {m.anhang_daten.map(|daten| {
                    let dateiname = m.anhang_name.unwrap_or_else(|| "Anhang".to_string());
                    if is_image {
                        view! {
                            <div class="space-y-1 pt-0.5">
                                <a href=daten.clone() target="_blank" title="Bild öffnen">
                                    <img
                                        src=daten.clone()
                                        alt=dateiname.clone()
                                        class="max-h-64 sm:max-h-72 lg:max-h-80 w-auto rounded-lg object-contain bg-white"
                                    />
                                </a>
                                <div class="flex items-center justify-between text-[11px] opacity-85 gap-2">
                                    <span class="truncate max-w-[160px] sm:max-w-[260px]">{dateiname.clone()}</span>
                                    <a href=daten download=dateiname class="underline font-medium shrink-0 py-1">
                                        "Download"
                                    </a>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="flex items-center justify-between gap-2 sm:gap-3 p-2 rounded-xl bg-black/10 text-xs">
                                <div class="flex items-center gap-2 min-w-0">
                                    <span class="text-base shrink-0">"📄"</span>
                                    <span class="font-medium truncate">{dateiname.clone()}</span>
                                </div>
                                <a
                                    href=daten
                                    download=dateiname
                                    class="btn btn-sm min-h-10 md:min-h-0 bg-white text-[#66023C] hover:bg-slate-100 border-none shrink-0"
                                >
                                    "Herunterladen"
                                </a>
                            </div>
                        }.into_any()
                    }
                })}

                <div class=move || if is_mine() { "text-[10px] text-white/70 text-right" } else { "text-[10px] text-slate-400 text-right" }>
                    {m.zeitpunkt}
                </div>
            </div>
        </div>
    }
}
