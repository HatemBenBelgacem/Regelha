use leptos::prelude::*;

/// Modalfenster für Eingabeformulare (DaisyUI `modal`).
///
/// Auf dem Smartphone erscheint es als Fenster am unteren Rand, ab `sm` mittig.
/// Schliessen per ✕, Klick auf den Hintergrund oder Escape-Taste.
/// Der Inhalt wird nur gerendert, solange das Fenster offen ist (beim SSR also nie),
/// damit Server- und Browser-Ausgabe für die Hydration gleich bleiben.
#[component]
pub fn Modal(
    /// Ist das Fenster offen?
    #[prop(into)]
    offen: Signal<bool>,
    /// Wird beim Schliessen aufgerufen (✕, Hintergrund, Escape)
    on_close: Callback<()>,
    /// Überschrift des Fensters (kann sich ändern, z.B. "Neue Buchung" / "Buchung bearbeiten")
    #[prop(into)]
    titel: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let children = StoredValue::new(children);

    view! {
        <Show when=move || offen.get()>
            <div
                class="modal modal-open modal-bottom sm:modal-middle"
                role="dialog"
                aria-modal="true"
                aria-label=move || titel.get()
                on:keydown=move |ev| {
                    if ev.key() == "Escape" {
                        on_close.run(());
                    }
                }
            >
                <div class="modal-box w-full sm:max-w-2xl p-4 sm:p-6">
                    <div class="flex items-center justify-between gap-2 mb-4">
                        <h3 class="font-bold text-lg text-[#66023C]">{move || titel.get()}</h3>
                        <button
                            type="button"
                            class="btn btn-circle btn-ghost"
                            aria-label="Fenster schliessen"
                            on:click=move |_| on_close.run(())
                        >
                            "✕"
                        </button>
                    </div>
                    {children.with_value(|c| c())}
                </div>
                <div class="modal-backdrop" on:click=move |_| on_close.run(())></div>
            </div>
        </Show>
    }
}
