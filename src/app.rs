use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{ParentRoute, Route, Router, Routes},
    StaticSegment,
};

use crate::components::{
    app_layout::AppLayout, buchungen::BuchungenSection, dashboard::BudgetDashboard,
    fixkosten::FixkostenSection,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="de" data-theme="light">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body class="min-h-screen bg-base-200 text-base-content">
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/budgetplaner.css"/>
        <Title text="Budgetplaner"/>

        <Router>
            <Routes fallback=|| "Seite nicht gefunden.".into_view()>
                <ParentRoute path=StaticSegment("") view=AppLayout>
                    <Route path=StaticSegment("") view=BudgetDashboard/>
                    <Route path=StaticSegment("buchungen") view=BuchungenSection/>
                    <Route path=StaticSegment("fixkosten") view=FixkostenSection/>
                </ParentRoute>
            </Routes>
        </Router>
    }
}
