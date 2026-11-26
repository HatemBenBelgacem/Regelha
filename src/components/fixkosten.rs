use chrono::{Datelike, Local, NaiveDate};
use leptos::prelude::*;
use uuid::Uuid;

use crate::backend::budget::{
    add_fixkosten, delete_fixkosten, filtere_fixkosten, get_fixkosten, kuendigungsfrist_text,
    update_fixkosten, update_fixkosten_status, FixkostenArt, FixkostenDaten, FixkostenEintrag,
    FixkostenFilter, FixkostenStatus, KategorieFilter, Zahlungsperiode,
};
use crate::backend::kategorien::{get_kategorien, kategorie_name};
use crate::components::modal::Modal;

/// Auswahlmöglichkeiten für die Kündigungsfrist (in Monaten)
const KUENDIGUNGSFRISTEN: [i32; 7] = [0, 1, 2, 3, 6, 12, 24];

fn status_select_class(status: FixkostenStatus) -> &'static str {
    match status {
        FixkostenStatus::Aktiv => "select select-xs select-bordered text-success font-medium",
        FixkostenStatus::Gekuendigt => "select select-xs select-bordered text-black font-medium",
        FixkostenStatus::Beendet => "select select-xs select-bordered text-base-content/50 font-medium",
    }
}

/// Status-Filter aus dem Auswahlfeld (leerer Wert = alle Status).
fn status_filter_aus_wert(wert: &str) -> Option<FixkostenStatus> {
    FixkostenStatus::ALLE
        .into_iter()
        .find(|s| s.as_str() == wert)
}

/// Kategorie-Filter aus dem Auswahlfeld (leer = alle, "ohne" = ohne Kategorie, sonst die Id).
fn kategorie_filter_aus_wert(wert: &str) -> KategorieFilter {
    match wert {
        "" => KategorieFilter::Alle,
        "ohne" => KategorieFilter::OhneKategorie,
        _ => Uuid::parse_str(wert)
            .map(KategorieFilter::Kategorie)
            .unwrap_or_default(),
    }
}

/// Wert eines Datumsfelds im Format JJJJ-MM-TT (leer, wenn kein Datum gesetzt ist).
fn datum_feld_wert(datum: Option<NaiveDate>) -> String {
    datum
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

/// Heutiges Datum im Format JJJJ-MM-TT (für das Datumsfeld).
fn heute_iso() -> String {
    Local::now().date_naive().format("%Y-%m-%d").to_string()
}

/// Überschrift des Modalfensters, je nach Art und ob ein Eintrag bearbeitet wird.
fn modal_titel(art: FixkostenArt, bearbeiten: bool) -> String {
    match (art, bearbeiten) {
        (FixkostenArt::Ausgabe, false) => "Neue Fixkosten".to_string(),
        (FixkostenArt::Ausgabe, true) => "Fixkosten bearbeiten".to_string(),
        (FixkostenArt::Einnahme, false) => "Neue Fixeinnahme".to_string(),
        (FixkostenArt::Einnahme, true) => "Fixeinnahme bearbeiten".to_string(),
    }
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
    let (art, set_art) = signal(FixkostenArt::Ausgabe);
    let (kategorie, set_kategorie) = signal(Option::<Uuid>::None);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (modal_offen, set_modal_offen) = signal(false);
    // Id des Eintrags, der gerade bearbeitet wird (None = neuer Eintrag)
    let (bearbeiten_id, set_bearbeiten_id) = signal(Option::<Uuid>::None);
    // Zweiter Klick auf "Löschen" bestätigt das Löschen
    let (loesch_bestaetigen, set_loesch_bestaetigen) = signal(false);
    // Filter für die Tabelle: Kategorie, Beginn (von/bis) und Status
    let (filter, set_filter) = signal(FixkostenFilter::default());

    let fixkosten_resource = Resource::new(|| (), |_| get_fixkosten());
    let kategorien_resource = Resource::new(|| (), |_| get_kategorien());
    // Kategorien für Auswahlfeld und Anzeige. Solange sie nicht geladen sind, ist die Liste leer.
    let kategorien = move || kategorien_resource.get().and_then(|r| r.ok()).unwrap_or_default();

    let create_action = Action::new(move |daten: &FixkostenDaten| {
        let daten = daten.clone();
        async move { add_fixkosten(daten).await }
    });

    let update_action = Action::new(move |(id, daten): &(Uuid, FixkostenDaten)| {
        let (id, daten) = (*id, daten.clone());
        async move { update_fixkosten(id, daten).await }
    });

    let status_action = Action::new(move |(id, status): &(Uuid, FixkostenStatus)| {
        let (id, status) = (*id, *status);
        async move { update_fixkosten_status(id, status).await }
    });

    let delete_action = Action::new(move |id: &Uuid| {
        let id = *id;
        async move { delete_fixkosten(id).await }
    });

    // Formular leeren und schliessen (nach Erfolg)
    let formular_schliessen = move || {
        set_buchungstext.set(String::new());
        set_betrag_input.set(String::new());
        set_error_msg.set(None);
        set_bearbeiten_id.set(None);
        set_loesch_bestaetigen.set(false);
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
        if let Some(res) = delete_action.value().get() {
            match res {
                Ok(()) => {
                    formular_schliessen();
                    fixkosten_resource.refetch();
                }
                Err(e) => {
                    set_loesch_bestaetigen.set(false);
                    set_error_msg.set(Some(e.to_string()));
                }
            }
        }
    });

    // Öffnet die Eingabemaske mit den Werten eines bestehenden Eintrags (Doppelklick auf die Zeile)
    let bearbeiten_oeffnen = move |f: FixkostenEintrag| {
        set_error_msg.set(None);
        set_loesch_bestaetigen.set(false);
        set_bearbeiten_id.set(Some(f.id));
        set_art.set(f.art);
        set_buchungstext.set(f.buchungstext.clone());
        set_betrag_input.set(format!("{:.2}", f.betrag));
        set_beginndatum.set(f.beginndatum.format("%Y-%m-%d").to_string());
        set_kuendigungsfrist.set(f.kuendigungsfrist_monate);
        set_zahlungsperiode.set(f.zahlungsperiode);
        set_status.set(f.status);
        set_kategorie.set(f.kategorie_id);
        set_modal_offen.set(true);
    };

    // Öffnet ein leeres Formular für eine neue Fixkosten- oder Fixeinnahme-Position
    let neu_oeffnen = move |neue_art: FixkostenArt| {
        set_bearbeiten_id.set(None);
        set_error_msg.set(None);
        set_loesch_bestaetigen.set(false);
        set_buchungstext.set(String::new());
        set_betrag_input.set(String::new());
        set_art.set(neue_art);
        set_beginndatum.set(heute_iso());
        set_kuendigungsfrist.set(0);
        set_zahlungsperiode.set(Zahlungsperiode::Monatlich);
        set_status.set(FixkostenStatus::Aktiv);
        set_kategorie.set(None);
        set_modal_offen.set(true);
    };

    // Erster Klick fragt nach, zweiter Klick löscht
    let loeschen_klick = move |id: Uuid| {
        if loesch_bestaetigen.get() {
            delete_action.dispatch(id);
        } else {
            set_loesch_bestaetigen.set(true);
        }
    };

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let raw_amount = betrag_input.get().replace(',', ".");
        match raw_amount.trim().parse::<f64>() {
            Ok(val) if val > 0.0 => {
                set_error_msg.set(None);
                let daten = FixkostenDaten {
                    buchungstext: buchungstext.get(),
                    betrag: val,
                    beginndatum: beginndatum.get(),
                    kuendigungsfrist_monate: kuendigungsfrist.get(),
                    zahlungsperiode: zahlungsperiode.get(),
                    status: status.get(),
                    art: art.get(),
                    kategorie_id: kategorie.get(),
                };
                match bearbeiten_id.get() {
                    Some(id) => {
                        update_action.dispatch((id, daten));
                    }
                    None => {
                        create_action.dispatch(daten);
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
                        <h2 class="card-title text-lg sm:text-xl">"Fixkosten & Fixeinnahmen"</h2>
                        <p class="text-xs sm:text-sm text-base-content/70">
                            "Wiederkehrende Kosten (Miete, Krankenkasse, Abos) und Einnahmen (Lohn, Mieteinnahmen) in CHF mit Vertragsdaten und Kategorie"
                        </p>
                    </div>
                    <div class="flex flex-wrap gap-2 self-start sm:self-auto">
                        <button
                            type="button"
                            class="btn btn-secondary"
                            on:click=move |_| neu_oeffnen(FixkostenArt::Ausgabe)
                        >
                            "+ Neue Fixkosten"
                        </button>
                        <button
                            type="button"
                            class="btn btn-outline btn-secondary"
                            on:click=move |_| neu_oeffnen(FixkostenArt::Einnahme)
                        >
                            "+ Neue Fixeinnahme"
                        </button>
                    </div>
                </div>

                // Modalfenster zum Erfassen und Bearbeiten von Fixkosten und Fixeinnahmen
                <Modal
                    offen=modal_offen
                    on_close=Callback::new(move |_| set_modal_offen.set(false))
                    titel=move || modal_titel(art.get(), bearbeiten_id.get().is_some())
                >
                    <form on:submit=on_submit class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <div>
                            <label class="label py-1">
                                <span class="label-text font-medium text-sm">"Art"</span>
                            </label>
                            <select
                                class="select select-bordered w-full"
                                on:change=move |ev| set_art.set(FixkostenArt::from_db_str(&event_target_value(&ev)))
                            >
                                {FixkostenArt::ALLE.into_iter().map(|a| view! {
                                    <option value=a.as_str() selected=move || art.get() == a>{a.label()}</option>
                                }).collect_view()}
                            </select>
                        </div>

                        <div>
                            <label class="label py-1">
                                <span class="label-text font-medium text-sm">"Kategorie"</span>
                            </label>
                            <select
                                class="select select-bordered w-full"
                                on:change=move |ev| set_kategorie.set(Uuid::parse_str(&event_target_value(&ev)).ok())
                            >
                                <option value="" selected=move || kategorie.get().is_none()>"Ohne Kategorie"</option>
                                {move || kategorien().into_iter().map(|k| {
                                    let id = k.id;
                                    view! {
                                        <option value=id.to_string() selected=move || kategorie.get() == Some(id)>{k.name}</option>
                                    }
                                }).collect_view()}
                            </select>
                            {move || match kategorien_resource.get() {
                                Some(Ok(liste)) if liste.is_empty() => view! {
                                    <p class="text-xs text-base-content/60 mt-1">
                                        "Noch keine Kategorien. Du kannst sie unter Einstellungen anlegen."
                                    </p>
                                }.into_any(),
                                _ => ().into_any(),
                            }}
                        </div>

                        <div class="sm:col-span-2">
                            <label class="label py-1">
                                <span class="label-text font-medium text-sm">"Buchungstext"</span>
                            </label>
                            <input
                                type="text"
                                placeholder="z.B. Miete, Krankenkasse, Strom oder Lohn..."
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
                            {move || bearbeiten_id.get().map(|id| view! {
                                <button
                                    type="button"
                                    class="btn btn-outline btn-error mr-auto"
                                    disabled=move || delete_action.pending().get()
                                    on:click=move |_| loeschen_klick(id)
                                >
                                    {move || if loesch_bestaetigen.get() { "Wirklich löschen?" } else { "Löschen" }}
                                </button>
                            })}
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

                // Filter: Kategorie, Beginn (von/bis) und Status
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-3 items-end">
                    <div>
                        <label class="label py-1">
                            <span class="label-text font-medium text-xs">"Kategorie"</span>
                        </label>
                        <select
                            class="select select-bordered select-sm w-full"
                            aria-label="Filter nach Kategorie"
                            on:change=move |ev| set_filter.update(|f| f.kategorie = kategorie_filter_aus_wert(&event_target_value(&ev)))
                        >
                            <option value="" selected=move || filter.get().kategorie == KategorieFilter::Alle>"Alle Kategorien"</option>
                            <option value="ohne" selected=move || filter.get().kategorie == KategorieFilter::OhneKategorie>"Ohne Kategorie"</option>
                            {move || kategorien().into_iter().map(|k| {
                                let id = k.id;
                                view! {
                                    <option value=id.to_string() selected=move || filter.get().kategorie == KategorieFilter::Kategorie(id)>{k.name}</option>
                                }
                            }).collect_view()}
                        </select>
                    </div>

                    <div>
                        <label class="label py-1">
                            <span class="label-text font-medium text-xs">"Beginn ab"</span>
                        </label>
                        <input
                            type="date"
                            aria-label="Filter: Beginn ab"
                            class="input input-bordered input-sm w-full"
                            prop:value=move || datum_feld_wert(filter.get().beginn_von)
                            on:input=move |ev| set_filter.update(|f| f.beginn_von = NaiveDate::parse_from_str(&event_target_value(&ev), "%Y-%m-%d").ok())
                        />
                    </div>

                    <div>
                        <label class="label py-1">
                            <span class="label-text font-medium text-xs">"Beginn bis"</span>
                        </label>
                        <input
                            type="date"
                            aria-label="Filter: Beginn bis"
                            class="input input-bordered input-sm w-full"
                            prop:value=move || datum_feld_wert(filter.get().beginn_bis)
                            on:input=move |ev| set_filter.update(|f| f.beginn_bis = NaiveDate::parse_from_str(&event_target_value(&ev), "%Y-%m-%d").ok())
                        />
                    </div>

                    <div>
                        <label class="label py-1">
                            <span class="label-text font-medium text-xs">"Status"</span>
                        </label>
                        <select
                            class="select select-bordered select-sm w-full"
                            aria-label="Filter nach Status"
                            on:change=move |ev| set_filter.update(|f| f.status = status_filter_aus_wert(&event_target_value(&ev)))
                        >
                            <option value="" selected=move || filter.get().status.is_none()>"Alle Status"</option>
                            {FixkostenStatus::ALLE.into_iter().map(|s| view! {
                                <option value=s.as_str() selected=move || filter.get().status == Some(s)>{s.label()}</option>
                            }).collect_view()}
                        </select>
                    </div>

                    <div>
                        <button
                            type="button"
                            class="btn btn-ghost btn-sm w-full"
                            disabled=move || !filter.get().ist_aktiv()
                            on:click=move |_| set_filter.set(FixkostenFilter::default())
                        >
                            "Filter zurücksetzen"
                        </button>
                    </div>
                </div>

                // Tabelle Fixkosten und Fixeinnahmen
                <div class="overflow-x-auto -mx-2 sm:mx-0">
                    <Suspense fallback=move || view! {
                        <div class="flex justify-center p-8">
                            <span class="loading loading-spinner loading-md text-secondary"></span>
                        </div>
                    }>
                        {move || {
                            fixkosten_resource.get().map(|res| match res {
                                Ok(alle) => {
                                    // Nur die Einträge, die zum Filter passen. Die Summen unten beziehen sich darauf.
                                    let filter_aktiv = filter.get().ist_aktiv();
                                    let items = filtere_fixkosten(&alle, &filter.get());
                                    let anzahl_hinweis = filter_aktiv.then(|| format!(
                                        "{} von {} Einträgen angezeigt. Die Summen unten beziehen sich nur auf diese Einträge.",
                                        items.len(),
                                        alle.len()
                                    ));

                                    // Summen pro Monat: nur Verträge, die im aktuellen Monat anfallen
                                    let summe_kosten: f64 = items
                                        .iter()
                                        .filter(|f| !f.ist_einnahme() && f.zaehlt_im_monat(today.year(), today.month()))
                                        .map(FixkostenEintrag::monatsbetrag)
                                        .sum();
                                    let summe_einnahmen: f64 = items
                                        .iter()
                                        .filter(|f| f.ist_einnahme() && f.zaehlt_im_monat(today.year(), today.month()))
                                        .map(FixkostenEintrag::monatsbetrag)
                                        .sum();
                                    let differenz = summe_einnahmen - summe_kosten;
                                    let differenz_text = if differenz >= 0.0 {
                                        format!("+ CHF {:.2}", differenz)
                                    } else {
                                        format!("- CHF {:.2}", differenz.abs())
                                    };
                                    let differenz_klasse = if differenz >= 0.0 {
                                        "text-right whitespace-nowrap text-success"
                                    } else {
                                        "text-right whitespace-nowrap text-error"
                                    };

                                    let kategorien_liste = kategorien();

                                    view! {
                                        {anzahl_hinweis.map(|text| view! {
                                            <p class="text-xs text-base-content/60 mb-2">{text}</p>
                                        })}
                                        <table class="table table-zebra table-xs w-full">
                                            <thead>
                                                <tr class="text-xs uppercase text-base-content/70">
                                                    <th>"Buchungstext"</th>
                                                    <th>"Beginn"</th>
                                                    <th>"Periode"</th>
                                                    <th>"Kündigungsfrist"</th>
                                                    <th>"Status"</th>
                                                    <th class="text-right">"Betrag (CHF)"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {if items.is_empty() {
                                                    let leer_text = if alle.is_empty() {
                                                        "Noch keine Fixkosten oder Fixeinnahmen erfasst."
                                                    } else {
                                                        "Keine Einträge passen zum gewählten Filter."
                                                    };
                                                    view! {
                                                        <tr>
                                                            <td colspan="6" class="text-center text-base-content/50 py-6">
                                                                {leer_text}
                                                            </td>
                                                        </tr>
                                                    }.into_any()
                                                } else {
                                                    items.into_iter().map(|f| {
                                                        let id = f.id;
                                                        let aktueller_status = f.status;
                                                        let monatsbetrag = f.monatsbetrag();
                                                        let ist_monatlich = f.zahlungsperiode == Zahlungsperiode::Monatlich;
                                                        let ist_einnahme = f.ist_einnahme();
                                                        let kategorie_text = kategorie_name(&kategorien_liste, f.kategorie_id);
                                                        let zeilen_class = if aktueller_status == FixkostenStatus::Beendet {
                                                            "opacity-60"
                                                        } else {
                                                            ""
                                                        };
                                                        let (betrag_text, betrag_klasse) = if ist_einnahme {
                                                            (format!("+ CHF {:.2}", f.betrag), "font-semibold text-success")
                                                        } else {
                                                            (format!("CHF {:.2}", f.betrag), "font-semibold text-black")
                                                        };
                                                        let zeile = f.clone();
                                                        view! {
                                                            <tr class=format!("{zeilen_class} cursor-pointer select-none hover:bg-base-200/60") title="Doppelklick zum Bearbeiten" on:dblclick=move |_| bearbeiten_oeffnen(zeile.clone())>
                                                                <td>
                                                                    <div class="flex flex-wrap items-center gap-2 font-medium">
                                                                        <span>{f.buchungstext}</span>
                                                                        {ist_einnahme.then(|| view! {
                                                                            <span class="badge badge-success badge-outline badge-xs whitespace-nowrap">"Einnahme"</span>
                                                                        })}
                                                                    </div>
                                                                    <div class="text-xs text-base-content/60">{kategorie_text}</div>
                                                                </td>
                                                                <td class="whitespace-nowrap font-mono text-xs text-base-content/80">
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
                                                                    <div class=betrag_klasse>
                                                                        {betrag_text}
                                                                    </div>
                                                                    {(!ist_monatlich).then(|| view! {
                                                                        <div class="text-xs text-base-content/60">
                                                                            {format!("≈ CHF {:.2} / Monat", monatsbetrag)}
                                                                        </div>
                                                                    })}
                                                                </td>
                                                            </tr>
                                                        }
                                                    }).collect_view().into_any()
                                                }}
                                            </tbody>
                                            <tfoot>
                                                <tr class="font-bold text-xs bg-base-200/50">
                                                    <td colspan="5">
                                                        <div class="flex flex-col">
                                                            <span>"Summe Fixkosten pro Monat"</span>
                                                            <span class="text-xs font-normal text-base-content/70">
                                                                "Aktive und gekündigte Verträge, auf den Monat umgerechnet"
                                                            </span>
                                                        </div>
                                                    </td>
                                                    <td class="text-right text-black whitespace-nowrap">
                                                        {format!("CHF {:.2}", summe_kosten)}
                                                    </td>
                                                </tr>
                                                <tr class="font-bold text-xs bg-base-200/50">
                                                    <td colspan="5">
                                                        <div class="flex flex-col">
                                                            <span>"Summe Fixeinnahmen pro Monat"</span>
                                                            <span class="text-xs font-normal text-base-content/70">
                                                                "Aktive und gekündigte Einnahmen, auf den Monat umgerechnet"
                                                            </span>
                                                        </div>
                                                    </td>
                                                    <td class="text-right text-success whitespace-nowrap">
                                                        {format!("+ CHF {:.2}", summe_einnahmen)}
                                                    </td>
                                                </tr>
                                                <tr class="font-bold text-xs bg-base-200/50">
                                                    <td colspan="5">
                                                        <div class="flex flex-col">
                                                            <span>"Differenz pro Monat"</span>
                                                            <span class="text-xs font-normal text-base-content/70">
                                                                "Fixeinnahmen minus Fixkosten"
                                                            </span>
                                                        </div>
                                                    </td>
                                                    <td class=differenz_klasse>
                                                        {differenz_text}
                                                    </td>
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
