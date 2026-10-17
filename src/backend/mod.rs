pub mod aufgaben;
pub mod auth;
pub mod budget;
pub mod chat;
#[cfg(feature = "ssr")]
pub mod db;

#[cfg(feature = "ssr")]
pub fn register_server_functions() {
    use leptos::server_fn::axum::register_explicit;

    // Auth Server Functions
    register_explicit::<auth::GetCurrentUser>();
    register_explicit::<auth::LoginUser>();
    register_explicit::<auth::RegisterUser>();
    register_explicit::<auth::LogoutUser>();

    // Budget / Finanzen Server Functions
    register_explicit::<budget::GetDashboardKennzahlen>();
    register_explicit::<budget::GetBuchungen>();
    register_explicit::<budget::AddBuchung>();
    register_explicit::<budget::DeleteBuchung>();
    register_explicit::<budget::GetFixkosten>();
    register_explicit::<budget::AddFixkosten>();
    register_explicit::<budget::DeleteFixkosten>();

    // Aufgaben Server Functions
    register_explicit::<aufgaben::GetAufgaben>();
    register_explicit::<aufgaben::AddAufgabe>();
    register_explicit::<aufgaben::ToggleAufgabeErledigt>();
    register_explicit::<aufgaben::DeleteAufgabe>();

    // Chat Server Functions
    register_explicit::<chat::GetChatUebersicht>();
    register_explicit::<chat::OpenDirectChat>();
    register_explicit::<chat::CreateGroupChat>();
    register_explicit::<chat::GetChatNachrichten>();
    register_explicit::<chat::SendChatNachricht>();
}
