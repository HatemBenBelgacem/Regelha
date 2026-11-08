use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{ParentRoute, Route, Router, Routes},
    StaticSegment,
};

use crate::components::{
    app_layout::AppLayout, aufgaben::AufgabenSection, buchungen::BuchungenSection,
    chat::ChatSection, dashboard::BudgetDashboard, fixkosten::FixkostenSection,
    monatskosten::MonatskostenSection, kontakte::KontakteSection,
    schulden::SchuldenSection,
};

/// Meldet den Service Worker an (für die Installation als App). Läuft nach dem Laden der Seite.
const SW_REGISTRIERUNG: &str = "if ('serviceWorker' in navigator) { window.addEventListener('load', function () { navigator.serviceWorker.register('/sw.js'); }); }";

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="de" data-theme="light">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                // PWA: Installation als App auf dem Handy
                <meta name="theme-color" content="#66023C"/>
                <meta name="apple-mobile-web-app-capable" content="yes"/>
                <meta name="apple-mobile-web-app-title" content="Regelha"/>
                <link rel="manifest" href="/manifest.json"/>
                <link rel="icon" type="image/png" sizes="192x192" href="/icons/icon-192.png"/>
                <link rel="apple-touch-icon" href="/icons/apple-touch-icon.png"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
                <script inner_html=SW_REGISTRIERUNG></script>
                <script src="/push.js"></script>
            </head>
            <body class="min-h-screen bg-[#F8FAFC] text-base-content">
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/regelha.css"/>
        <Title text="Regelha"/>

        <Router>
            <Routes fallback=|| "Seite nicht gefunden.".into_view()>
                <ParentRoute path=StaticSegment("") view=AppLayout>
                    <Route path=StaticSegment("") view=BudgetDashboard/>
                    <Route path=StaticSegment("buchungen") view=BuchungenSection/>
                    <Route path=StaticSegment("fixkosten") view=FixkostenSection/>
                    <Route path=StaticSegment("monatskosten") view=MonatskostenSection/>
                    <Route path=StaticSegment("schulden") view=SchuldenSection/>
                    <Route path=StaticSegment("aufgaben") view=AufgabenSection/>
                    <Route path=StaticSegment("kontakte") view=KontakteSection/>
                    <Route path=StaticSegment("chat") view=ChatSection/>
                </ParentRoute>
            </Routes>
        </Router>
    }
}
