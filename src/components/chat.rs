use leptos::prelude::*;
use uuid::Uuid;

use crate::backend::chat::{
    create_group_chat, get_chat_nachrichten, get_chat_uebersicht, open_direct_chat,
    send_chat_nachricht, ChatRaum,
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

#[component]
pub fn ChatSection() -> impl IntoView {
    let uebersicht_resource = Resource::new(|| (), |_| get_chat_uebersicht());

    let (active_chat, set_active_chat) = signal(Option::<ChatRaum>::None);
    let (nachricht_text, set_nachricht_text) = signal(String::new());
    let (pending_anhaenge, set_pending_anhaenge) = signal(Vec::<PendingAnhang>::new());
    let (is_dragging, set_is_dragging) = signal(false);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    // Gruppenchat-Erstellung
    let (show_group_form, set_show_group_form) = signal(false);
    let (group_name, set_group_name) = signal(String::new());
    let (selected_group_members, set_selected_group_members) = signal(Vec::<Uuid>::new());

    let nachrichten_resource = Resource::new(
        move || active_chat.get().map(|c| c.id),
        |chat_id_opt| async move {
            match chat_id_opt {
                Some(cid) => get_chat_nachrichten(cid).await,
                None => Ok(Vec::new()),
            }
        },
    );

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
            match res {
                Ok(raum) => {
                    set_active_chat.set(Some(raum));
                    set_error_msg.set(None);
                    uebersicht_resource.refetch();
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
                if anhaenge.is_empty() {
                    send_chat_nachricht(chat_id, text, None, None, None).await?;
                } else {
                    for (idx, a) in anhaenge.into_iter().enumerate() {
                        let msg_text = if idx == 0 {
                            text.clone()
                        } else {
                            String::new()
                        };
                        send_chat_nachricht(
                            chat_id,
                            msg_text,
                            Some(a.name),
                            Some(a.typ),
                            Some(a.daten),
                        )
                        .await?;
                    }
                }
                Ok::<(), ServerFnError>(())
            }
        },
    );

    Effect::new(move |_| {
        if let Some(res) = send_action.value().get() {
            match res {
                Ok(()) => {
                    set_nachricht_text.set(String::new());
                    set_pending_anhaenge.set(Vec::new());
                    set_error_msg.set(None);
                    nachrichten_resource.refetch();
                }
                Err(e) => set_error_msg.set(Some(e.to_string())),
            }
        }
    });

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
                "Bitte zuerst links einen Chatteilnehmer oder Gruppenchat auswählen.".to_string(),
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
        <section class="space-y-4">
            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
                <div>
                    <h1 class="text-2xl sm:text-3xl font-bold tracking-tight">"Chat"</h1>
                    <p class="text-xs sm:text-sm text-base-content/70">
                        "Direkt- und Gruppenchats mit Datei- & Bildupload per Drag & Drop oder Copy & Paste"
                    </p>
                </div>
            </div>

            {move || error_msg.get().map(|msg| view! {
                <div class="alert alert-warning py-2 text-sm">
                    <span>{msg}</span>
                </div>
            })}

            <div class="grid grid-cols-1 lg:grid-cols-12 gap-4 min-h-[640px]">
                // LINKE SPALTE: Chatteilnehmer & Gruppenchats
                <div class="lg:col-span-4 card bg-base-100 shadow-sm border border-base-300 flex flex-col">
                    <div class="card-body p-4 space-y-5 flex-1 overflow-y-auto">
                        // 1. Bereich: Gruppenchats + Button zum Erstellen
                        <div class="space-y-2">
                            <div class="flex items-center justify-between">
                                <h2 class="text-xs font-bold uppercase tracking-wider text-slate-500">
                                    "Gruppenchats"
                                </h2>
                                <button
                                    type="button"
                                    class="btn btn-xs bg-[#66023C] hover:bg-[#7A2048] text-white border-none"
                                    on:click=move |_| set_show_group_form.update(|v| *v = !*v)
                                >
                                    {move || if show_group_form.get() { "Abbrechen" } else { "+ Gruppenchat" }}
                                </button>
                            </div>

                            // Formular zum Erstellen eines neuen Gruppenchats
                            {move || show_group_form.get().then(|| view! {
                                <form
                                    on:submit=on_create_group_submit
                                    class="p-3 rounded-box bg-slate-50 border border-slate-200 space-y-3"
                                >
                                    <div>
                                        <label class="label py-0.5">
                                            <span class="label-text text-xs font-semibold">"Gruppenname"</span>
                                        </label>
                                        <input
                                            type="text"
                                            placeholder="z.B. Haushalt, Projekt, Familie..."
                                            class="input input-bordered input-xs sm:input-sm w-full"
                                            prop:value=move || group_name.get()
                                            on:input=move |ev| set_group_name.set(event_target_value(&ev))
                                            required
                                        />
                                    </div>

                                    <div>
                                        <div class="text-xs font-semibold mb-1 text-slate-600">
                                            "Teilnehmer auswählen:"
                                        </div>
                                        <div class="max-h-32 overflow-y-auto space-y-1 pr-1">
                                            {move || uebersicht_resource.get().and_then(|r| r.ok()).map(|ueb| {
                                                let my_id = ueb.aktueller_benutzer_id;
                                                ueb.alle_teilnehmer.into_iter().map(|t| {
                                                    let uid = t.id;
                                                    let is_self = uid == my_id;
                                                    let checked = move || is_self || selected_group_members.get().contains(&uid);
                                                    view! {
                                                        <label class="flex items-center gap-2 text-xs cursor-pointer py-0.5">
                                                            <input
                                                                type="checkbox"
                                                                class="checkbox checkbox-xs checkbox-primary"
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
                                                            <span>
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
                                        class="btn btn-primary btn-xs sm:btn-sm w-full"
                                        disabled=move || create_group_action.pending().get()
                                    >
                                        "Gruppenchat erstellen"
                                    </button>
                                </form>
                            })}

                            <Suspense fallback=move || view! {
                                <div class="text-xs text-slate-400 py-2">"Lade Chats..."</div>
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
                                                <div class="text-xs text-slate-400 italic py-1">
                                                    "Noch kein Gruppenchat vorhanden."
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <ul class="space-y-1">
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
                                                            let is_active = active_chat.get().map(|c| c.id == gid).unwrap_or(false);
                                                            if is_active {
                                                                "w-full text-left px-3 py-2 rounded-lg bg-[#66023C]/10 text-[#66023C] font-semibold transition-colors flex items-center justify-between"
                                                            } else {
                                                                "w-full text-left px-3 py-2 rounded-lg hover:bg-slate-100 text-slate-700 transition-colors flex items-center justify-between"
                                                            }
                                                        };

                                                        view! {
                                                            <li>
                                                                <button
                                                                    type="button"
                                                                    class=btn_class
                                                                    on:click=move |_| set_active_chat.set(Some(g_clone.clone()))
                                                                >
                                                                    <div class="min-w-0">
                                                                        <div class="text-sm truncate flex items-center gap-1.5">
                                                                            <span>"👥"</span>
                                                                            <span class="truncate">{g.name}</span>
                                                                        </div>
                                                                        <div class="text-[11px] text-slate-400 truncate">
                                                                            {member_names}
                                                                        </div>
                                                                    </div>
                                                                    <span class="badge badge-ghost badge-xs">
                                                                        {format!("{}", g.teilnehmer.len())}
                                                                    </span>
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

                        <div class="divider my-1"></div>

                        // 2. Bereich: Alle Chatteilnehmer (1:1 Direktchat starten & Übersicht der Teilnehmer)
                        <div class="space-y-2">
                            <div class="flex items-center justify-between">
                                <h2 class="text-xs font-bold uppercase tracking-wider text-slate-500">
                                    "Chatteilnehmer"
                                </h2>
                                <span class="text-[11px] text-slate-400">
                                    "Klicken für Direktchat"
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
                                            <ul class="space-y-1">
                                                {ueb.alle_teilnehmer.into_iter().map(|teilnehmer| {
                                                    let tid = teilnehmer.id;
                                                    let tname = teilnehmer.benutzername.clone();
                                                    let initial = tname
                                                        .chars()
                                                        .next()
                                                        .unwrap_or('?')
                                                        .to_uppercase()
                                                        .to_string();
                                                    let is_self = tid == my_id;

                                                    let is_in_active_chat = move || {
                                                        active_chat
                                                            .get()
                                                            .map(|c| c.teilnehmer.iter().any(|m| m.id == tid))
                                                            .unwrap_or(false)
                                                    };

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
                                                        if is_direct_partner() {
                                                            "w-full text-left px-3 py-2 rounded-lg bg-[#66023C]/10 text-[#66023C] font-semibold transition-colors flex items-center justify-between"
                                                        } else {
                                                            "w-full text-left px-3 py-2 rounded-lg hover:bg-slate-100 text-slate-700 transition-colors flex items-center justify-between"
                                                        }
                                                    };

                                                    view! {
                                                        <li>
                                                            <button
                                                                type="button"
                                                                class=row_class
                                                                on:click=move |_| {
                                                                    open_direct_action.dispatch(tid);
                                                                }
                                                            >
                                                                <div class="flex items-center gap-2.5 min-w-0">
                                                                    <div class="w-8 h-8 rounded-full bg-[#66023C] text-white flex items-center justify-center text-xs font-bold shrink-0">
                                                                        {initial}
                                                                    </div>
                                                                    <div class="truncate">
                                                                        <div class="text-sm truncate">
                                                                            {tname}
                                                                            {if is_self { " (Du)" } else { "" }}
                                                                        </div>
                                                                    </div>
                                                                </div>

                                                                {move || is_in_active_chat().then(|| view! {
                                                                    <span class="badge badge-xs bg-[#66023C]/15 text-[#66023C] border-none">
                                                                        "Im Chat"
                                                                    </span>
                                                                })}
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
                        if is_dragging.get() {
                            "lg:col-span-8 card bg-base-100 shadow-sm border-2 border-dashed border-[#66023C] bg-[#66023C]/5 flex flex-col transition-all"
                        } else {
                            "lg:col-span-8 card bg-base-100 shadow-sm border border-base-300 flex flex-col transition-all"
                        }
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
                    <div class="card-body p-4 sm:p-6 flex flex-col flex-1 justify-between space-y-4">
                        // Kopfbereich des ausgewählten Chats
                        <div class="flex flex-wrap items-center justify-between gap-2 pb-3 border-b border-slate-200">
                            {move || match active_chat.get() {
                                Some(chat) => {
                                    let teilnehmer_liste = chat
                                        .teilnehmer
                                        .iter()
                                        .map(|t| t.benutzername.clone())
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    view! {
                                        <div>
                                            <div class="flex items-center gap-2">
                                                <h2 class="text-base sm:text-lg font-bold text-[#66023C]">
                                                    {chat.name}
                                                </h2>
                                                <span class="badge badge-xs sm:badge-sm badge-ghost">
                                                    {if chat.ist_gruppe { "Gruppenchat" } else { "Direktchat" }}
                                                </span>
                                            </div>
                                            <p class="text-xs text-slate-500">
                                                {format!("Teilnehmer: {teilnehmer_liste}")}
                                            </p>
                                        </div>
                                        <button
                                            type="button"
                                            class="btn btn-ghost btn-xs text-slate-500"
                                            on:click=move |_| nachrichten_resource.refetch()
                                            title="Nachrichten aktualisieren"
                                        >
                                            "↻ Aktualisieren"
                                        </button>
                                    }.into_any()
                                }
                                None => view! {
                                    <div>
                                        <h2 class="text-base sm:text-lg font-semibold text-slate-500">
                                            "Kein Chat ausgewählt"
                                        </h2>
                                        <p class="text-xs text-slate-400">
                                            "Wähle links einen Chatteilnehmer oder erstelle einen Gruppenchat."
                                        </p>
                                    </div>
                                }.into_any(),
                            }}
                        </div>

                        // Drag & Drop Hinweis-Banner wenn Datei gezogen wird
                        {move || is_dragging.get().then(|| view! {
                            <div class="p-4 rounded-xl bg-[#66023C]/10 border border-[#66023C] text-[#66023C] text-center text-sm font-semibold">
                                "📂 Bilder oder Dokumente hier loslassen, um sie anzuhängen"
                            </div>
                        })}

                        // Nachrichtenliste
                        <div class="flex-1 overflow-y-auto space-y-3 max-h-[420px] min-h-[300px] pr-1">
                            <Suspense fallback=move || view! {
                                <div class="flex justify-center p-8">
                                    <span class="loading loading-spinner loading-md text-primary"></span>
                                </div>
                            }>
                                {move || {
                                    if active_chat.get().is_none() {
                                        return view! {
                                            <div class="h-full flex items-center justify-center text-slate-400 text-sm py-16 text-center">
                                                "Klicke links auf einen Chatteilnehmer oder Gruppenchat, um Nachrichten anzuzeigen."
                                            </div>
                                        }.into_any();
                                    }

                                    let my_id = uebersicht_resource
                                        .get()
                                        .and_then(|r| r.ok())
                                        .map(|u| u.aktueller_benutzer_id);

                                    nachrichten_resource.get().map(|res| match res {
                                        Ok(msgs) => {
                                            if msgs.is_empty() {
                                                view! {
                                                    <div class="h-full flex flex-col items-center justify-center text-slate-400 text-sm py-16 space-y-1 text-center">
                                                        <p class="font-medium">"Noch keine Nachrichten in diesem Chat."</p>
                                                        <p class="text-xs">"Schreibe eine Nachricht oder ziehe Bilder & Dokumente per Drag & Drop / Copy & Paste hierher."</p>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div class="space-y-3">
                                                        {msgs.into_iter().map(|m| {
                                                            let is_mine = my_id.map(|id| id == m.benutzer_id).unwrap_or(false);
                                                            let align_class = if is_mine {
                                                                "flex flex-col items-end"
                                                            } else {
                                                                "flex flex-col items-start"
                                                            };
                                                            let bubble_class = if is_mine {
                                                                "max-w-[85%] sm:max-w-[75%] rounded-2xl px-4 py-2.5 bg-[#66023C] text-white shadow-xs space-y-2"
                                                            } else {
                                                                "max-w-[85%] sm:max-w-[75%] rounded-2xl px-4 py-2.5 bg-slate-100 text-slate-800 border border-slate-200 shadow-xs space-y-2"
                                                            };

                                                            let is_image = m
                                                                .anhang_typ
                                                                .as_deref()
                                                                .map(|t| t.starts_with("image/"))
                                                                .unwrap_or(false);

                                                            view! {
                                                                <div class=align_class>
                                                                    <div class="text-[11px] text-slate-400 mb-0.5 px-1 flex items-center gap-1.5">
                                                                        <span class="font-semibold text-slate-600">{m.benutzername}</span>
                                                                        <span>"•"</span>
                                                                        <span>{m.zeitpunkt}</span>
                                                                    </div>
                                                                    <div class=bubble_class>
                                                                        {(!m.inhalt.is_empty()).then(|| view! {
                                                                            <p class="text-sm whitespace-pre-wrap break-words">{m.inhalt}</p>
                                                                        })}

                                                                        {m.anhang_daten.map(|daten| {
                                                                            let dateiname = m.anhang_name.unwrap_or_else(|| "Anhang".to_string());
                                                                            if is_image {
                                                                                view! {
                                                                                    <div class="space-y-1 pt-1">
                                                                                        <a href=daten.clone() target="_blank" title="Bild öffnen">
                                                                                            <img
                                                                                                src=daten.clone()
                                                                                                alt=dateiname.clone()
                                                                                                class="max-h-60 rounded-lg object-contain border border-white/20 bg-white"
                                                                                            />
                                                                                        </a>
                                                                                        <div class="flex items-center justify-between text-[11px] opacity-85">
                                                                                            <span class="truncate max-w-[180px]">{dateiname.clone()}</span>
                                                                                            <a
                                                                                                href=daten
                                                                                                download=dateiname
                                                                                                class="underline font-medium ml-2"
                                                                                            >
                                                                                                "Download"
                                                                                            </a>
                                                                                        </div>
                                                                                    </div>
                                                                                }.into_any()
                                                                            } else {
                                                                                view! {
                                                                                    <div class="flex items-center justify-between gap-3 p-2.5 rounded-xl bg-black/15 text-xs">
                                                                                        <div class="flex items-center gap-2 min-w-0">
                                                                                            <span class="text-base">"📄"</span>
                                                                                            <span class="font-medium truncate">{dateiname.clone()}</span>
                                                                                        </div>
                                                                                        <a
                                                                                            href=daten
                                                                                            download=dateiname
                                                                                            class="btn btn-xs bg-white text-[#66023C] hover:bg-slate-100 border-none shrink-0"
                                                                                        >
                                                                                            "Herunterladen"
                                                                                        </a>
                                                                                    </div>
                                                                                }.into_any()
                                                                            }
                                                                        })}
                                                                    </div>
                                                                </div>
                                                            }
                                                        }).collect_view()}
                                                    </div>
                                                }.into_any()
                                            }
                                        }
                                        Err(e) => view! {
                                            <div class="alert alert-error text-xs">
                                                <span>{format!("Fehler beim Laden der Nachrichten: {e}")}</span>
                                            </div>
                                        }.into_any(),
                                    }).unwrap_or_else(|| view! { <div></div> }.into_any())
                                }}
                            </Suspense>
                        </div>

                        // Vorschau-Leiste für angehängte Bilder & Dokumente (vor dem Senden)
                        {move || {
                            let list = pending_anhaenge.get();
                            (!list.is_empty()).then(|| view! {
                                <div class="flex flex-wrap gap-2 p-2.5 rounded-xl bg-slate-50 border border-slate-200">
                                    {list.into_iter().enumerate().map(|(idx, item)| {
                                        let is_img = item.typ.starts_with("image/");
                                        view! {
                                            <div class="flex items-center gap-2 bg-white px-2.5 py-1.5 rounded-lg border border-slate-200 shadow-2xs text-xs">
                                                {if is_img {
                                                    view! {
                                                        <img src=item.daten class="w-8 h-8 object-cover rounded" alt=item.name.clone() />
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <span class="text-base">"📎"</span>
                                                    }.into_any()
                                                }}
                                                <span class="max-w-[140px] truncate font-medium text-slate-700">
                                                    {item.name}
                                                </span>
                                                <button
                                                    type="button"
                                                    class="btn btn-ghost btn-xs text-error px-1"
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

                        // Eingabezeile (mit Copy & Paste, Drag & Drop und Datei-Auswahl-Button)
                        <form on:submit=on_send_submit class="pt-2 border-t border-slate-200 space-y-2">
                            <div class="flex items-center gap-2">
                                <label
                                    class="btn btn-outline btn-sm border-slate-300 text-slate-700 hover:bg-[#66023C] hover:text-white hover:border-[#66023C] cursor-pointer shrink-0"
                                    title="Bilder oder Dokumente auswählen"
                                >
                                    <span>"📎 Datei / Bild"</span>
                                    <input
                                        type="file"
                                        multiple
                                        class="hidden"
                                        on:change=on_file_input_change
                                    />
                                </label>

                                <input
                                    type="text"
                                    placeholder="Nachricht schreiben oder Bild/Dokument per Cmd+V einfügen..."
                                    class="input input-bordered input-sm flex-1"
                                    prop:value=move || nachricht_text.get()
                                    on:input=move |ev| set_nachricht_text.set(event_target_value(&ev))
                                    on:paste=on_paste_files
                                    disabled=move || active_chat.get().is_none()
                                />

                                <button
                                    type="submit"
                                    class="btn btn-primary btn-sm shrink-0"
                                    disabled=move || active_chat.get().is_none() || send_action.pending().get()
                                >
                                    "Senden"
                                </button>
                            </div>
                            <div class="text-[11px] text-slate-400 flex items-center justify-between px-1">
                                <span>"Tipp: Bilder & Dokumente einfach per Drag & Drop in das Chatfenster ziehen oder per Copy & Paste (Cmd+V / Strg+V) einfügen."</span>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        </section>
    }
}
