use chrono::{Datelike, Local};
use leptos::prelude::*;

use crate::components::modal::Modal;
use crate::backend::budget::{
    add_fixkosten, delete_fixkosten, get_fixkosten, kuendigungsfrist_text,
    update_fixkosten, update_fixkosten_status, FixkostenEintrag, FixkostenStatus,
    Zahlungsperiode,
};

/// Auswahlmöglichkeiten für die Kündigungsfrist (in Monaten)
const KUENDIGUNGSFRISTEN: [i32; 7] = [0, 1, 2, 3, 6, 12, 24];

fn status_select_class(status: FixkostenStatus) -> &'static str {
    match status {
        FixkostenStatus::Aktiv => "select select-xs select-bordered text-success font-medium",
        FixkostenStatus::Gekuendigt => "select select-xs select-bordered text-black font-medium",
        FixkostenStatus::Beendet => "select select-xs select-bordered text-base-content/50 font-medium",
    }
}

/// Heutiges Datum im Format JJJJ-MM-TT (für das Datumsfeld).
fn heute_iso() -> String {
    Local::now().date_naive().format("%Y-%m-%d").to_string()
}

#[component]
pub fn FixkostenSection() -> impl IntoView {
    let today = Local::now().date_naive();

    let (buchungstext, set_buchungstext) = signal(String::new());
    let (betrag_input, set_betrag_input) = signal(String::new());
    let (beginndatum, set_beginndatum) = signal(heute_iso());
    let (kuendigungsfrist, set_kuendigungsfrist) = signal(0_i32);
    let (zahlungsperiode, set_zahlungsperiode) = signal(Zahlungsperiode::Monatlich);
    let (status, set_status) = signal(FixkostenStatus::Aktiv);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (modal_offen, set_modal_offen) = signal(false);
    // Id des Vertrags, der gerade bearbeitet wird (None = neuer Vertrag)
    let (bearbeiten_id, set_bearbeiten_id) = signal(Option::<uuid::Uuid>::None);

    let fixkosten_resource = Resource::new(|| (), |_| get_fixkosten());

    let create_action = Action::new(
        move |(text, betrag, beginn, frist, periode, status): &(
            String,
            f64,
            String,
            i32,
            Zahlungsperiode,
            FixkostenStatus,
        )| {
            let (text, betrag, beginn, frist, periode, status) =
                (text.clone(), *betrag, beginn.clone(), *frist, *periode, *status);
            async move { add_fixkosten(text, betrag, beginn, frist, periode, status).await }
        },
    );

    let update_action = Action::new(
        move |(id, text, betrag, beginn, frist, periode, status): &(
            uuid::Uuid,
            String,
            f64,
            String,
            i32,
            Zahlungsperiode,
            FixkostenStatus,
        )| {
            let (id, text, betrag, beginn, frist, periode, status) = (
                *id,
                text.clone(),
                *betrag,
                beginn.clone(),
                *frist,
                *periode,
                *status,
            );
            async move {
                update_fixkosten(id, text, betrag, beginn, frist, periode, status).await
            }
        },
    );

    let status_action = Action::new(move |(id, status): &(uuid::Uuid, FixkostenStatus)| {
        let (id, status) = (*id, *status);
        async move { update_fixkosten_status(id, status).await }
    });

    let delete_action = Action::new(move |id: &uuid::Uuid| {
        let id = *id;
        async move { delete_fixkosten(id).await }
    });

    // Formular leeren und schliessen (nach Erfolg)
    let formular_schliessen = move || {
        set_buchungstext.set(String::new());
        set_betrag_input.set(String::new());
        set_error_msg.set(None);
        set_bearbeiten_id.set(None);
        set_modal_offen.set(false);
    };

    Effect::new(move |_| {
        if let Some(res) = create_action.value().get() {
            match res {
                Ok(_) => {
                    formular_schliessen();
                    fixkosten_resource.refetch();
                }
                Err(e) => set_error_msg.set(Some(e.to_string())),
            }
        }
    });

    Effect::new(move |_| {
        if let Some(res) = update_action.value().get() {
            match res {
                Ok(()) => {
                    formular_schliessen();
                    fixkosten_resource.refetch();
                }
                Err(e) => set_error_msg.set(Some(e.to_string())),
            }
        }
    });

    Effect::new(move |_| {
        if let Some(res) = status_action.value().get() {
            match res {
                Ok(()) => fixkosten_resource.refetch(),
                Err(e) => set_error_msg.set(Some(e.to_string())),
            }
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(())) = delete_action.value().get() {
            fixkosten_resource.refetch();
        }
    });

    // Öffnet das Formular mit den Werten eines bestehenden Vertrags (Doppelklick oder ✎)
    let bearbeiten_oeffnen = move |f: FixkostenEintrag| {
        set_error_msg.set(None);
        set_bearbeiten_id.set(Some(f.id));
        set_buchungstext.set(f.buchungstext.clone());
        set_betrag_input.set(format!("{:.2}", f.betrag));
        set_beginndatum.set(f.beginndatum.format("%Y-%m-%d").to_string());
        set_kuendigungsfrist.set(f.kuendigungsfrist_monate);
        set_zahlungsperiode.set(f.zahlungsperiode);
        set_status.set(f.status);
        set_modal_offen.set(true);
    };

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let text = buchungstext.get();
        let raw_amount = betrag_input.get().replace(',', ".");
        match raw_amount.trim().parse::<f64>() {
            Ok(val) if val > 0.0 => {
                set_error_msg.set(None);
                match bearbeiten_id.get() {
                    Some(id) => {
                        update_action.dispatch((
                            id,
                            text,
                            val,
                            beginndatum.get(),
                            kuendigungsfrist.get(),
                            zahlungsperiode.get(),
                            status.get(),
                        ));
                    }
                    None => {
                        create_action.dispatch((
                            text,
                            val,
                            beginndatum.get(),
                            kuendigungsfrist.get(),
                            zahlungsperiode.get(),
                            status.get(),
                        ));
                    }
                }
            }
            _ => {
                set_error_msg.set(Some(
                    "Bitte einen gültigen positiven Betrag in CHF eingeben (z.B. 950.00)."
                        .to_string(),
                ));
            }
        }
    };

    view! {
        <section class="card bg-base-100 shadow-sm border border-base-300">
            <div class="card-body p-4 sm:p-6 space-y-6">
                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                    <div>
                        <h2 class="card-title text-lg sm:text-xl">"Fixkosten"</h2>
                        <p class="text-xs sm:text-sm text-base-content/70">
                            "Erfassung wiederkehrender Fixkosten in CHF (Miete, Krankenkasse, Abos) mit Vertragsdaten"
                        </p>
                    </div>
                    <button
                        type="button"
                        class="btn btn-secondary self-start sm:self-auto"
                        on:click=move |_| {
                            set_bearbeiten_id.set(None);
                            set_error_msg.set(None);
                            set_buchungstext.set(String::new());
                            set_betrag_input.set(String::new());
                            set_beginndatum.set(heute_iso());
                            set_kuendigungsfrist.set(0);
                            set_zahlungsperiode.set(Zahlungsperiode::Monatlich);
                            set_status.set(FixkostenStatus::Aktiv);
                            set_modal_offen.set(true);
                        }
                    >
                        "+ Neue Fixkosten"
                    </button>
                </div>

                // Modalfenster zum Erfassen und Bearbeiten von Fixkosten
                <Modal
                    offen=modal_offen
                    on_close=Callback::new(move |_| set_modal_offen.set(false))
                    titel=move || if bearbeiten_id.get().is_some() { "Fixkosten bearbeiten".to_string() } else { "Neue Fixkosten".to_string() }
                >
                    <form on:submit=on_submit class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <div class="sm:col-span-2">
                            <label class="label py-1">
                                <span class="label-text font-medium text-sm">"Buchungstext"</span>
                            </label>
                            <input
                                type="text"
                                placeholder="z.B. Miete, Krankenkasse, Strom..."
                                class="input input-bordered w-full"
                                prop:value=move || buchungstext.get()
                                on:input=move |ev| set_buchungstext.set(event_target_value(&ev))
                                required
                            />
                        </div>

                        <div>
                            <label class="label py-1">
                                <span class="label-text font-medium text-sm">"Betrag pro Periode (CHF)"</span>
                            </label>
                            <input
                                type="text"
                                inputmode="decimal"
                                placeholder="0.00"
                                class="input input-bordered w-full"
                                prop:value=move || betrag_input.get()
                                on:input=move |ev| set_betrag_input.set(event_target_value(&ev))
                                required
                            />
                        </div>

                        <div>
                            <label class="label py-1">
                                <span class="label-text font-medium text-sm">"Zahlungsperiode"</span>
                            </label>
                            <select
                                class="select select-bordered w-full"
                                on:change=move |ev| set_zahlungsperiode.set(Zahlungsperiode::from_db_str(&event_target_value(&ev)))
                            >
                                {Zahlungsperiode::ALLE.into_iter().map(|p| view! {
                                    <option value=p.as_str() selected=move || zahlungsperiode.get() == p>{p.label()}</option>
                                }).collect_view()}
                            </select>
                        </div>

                        <div>
                            <label class="label py-1">
                                <span class="label-text font-medium text-sm">"Beginndatum"</span>
                            </label>
                            <input
                                type="date"
                                class="input input-bordered w-full"
                                prop:value=move || beginndatum.get()
                                on:input=move |ev| set_beginndatum.set(event_target_value(&ev))
                                required
                            />
                        </div>

                        <div>
                            <label class="label py-1">
                                <span class="label-text font-medium text-sm">"Kündigungsfrist"</span>
                            </label>
                            <select
                                class="select select-bordered w-full"
                                on:change=move |ev| set_kuendigungsfrist.set(event_target_value(&ev).parse().unwrap_or(0))
                            >
                                {KUENDIGUNGSFRISTEN.into_iter().map(|m| view! {
                                    <option value=m.to_string() selected=move || kuendigungsfrist.get() == m>{kuendigungsfrist_text(m)}</option>
                                }).collect_view()}
                            </select>
                        </div>

                        <div>
                            <label class="label py-1">
                                <span class="label-text font-medium text-sm">"Status"</span>
                            </label>
                            <select
                                class="select select-bordered w-full"
                                on:change=move |ev| set_status.set(FixkostenStatus::from_db_str(&event_target_value(&ev)))
                            >
                                {FixkostenStatus::ALLE.into_iter().map(|s| view! {
                                    <option value=s.as_str() selected=move || status.get() == s>{s.label()}</option>
                                }).collect_view()}
                            </select>
                        </div>

                        {move || error_msg.get().map(|msg| view! {
                            <div class="alert alert-warning py-2 text-sm sm:col-span-2">
                                <span>{msg}</span>
                            </div>
                        })}

                        <div class="modal-action sm:col-span-2 mt-2">
                            <button type="button" class="btn btn-ghost" on:click=move |_| set_modal_offen.set(false)>
                                "Abbrechen"
                            </button>
                            <button
                                type="submit"
                                class="btn btn-secondary"
                                disabled=move || create_action.pending().get() || update_action.pending().get()
                            >
                                {move || if bearbeiten_id.get().is_some() { "Speichern" } else { "Hinzufügen" }}
                            </button>
                        </div>
                    </form>
                </Modal>

                // Fehler beim Ändern des Status (ausserhalb des Modalfensters)
                {move || (!modal_offen.get()).then(|| error_msg.get()).flatten().map(|msg| view! {
                    <div class="alert alert-warning py-2 text-sm">
                        <span>{msg}</span>
                    </div>
                })}

                // Tabelle Fixkosten
                <div class="overflow-x-auto -mx-2 sm:mx-0">
                    <Suspense fallback=move || view! {
                        <div class="flex justify-center p-8">
                            <span class="loading loading-spinner loading-md text-secondary"></span>
                        </div>
                    }>
                        {move || {
                            fixkosten_resource.get().map(|res| match res {
                                Ok(items) => {
                                    // Summe pro Monat: nur Verträge, die im aktuellen Monat Kosten verursachen
                                    let summe_pro_monat: f64 = items
                                        .iter()
                                        .filter(|f| f.zaehlt_im_monat(today.year(), today.month()))
                                        .map(FixkostenEintrag::monatsbetrag)
                                        .sum();

                                    view! {
                                        <table class="table table-zebra table-sm sm:table-md w-full">
                                            <thead>
                                                <tr class="text-xs sm:text-sm uppercase text-base-content/70">
                                                    <th>"Buchungstext"</th>
                                                    <th>"Beginn"</th>
                                                    <th>"Periode"</th>
                                                    <th>"Kündigungsfrist"</th>
                                                    <th>"Status"</th>
                                                    <th class="text-right">"Betrag (CHF)"</th>
                                                    <th class="w-20 text-right"></th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {if items.is_empty() {
                                                    view! {
                                                        <tr>
                                                            <td colspan="7" class="text-center text-base-content/50 py-6">
                                                                "Noch keine Fixkosten erfasst."
                                                            </td>
                                                        </tr>
                                                    }.into_any()
                                                } else {
                                                    items.into_iter().map(|f| {
                                                        let id = f.id;
                                                        let aktueller_status = f.status;
                                                        let monatsbetrag = f.monatsbetrag();
                                                        let ist_monatlich = f.zahlungsperiode == Zahlungsperiode::Monatlich;
                                                        let zeilen_class = if aktueller_status == FixkostenStatus::Beendet {
                                                            "opacity-60"
                                                        } else {
                                                            ""
                                                        };
                                                        let bearbeiten = f.clone();
                                                        let zeile_dbl = f.clone();
                                                        view! {
                                                            <tr class=format!("{zeilen_class} select-none") title="Doppelklick zum Bearbeiten" on:dblclick=move |_| bearbeiten_oeffnen(zeile_dbl.clone())>
                                                                <td class="font-medium">{f.buchungstext}</td>
                                                                <td class="whitespace-nowrap font-mono text-xs sm:text-sm text-base-content/80">
                                                                    {f.beginndatum.format("%d.%m.%Y").to_string()}
                                                                </td>
                                                                <td class="whitespace-nowrap">{f.zahlungsperiode.label()}</td>
                                                                <td class="whitespace-nowrap">{kuendigungsfrist_text(f.kuendigungsfrist_monate)}</td>
                                                                <td>
                                                                    <select
                                                                        class=status_select_class(aktueller_status)
                                                                        aria-label="Status ändern"
                                                                        on:dblclick=move |ev| { ev.stop_propagation(); }
                                                                        disabled=move || status_action.pending().get()
                                                                        on:change=move |ev| {
                                                                            let neu = FixkostenStatus::from_db_str(&event_target_value(&ev));
                                                                            if neu != aktueller_status {
                                                                                status_action.dispatch((id, neu));
                                                                            }
                                                                        }
                                                                    >
                                                                        {FixkostenStatus::ALLE.into_iter().map(|s| view! {
                                                                            <option value=s.as_str() selected=s == aktueller_status>{s.label()}</option>
                                                                        }).collect_view()}
                                                                    </select>
                                                                </td>
                                                                <td class="text-right whitespace-nowrap">
                                                                    <div class="font-semibold text-black">
                                                                        {format!("CHF {:.2}", f.betrag)}
                                                                    </div>
                                                                    {(!ist_monatlich).then(|| view! {
                                                                        <div class="text-xs text-base-content/60">
                                                                            {format!("≈ CHF {:.2} / Monat", monatsbetrag)}
                                                                        </div>
                                                                    })}
                                                                </td>
                                                                <td class="text-right whitespace-nowrap">
                                                                    <button
                                                                        type="button"
                                                                        class="btn btn-ghost"
                                                                        title="Fixkosten bearbeiten"
                                                                        aria-label="Fixkosten bearbeiten"
                                                                        on:click=move |_| bearbeiten_oeffnen(bearbeiten.clone())
                                                                    >
                                                                        "✎"
                                                                    </button>
                                                                    <button
                                                                        type="button"
                                                                        class="btn btn-ghost text-error"
                                                                        title="Fixkosten löschen"
                                                                        on:dblclick=move |ev| { ev.stop_propagation(); }
                                                                        on:click=move |_| { delete_action.dispatch(id); }
                                                                    >
                                                                        "✕"
                                                                    </button>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }).collect_view().into_any()
                                                }}
                                            </tbody>
                                            <tfoot>
                                                <tr class="font-bold text-sm sm:text-base bg-base-200/50">
                                                    <td colspan="5">
                                                        <div class="flex flex-col">
                                                            <span>"Summe Fixkosten pro Monat"</span>
                                                            <span class="text-xs font-normal text-base-content/70">
                                                                "Aktive und gekündigte Verträge, auf den Monat umgerechnet"
                                                            </span>
                                                        </div>
                                                    </td>
                                                    <td class="text-right text-black whitespace-nowrap">
                                                        {format!("CHF {:.2}", summe_pro_monat)}
                                                    </td>
                                                    <td></td>
                                                </tr>
                                            </tfoot>
                                        </table>
                                    }.into_any()
                                }
                                Err(err) => view! {
                                    <div class="alert alert-error">
                                        <span>{format!("Fehler beim Laden der Fixkosten: {err}")}</span>
                                    </div>
                                }.into_any(),
                            })
                        }}
                    </Suspense>
                </div>
            </div>
        </section>
    }
}
