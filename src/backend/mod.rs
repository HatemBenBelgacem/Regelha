pub mod aufgaben;
pub mod auth;
pub mod budget;
pub mod chat;
#[cfg(feature = "ssr")]
pub mod db;
pub mod kontakte;
pub mod push;
pub mod schulden;

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
    register_explicit::<budget::UpdateBuchung>();
    register_explicit::<budget::DeleteBuchung>();
    register_explicit::<budget::GetFixkosten>();
    register_explicit::<budget::AddFixkosten>();
    register_explicit::<budget::UpdateFixkosten>();
    register_explicit::<budget::UpdateFixkostenStatus>();
    register_explicit::<budget::DeleteFixkosten>();
    register_explicit::<budget::GetMonatskosten>();

    // Aufgaben Server Functions
    register_explicit::<aufgaben::GetAufgaben>();
    register_explicit::<aufgaben::AddAufgabe>();
    register_explicit::<aufgaben::UpdateAufgabe>();
    register_explicit::<aufgaben::ToggleAufgabeErledigt>();
    register_explicit::<aufgaben::DeleteAufgabe>();

    // Schulden Server Functions
    register_explicit::<schulden::GetSchulden>();
    register_explicit::<schulden::AddSchuld>();
    register_explicit::<schulden::UpdateSchuld>();
    register_explicit::<schulden::DeleteSchuld>();

    // Kontakte Server Functions
    register_explicit::<kontakte::GetKontakte>();
    register_explicit::<kontakte::AddKontakt>();
    register_explicit::<kontakte::UpdateKontakt>();
    register_explicit::<kontakte::DeleteKontakt>();

    // Chat Server Functions
    register_explicit::<chat::GetChatUebersicht>();
    register_explicit::<chat::OpenDirectChat>();
    register_explicit::<chat::CreateGroupChat>();
    register_explicit::<chat::GetChatNachrichten>();
    register_explicit::<chat::SendChatNachricht>();
    register_explicit::<chat::ChatLiveAbfrage>();
    register_explicit::<chat::SetChatTippstatus>();
    register_explicit::<chat::GetUngeleseneChats>();
    register_explicit::<chat::MarkiereChatGelesen>();

    // Push-Benachrichtigungen
    register_explicit::<push::GetPushPublicKey>();
    register_explicit::<push::SpeicherePushAbo>();
}
