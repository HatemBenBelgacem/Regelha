use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: Uuid,
    pub benutzername: String,
}

#[cfg(feature = "ssr")]
pub fn default_admin_uuid() -> Uuid {
    Uuid::from_u128(0x00000000_0000_4000_8000_000000000001)
}

/// Gültigkeit einer Session (Token-Ablauf und Cookie-Max-Age).
#[cfg(feature = "ssr")]
const SESSION_TTL_SECS: i64 = 60 * 60 * 24 * 7;

#[cfg(feature = "ssr")]
const MIN_SECRET_LEN: usize = 32;

#[cfg(feature = "ssr")]
static SESSION_SECRET: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();

/// Produktion = Release-Build oder Deployment auf Railway.
#[cfg(feature = "ssr")]
fn is_production() -> bool {
    !cfg!(debug_assertions) || std::env::var_os("RAILWAY_ENVIRONMENT").is_some()
}

#[cfg(feature = "ssr")]
fn load_session_secret() -> Result<Vec<u8>, String> {
    match std::env::var("SESSION_SECRET") {
        Ok(secret) if secret.len() >= MIN_SECRET_LEN => Ok(secret.into_bytes()),
        Ok(_) if is_production() => Err(format!(
            "SESSION_SECRET ist zu kurz (mindestens {MIN_SECRET_LEN} Zeichen)."
        )),
        Ok(secret) if !secret.is_empty() => {
            tracing::warn!(
                "SESSION_SECRET ist kürzer als {MIN_SECRET_LEN} Zeichen – nur für Entwicklung akzeptiert."
            );
            Ok(secret.into_bytes())
        }
        _ if is_production() => Err(
            "SESSION_SECRET ist nicht gesetzt. Ohne Secret startet der Server in Produktion nicht."
                .to_string(),
        ),
        _ => {
            use password_hash::rand_core::{OsRng, RngCore};
            tracing::warn!(
                "SESSION_SECRET ist nicht gesetzt – nutze zufälliges Secret (Sessions gelten nur bis zum Neustart)."
            );
            let mut secret = vec![0u8; 32];
            OsRng.fill_bytes(&mut secret);
            Ok(secret)
        }
    }
}

/// Lädt das Session-Secret beim Serverstart. Liefert einen Fehler, wenn in Produktion
/// kein (ausreichend langes) `SESSION_SECRET` gesetzt ist.
#[cfg(feature = "ssr")]
pub fn init_session_secret() -> Result<(), String> {
    let secret = load_session_secret()?;
    let _ = SESSION_SECRET.set(secret);
    Ok(())
}

#[cfg(feature = "ssr")]
fn session_secret() -> &'static [u8] {
    SESSION_SECRET.get_or_init(|| load_session_secret().unwrap_or_else(|e| panic!("{e}")))
}

#[cfg(feature = "ssr")]
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(feature = "ssr")]
fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok())
        .collect()
}

#[cfg(feature = "ssr")]
fn session_mac(payload: &str) -> hmac::Hmac<sha2::Sha256> {
    use hmac::Mac;
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(session_secret())
        .expect("HMAC akzeptiert Schlüssel jeder Länge");
    mac.update(b"regelha-session-v2.");
    mac.update(payload.as_bytes());
    mac
}

/// Token-Format: `<user-uuid>.<ablauf-unix-sekunden>.<hmac-sha256-hex>`
#[cfg(feature = "ssr")]
fn create_signed_session_token(user_id: &Uuid) -> String {
    use hmac::Mac;
    let expires = chrono::Utc::now().timestamp() + SESSION_TTL_SECS;
    let payload = format!("{user_id}.{expires}");
    let sig = session_mac(&payload).finalize().into_bytes();
    format!("{payload}.{}", hex_encode(&sig))
}

#[cfg(feature = "ssr")]
fn verify_signed_session_token(token: &str) -> Option<Uuid> {
    use hmac::Mac;
    let (payload, sig_hex) = token.rsplit_once('.')?;
    let sig = hex_decode(sig_hex)?;
    // verify_slice vergleicht in konstanter Zeit
    session_mac(payload).verify_slice(&sig).ok()?;

    let (id_str, expires_str) = payload.split_once('.')?;
    let expires: i64 = expires_str.parse().ok()?;
    if expires <= chrono::Utc::now().timestamp() {
        return None;
    }
    Uuid::parse_str(id_str).ok()
}

/// Erzeugt einen argon2id-Hash im PHC-Format (`$argon2id$v=19$...`).
#[cfg(feature = "ssr")]
pub fn hash_password(passwort: &str) -> String {
    use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
    use password_hash::rand_core::OsRng;

    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(passwort.as_bytes(), &salt)
        .expect("argon2-Hashing fehlgeschlagen")
        .to_string()
}

/// Alter FNV-artiger Hash (nur noch zum Prüfen und Migrieren bestehender Einträge).
#[cfg(feature = "ssr")]
fn legacy_hash_password(benutzername: &str, passwort: &str) -> String {
    let input = format!("regelha::v1::{benutzername}::{passwort}");
    let mut h1: u64 = 0xcbf29ce484222325;
    let mut h2: u64 = 0x84222325cbf29ce4;
    for (idx, b) in input.bytes().enumerate() {
        h1 ^= b as u64;
        h1 = h1.wrapping_mul(0x100000001b3);
        h2 ^= (b as u64).wrapping_add(idx as u64);
        h2 = h2.wrapping_mul(0x100000001b3);
    }
    format!("{h1:016x}{h2:016x}")
}

#[cfg(feature = "ssr")]
#[derive(Debug, PartialEq, Eq)]
enum PasswortPruefung {
    Korrekt,
    /// Korrekt, aber noch mit dem alten Hash gespeichert -> neu hashen.
    KorrektVeraltet,
    Falsch,
}

#[cfg(feature = "ssr")]
fn verify_password(benutzername: &str, passwort: &str, gespeichert: &str) -> PasswortPruefung {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    if gespeichert.starts_with("$argon2") {
        let ok = PasswordHash::new(gespeichert)
            .map(|h| Argon2::default().verify_password(passwort.as_bytes(), &h).is_ok())
            .unwrap_or(false);
        if ok { PasswortPruefung::Korrekt } else { PasswortPruefung::Falsch }
    } else {
        use subtle::ConstantTimeEq;
        let legacy = legacy_hash_password(benutzername, passwort);
        if bool::from(legacy.as_bytes().ct_eq(gespeichert.as_bytes())) {
            PasswortPruefung::KorrektVeraltet
        } else {
            PasswortPruefung::Falsch
        }
    }
}

/// Hash für unbekannte Benutzernamen, damit die Antwortzeit nicht verrät, ob es den Namen gibt.
#[cfg(feature = "ssr")]
static DUMMY_HASH: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| hash_password("regelha-dummy-passwort"));

#[cfg(feature = "ssr")]
async fn run_blocking<T: Send + 'static>(
    f: impl FnOnce() -> T + Send + 'static,
) -> Result<T, ServerFnError> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| ServerFnError::new(format!("Interner Fehler: {e}")))
}

/// Mindestlänge für das Admin-Passwort aus `ADMIN_PASSWORD`.
#[cfg(feature = "ssr")]
const MIN_ADMIN_PASSWORD_LEN: usize = 12;

/// Demo-Passwort für die Standardkonten. Gibt es nur in der Entwicklung, nie in Produktion.
#[cfg(feature = "ssr")]
pub fn dev_standardpasswort(benutzername: &str) -> Option<&'static str> {
    if is_production() {
        return None;
    }
    match benutzername {
        "admin" => Some("admin123"),
        "anna" => Some("anna123"),
        "lukas" => Some("lukas123"),
        _ => None,
    }
}

/// Liest das Admin-Passwort aus `ADMIN_PASSWORD`. Ist es gesetzt, gilt es als maßgeblich
/// und wird bei jedem Serverstart für das Konto `admin` übernommen.
#[cfg(feature = "ssr")]
pub fn admin_passwort_aus_env() -> Option<String> {
    let passwort = std::env::var("ADMIN_PASSWORD").ok()?;
    if passwort.len() >= MIN_ADMIN_PASSWORD_LEN {
        Some(passwort)
    } else {
        tracing::warn!(
            "ADMIN_PASSWORD ist kürzer als {MIN_ADMIN_PASSWORD_LEN} Zeichen und wird ignoriert."
        );
        None
    }
}

#[cfg(feature = "ssr")]
pub(crate) mod store {
    use super::*;
    use std::sync::{LazyLock, Mutex};

    #[derive(Clone, Debug)]
    pub struct StoredUser {
        pub id: Uuid,
        pub benutzername: String,
        pub passwort_hash: String,
    }

    pub static USERS_MEMORY: LazyLock<Mutex<Vec<StoredUser>>> = LazyLock::new(|| {
        let mut users = Vec::new();
        let admin_passwort = admin_passwort_aus_env()
            .or_else(|| dev_standardpasswort("admin").map(str::to_string));
        if let Some(passwort) = admin_passwort {
            users.push(StoredUser {
                id: default_admin_uuid(),
                benutzername: "admin".to_string(),
                passwort_hash: hash_password(&passwort),
            });
        }
        if let Some(passwort) = dev_standardpasswort("anna") {
            users.push(StoredUser {
                id: Uuid::from_u128(0x00000000_0000_4000_8000_000000000002),
                benutzername: "anna".to_string(),
                passwort_hash: hash_password(passwort),
            });
        }
        if let Some(passwort) = dev_standardpasswort("lukas") {
            users.push(StoredUser {
                id: Uuid::from_u128(0x00000000_0000_4000_8000_000000000003),
                benutzername: "lukas".to_string(),
                passwort_hash: hash_password(passwort),
            });
        }
        Mutex::new(users)
    });
}

#[cfg(feature = "ssr")]
fn set_session_cookie(token: &str, max_age_secs: i64) {
    use axum::http::header::{HeaderValue, SET_COOKIE};
    use leptos_axum::ResponseOptions;

    if let Some(res) = use_context::<ResponseOptions>() {
        let cookie = format!(
            "regelha_session={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age_secs}"
        );
        if let Ok(val) = HeaderValue::from_str(&cookie) {
            res.insert_header(SET_COOKIE, val);
        }
    }
}

#[cfg(feature = "ssr")]
async fn extract_session_token() -> Option<String> {
    use axum::http::{header::COOKIE, HeaderMap};

    let headers: HeaderMap = leptos_axum::extract().await.ok()?;
    let cookie_str = headers.get(COOKIE)?.to_str().ok()?;
    for part in cookie_str.split(';') {
        let trimmed = part.trim();
        if let Some(val) = trimmed.strip_prefix("regelha_session=") {
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

#[cfg(feature = "ssr")]
pub async fn require_current_user() -> Result<AuthUser, ServerFnError> {
    match get_current_user().await? {
        Some(user) => Ok(user),
        None => Err(ServerFnError::new(
            "Nicht angemeldet. Bitte zuerst anmelden.",
        )),
    }
}

#[server(
    GetCurrentUser,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "get_current_user"
)]
pub async fn get_current_user() -> Result<Option<AuthUser>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use sqlx::{PgPool, Row};

        if let Some(token) = extract_session_token().await {
            if let Some(user_id) = verify_signed_session_token(&token) {
                if let Some(pool) = use_context::<PgPool>() {
                    let row_opt = sqlx::query(
                        "SELECT id, benutzername FROM benutzer WHERE id = $1",
                    )
                    .bind(user_id)
                    .fetch_optional(&pool)
                    .await
                    .ok()
                    .flatten();

                    return Ok(row_opt.map(|r| AuthUser {
                        id: r.get("id"),
                        benutzername: r.get("benutzername"),
                    }));
                }
                let guard = store::USERS_MEMORY
                    .lock()
                    .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
                return Ok(guard.iter().find(|u| u.id == user_id).map(|u| AuthUser {
                    id: u.id,
                    benutzername: u.benutzername.clone(),
                }));
            }
        }
    }
    Ok(None)
}

#[server(
    LoginUser,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "login_user"
)]
pub async fn login_user(
    benutzername: String,
    passwort: String,
) -> Result<AuthUser, ServerFnError> {
    let username = benutzername.trim().to_string();
    if username.is_empty() || passwort.is_empty() {
        return Err(ServerFnError::new(
            "Bitte Benutzername und Passwort eingeben.",
        ));
    }

    #[cfg(feature = "ssr")]
    {
        use sqlx::{PgPool, Row};

        // (Benutzer, gespeicherter Hash); bei unbekanntem Namen gegen Dummy-Hash prüfen
        let (kandidat, gespeichert) = if let Some(pool) = use_context::<PgPool>() {
            let row_opt = sqlx::query(
                "SELECT id, benutzername, passwort_hash FROM benutzer WHERE benutzername = $1",
            )
            .bind(&username)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Fehler bei Anmeldung: {e}")))?;

            match row_opt {
                Some(row) => (
                    Some(AuthUser {
                        id: row.get("id"),
                        benutzername: row.get("benutzername"),
                    }),
                    row.get::<String, _>("passwort_hash"),
                ),
                None => (None, DUMMY_HASH.clone()),
            }
        } else {
            let guard = store::USERS_MEMORY
                .lock()
                .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
            match guard.iter().find(|u| u.benutzername == username) {
                Some(u) => (
                    Some(AuthUser {
                        id: u.id,
                        benutzername: u.benutzername.clone(),
                    }),
                    u.passwort_hash.clone(),
                ),
                None => (None, DUMMY_HASH.clone()),
            }
        };

        let pruefung = {
            let (name, pw, hash) = (username.clone(), passwort.clone(), gespeichert.clone());
            run_blocking(move || verify_password(&name, &pw, &hash)).await?
        };

        let authenticated_user = match (kandidat, pruefung) {
            (Some(user), PasswortPruefung::Korrekt) => Some(user),
            (Some(user), PasswortPruefung::KorrektVeraltet) => {
                // Alt-Hash beim erfolgreichen Login auf argon2 migrieren
                let pw = passwort.clone();
                let neuer_hash = run_blocking(move || hash_password(&pw)).await?;
                if let Some(pool) = use_context::<PgPool>() {
                    if let Err(e) = sqlx::query(
                        "UPDATE benutzer SET passwort_hash = $1 WHERE id = $2 AND passwort_hash = $3",
                    )
                    .bind(&neuer_hash)
                    .bind(user.id)
                    .bind(&gespeichert)
                    .execute(&pool)
                    .await
                    {
                        tracing::warn!("Passwort-Hash von {} nicht migriert: {e}", user.benutzername);
                    }
                } else if let Ok(mut guard) = store::USERS_MEMORY.lock() {
                    if let Some(u) = guard
                        .iter_mut()
                        .find(|u| u.id == user.id && u.passwort_hash == gespeichert)
                    {
                        u.passwort_hash = neuer_hash;
                    }
                }
                Some(user)
            }
            _ => None,
        };

        match authenticated_user {
            Some(user) => {
                let token = create_signed_session_token(&user.id);
                set_session_cookie(&token, SESSION_TTL_SECS);
                Ok(user)
            }
            None => Err(ServerFnError::new(
                "Ungültiger Benutzername oder falsches Passwort.",
            )),
        }
    }
}

#[server(
    RegisterUser,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "register_user"
)]
pub async fn register_user(
    benutzername: String,
    passwort: String,
) -> Result<AuthUser, ServerFnError> {
    let username = benutzername.trim().to_string();
    if username.len() < 3 {
        return Err(ServerFnError::new(
            "Der Benutzername muss mindestens 3 Zeichen lang sein.",
        ));
    }
    if passwort.len() < 4 {
        return Err(ServerFnError::new(
            "Das Passwort muss mindestens 4 Zeichen lang sein.",
        ));
    }

    #[cfg(feature = "ssr")]
    {
        use sqlx::PgPool;

        let new_id = Uuid::new_v4();
        let pw_hash = run_blocking(move || hash_password(&passwort)).await?;

        if let Some(pool) = use_context::<PgPool>() {
            let res = sqlx::query(
                "INSERT INTO benutzer (id, benutzername, passwort_hash) VALUES ($1, $2, $3)",
            )
            .bind(new_id)
            .bind(&username)
            .bind(&pw_hash)
            .execute(&pool)
            .await;

            if res.is_err() {
                return Err(ServerFnError::new(
                    "Dieser Benutzername ist bereits vergeben.",
                ));
            }
        } else {
            let mut guard = store::USERS_MEMORY
                .lock()
                .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
            if guard.iter().any(|u| u.benutzername == username) {
                return Err(ServerFnError::new(
                    "Dieser Benutzername ist bereits vergeben.",
                ));
            }
            guard.push(store::StoredUser {
                id: new_id,
                benutzername: username.clone(),
                passwort_hash: pw_hash,
            });
        }

        let user = AuthUser {
            id: new_id,
            benutzername: username,
        };
        let token = create_signed_session_token(&user.id);
        set_session_cookie(&token, SESSION_TTL_SECS);
        Ok(user)
    }
}

#[server(
    LogoutUser,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "logout_user"
)]
pub async fn logout_user() -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        set_session_cookie("", 0);
    }
    Ok(())
}
