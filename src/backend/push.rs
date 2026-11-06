use leptos::prelude::*;

// Geräte-Abos für Push-Benachrichtigungen (Web Push, VAPID).
//
// Der Browser meldet ein Gerät mit einem Abo an: Adresse des Push-Dienstes (endpoint) und zwei
// Schlüssel. Der Server verschlüsselt die Nachricht für genau dieses Gerät und schickt sie an den
// Push-Dienst. Der Dienst weckt damit den Browser, auch wenn die App geschlossen ist.
//
// Server-Einstellungen (Umgebungsvariablen):
// - VAPID_PUBLIC_KEY:  öffentlicher Schlüssel (wird an den Browser gegeben)
// - VAPID_PRIVATE_KEY: privater Schlüssel (geheim, nie ins Repository)
// - VAPID_SUBJECT:     Kontakt, z.B. "mailto:admin@example.org"

#[cfg(feature = "ssr")]
use uuid::Uuid;

/// Push-Dienste, an die der Server senden darf (Chrome/Edge, Firefox, Safari).
/// Nur diese Adressen werden gespeichert. Sonst könnte der Server an beliebige Ziele senden.
#[cfg(feature = "ssr")]
const ERLAUBTE_PUSH_HOSTS: [&str; 3] = [
    "fcm.googleapis.com",
    "updates.push.services.mozilla.com",
    "web.push.apple.com",
];

#[cfg(feature = "ssr")]
fn pruefe_push_endpoint(endpoint: &str) -> Result<(), ServerFnError> {
    let rest = endpoint
        .strip_prefix("https://")
        .ok_or_else(|| ServerFnError::new("Ungültiger Push-Endpunkt."))?;
    let host = rest.split('/').next().unwrap_or("");
    let erlaubt = ERLAUBTE_PUSH_HOSTS.contains(&host) || host.ends_with(".notify.windows.com");
    if !erlaubt || host.contains('@') || host.contains(':') {
        return Err(ServerFnError::new("Dieser Push-Dienst wird nicht unterstützt."));
    }
    Ok(())
}

/// Kurze Vorschau für die Benachrichtigung (höchstens 120 Zeichen).
#[cfg(feature = "ssr")]
pub fn kurz_vorschau(inhalt: &str, hat_anhang: bool) -> String {
    let text = inhalt.trim();
    if text.is_empty() {
        return if hat_anhang { "📎 Datei angehängt".to_string() } else { String::new() };
    }
    if text.chars().count() <= 120 {
        return text.to_string();
    }
    let kurz: String = text.chars().take(119).collect();
    format!("{kurz}…")
}

#[cfg(feature = "ssr")]
mod store {
    use super::*;
    use std::sync::{LazyLock, Mutex};

    #[derive(Clone, Debug)]
    pub struct StoredPushAbo {
        pub endpoint: String,
        pub benutzer_id: Uuid,
        pub p256dh: String,
        pub auth: String,
    }

    pub static PUSH_ABOS_MEMORY: LazyLock<Mutex<Vec<StoredPushAbo>>> =
        LazyLock::new(|| Mutex::new(Vec::new()));
}

/// Öffentlicher VAPID-Schlüssel für den Browser. Ohne Schlüssel sind Push-Nachrichten aus.
#[server(
    GetPushPublicKey,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "get_push_public_key"
)]
pub async fn get_push_public_key() -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let schluessel = std::env::var("VAPID_PUBLIC_KEY")
            .unwrap_or_default()
            .trim()
            .to_string();
        if schluessel.is_empty() {
            return Err(ServerFnError::new(
                "Push-Benachrichtigungen sind auf dem Server nicht eingerichtet.",
            ));
        }
        Ok(schluessel)
    }

    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

/// Speichert das Abo eines Geräts für den angemeldeten Benutzer.
/// Ein Gerät, das schon angemeldet ist, bekommt den Benutzer und die Schlüssel neu.
#[server(
    SpeicherePushAbo,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "speichere_push_abo"
)]
pub async fn speichere_push_abo(
    endpoint: String,
    p256dh: String,
    auth: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let current_user = require_current_user().await?;
        let endpoint = endpoint.trim().to_string();
        pruefe_push_endpoint(&endpoint)?;
        let p256dh = p256dh.trim().to_string();
        let auth = auth.trim().to_string();
        if p256dh.is_empty() || auth.is_empty() {
            return Err(ServerFnError::new("Die Schlüssel des Geräts fehlen."));
        }

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query(
                r#"
                INSERT INTO push_abo (endpoint, benutzer_id, p256dh, auth)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (endpoint)
                DO UPDATE SET benutzer_id = EXCLUDED.benutzer_id,
                              p256dh = EXCLUDED.p256dh,
                              auth = EXCLUDED.auth
                "#,
            )
            .bind(&endpoint)
            .bind(current_user.id)
            .bind(&p256dh)
            .bind(&auth)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Fehler (push_abo): {e}")))?;
            return Ok(());
        }

        // In-Memory-Fallback
        let mut guard = store::PUSH_ABOS_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.retain(|a| a.endpoint != endpoint);
        guard.push(store::StoredPushAbo {
            endpoint,
            benutzer_id: current_user.id,
            p256dh,
            auth,
        });
        Ok(())
    }

    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

// -----------------------------------------------------------------------------
// Versand (nur auf dem Server)
// -----------------------------------------------------------------------------

#[cfg(feature = "ssr")]
#[derive(Clone, Debug)]
struct AboDaten {
    endpoint: String,
    p256dh: String,
    auth: String,
}

#[cfg(feature = "ssr")]
fn env_wert(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Alle Geräte der anderen Chat-Mitglieder (ohne den Absender).
#[cfg(feature = "ssr")]
async fn lade_abos_der_empfaenger(
    pool: Option<&sqlx::PgPool>,
    chat_id: Uuid,
    absender_id: Uuid,
) -> Vec<AboDaten> {
    use sqlx::Row;

    if let Some(pool) = pool {
        return match sqlx::query(
            r#"
            SELECT p.endpoint, p.p256dh, p.auth
            FROM push_abo p
            INNER JOIN rel_benutzer_chat r ON r.benutzer_id = p.benutzer_id
            WHERE r.chat_id = $1 AND p.benutzer_id <> $2
            "#,
        )
        .bind(chat_id)
        .bind(absender_id)
        .fetch_all(pool)
        .await
        {
            Ok(rows) => rows
                .into_iter()
                .map(|r| AboDaten {
                    endpoint: r.get("endpoint"),
                    p256dh: r.get("p256dh"),
                    auth: r.get("auth"),
                })
                .collect(),
            Err(e) => {
                tracing::warn!("Push: Abos konnten nicht geladen werden: {e}");
                Vec::new()
            }
        };
    }

    // In-Memory-Fallback
    let mitglieder: Vec<Uuid> = {
        let rel = crate::backend::chat::store::REL_BENUTZER_CHAT_MEMORY.lock().ok();
        match rel {
            Some(g) => g
                .iter()
                .filter(|r| r.chat_id == chat_id && r.benutzer_id != absender_id)
                .map(|r| r.benutzer_id)
                .collect(),
            None => Vec::new(),
        }
    };
    match store::PUSH_ABOS_MEMORY.lock() {
        Ok(abos) => abos
            .iter()
            .filter(|a| mitglieder.contains(&a.benutzer_id))
            .map(|a| AboDaten {
                endpoint: a.endpoint.clone(),
                p256dh: a.p256dh.clone(),
                auth: a.auth.clone(),
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Entfernt ein Abo, das der Push-Dienst als ungültig meldet (z.B. Gerät abgemeldet).
#[cfg(feature = "ssr")]
async fn entferne_abo(pool: Option<&sqlx::PgPool>, endpoint: &str) {
    if let Some(pool) = pool {
        let _ = sqlx::query("DELETE FROM push_abo WHERE endpoint = $1")
            .bind(endpoint)
            .execute(pool)
            .await;
        return;
    }
    if let Ok(mut guard) = store::PUSH_ABOS_MEMORY.lock() {
        guard.retain(|a| a.endpoint != endpoint);
    }
}

/// Sendet eine Nachricht an ein einzelnes Gerät. Verschlüsselt sie mit den Schlüsseln des Geräts.
#[cfg(feature = "ssr")]
async fn sende_an_abo(
    client: &web_push::IsahcWebPushClient,
    abo: &AboDaten,
    private_key: &str,
    subject: &str,
    payload: &[u8],
) -> Result<(), web_push::WebPushError> {
    use web_push::{
        ContentEncoding, SubscriptionInfo, VapidSignatureBuilder, WebPushClient,
        WebPushMessageBuilder,
    };

    let info = SubscriptionInfo::new(abo.endpoint.as_str(), abo.p256dh.as_str(), abo.auth.as_str());
    let mut signatur = VapidSignatureBuilder::from_base64(private_key, &info)?;
    signatur.add_claim("sub", subject);
    let signatur = signatur.build()?;

    let mut nachricht = WebPushMessageBuilder::new(&info);
    nachricht.set_ttl(3600);
    nachricht.set_payload(ContentEncoding::Aes128Gcm, payload);
    nachricht.set_vapid_signature(signatur);
    client.send(nachricht.build()?).await
}

/// Schickt eine Benachrichtigung an alle anderen Mitglieder des Chats.
/// Läuft im Hintergrund, damit das Senden der Nachricht nicht wartet.
#[cfg(feature = "ssr")]
pub fn benachrichtige_nach_nachricht(
    pool: Option<sqlx::PgPool>,
    chat_id: Uuid,
    absender_id: Uuid,
    absender_name: String,
    vorschau: String,
) {
    tokio::spawn(async move {
        let abos = lade_abos_der_empfaenger(pool.as_ref(), chat_id, absender_id).await;
        tracing::info!("Push: {} Gerät(e) für Chat {chat_id} gefunden", abos.len());
        if abos.is_empty() {
            return;
        }
        let Some(private_key) = env_wert("VAPID_PRIVATE_KEY") else {
            tracing::warn!("Push: VAPID_PRIVATE_KEY fehlt, keine Benachrichtigungen gesendet");
            return;
        };
        let subject =
            env_wert("VAPID_SUBJECT").unwrap_or_else(|| "mailto:admin@example.org".to_string());
        let client = match web_push::IsahcWebPushClient::new() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Push: Client konnte nicht erstellt werden: {e}");
                return;
            }
        };

        let payload = serde_json::json!({
            "title": format!("Neue Nachricht von {absender_name}"),
            "body": vorschau,
            "url": "/chat",
            "tag": format!("chat-{chat_id}"),
        })
        .to_string();

        for abo in abos {
            match sende_an_abo(&client, &abo, &private_key, &subject, payload.as_bytes()).await {
                Ok(()) => tracing::info!("Push: Benachrichtigung an ein Gerät gesendet"),
                Err(
                    web_push::WebPushError::EndpointNotValid(_)
                    | web_push::WebPushError::EndpointNotFound(_),
                ) => {
                    tracing::info!("Push: ungültiges Gerät wird entfernt");
                    entferne_abo(pool.as_ref(), &abo.endpoint).await;
                }
                Err(e) => tracing::warn!("Push an ein Gerät fehlgeschlagen: {e}"),
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erlaubt_nur_bekannte_push_dienste() {
        assert!(pruefe_push_endpoint("https://fcm.googleapis.com/fcm/send/abc").is_ok());
        assert!(pruefe_push_endpoint("https://updates.push.services.mozilla.com/wpush/v2/x").is_ok());
        assert!(pruefe_push_endpoint("https://wns2-par02p.notify.windows.com/w/?x").is_ok());
        assert!(pruefe_push_endpoint("https://web.push.apple.com/abc").is_ok());
    }

    #[test]
    fn lehnt_fremde_und_gefaehrliche_adressen_ab() {
        assert!(pruefe_push_endpoint("https://evil.example.com/x").is_err());
        assert!(pruefe_push_endpoint("http://fcm.googleapis.com/x").is_err());
        assert!(pruefe_push_endpoint("https://fcm.googleapis.com:8443/x").is_err());
        assert!(pruefe_push_endpoint("https://user@fcm.googleapis.com/x").is_err());
        assert!(pruefe_push_endpoint("https://127.0.0.1/x").is_err());
    }

    #[test]
    fn vorschau_wird_gekuerzt() {
        assert_eq!(kurz_vorschau("  Hallo  ", false), "Hallo");
        assert_eq!(kurz_vorschau("", true), "📎 Datei angehängt");
        let lang = "x".repeat(200);
        let v = kurz_vorschau(&lang, false);
        assert_eq!(v.chars().count(), 120);
        assert!(v.ends_with('…'));
    }

    /// Prüft den Versand bis zur HTTP-Anfrage: verschlüsselt, mit VAPID-Signatur und TTL.
    /// Die Nachricht geht an einen lokalen Test-Server statt an einen echten Push-Dienst.
    #[tokio::test]
    async fn versand_schickt_verschluesselte_nachricht_mit_vapid() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        // Testschlüssel aus der Dokumentation der web-push-Crate (kein Geheimnis)
        let private_key = "IQ9Ur0ykXoHS9gzfYX0aBjy9lvdrjx_PFUXmie9YRcY";

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut daten: Vec<u8> = Vec::new();
            let mut puffer = [0u8; 4096];
            loop {
                let n = sock.read(&mut puffer).await.unwrap();
                if n == 0 {
                    break;
                }
                daten.extend_from_slice(&puffer[..n]);
                if let Some(ende) = daten.windows(4).position(|w| w == b"\r\n\r\n") {
                    let kopf = String::from_utf8_lossy(&daten[..ende]).to_string();
                    let laenge: usize = kopf
                        .lines()
                        .find_map(|zeile| {
                            zeile
                                .to_ascii_lowercase()
                                .strip_prefix("content-length:")
                                .map(|v| v.trim().parse::<usize>().unwrap_or(0))
                        })
                        .unwrap_or(0);
                    while daten.len() < ende + 4 + laenge {
                        let n = sock.read(&mut puffer).await.unwrap();
                        if n == 0 {
                            break;
                        }
                        daten.extend_from_slice(&puffer[..n]);
                    }
                    sock.write_all(b"HTTP/1.1 201 Created\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                        .await
                        .unwrap();
                    return kopf;
                }
            }
            String::new()
        });

        // Gültige Geräteschlüssel aus den Testdaten der web-push-Crate
        let abo = AboDaten {
            endpoint: format!("http://127.0.0.1:{port}/push/test"),
            p256dh: "BLMbF9ffKBiWQLCKvTHb6LO8Nb6dcUh6TItC455vu2kElga6PQvUmaFyCdykxY2nOSSL3yKgfbmFLRTUaGv4yV8".to_string(),
            auth: "xS03Fi5ErfTNH_l9WHE9Ig".to_string(),
        };
        let client = web_push::IsahcWebPushClient::new().unwrap();
        let payload = br#"{"title":"Test","body":"Hallo"}"#;

        let ergebnis = sende_an_abo(&client, &abo, private_key, "mailto:test@example.org", payload).await;
        let kopf = server.await.unwrap();

        assert!(ergebnis.is_ok(), "Versand fehlgeschlagen: {:?}", ergebnis.err());
        let klein = kopf.to_ascii_lowercase();
        assert!(klein.starts_with("post /push/test"), "{kopf}");
        assert!(klein.contains("content-encoding: aes128gcm"), "{kopf}");
        assert!(klein.contains("authorization: vapid t="), "{kopf}");
        assert!(klein.contains("ttl: 3600"), "{kopf}");
    }
}
