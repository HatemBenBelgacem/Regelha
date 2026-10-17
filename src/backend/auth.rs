use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: Uuid,
    pub benutzername: String,
}

#[cfg(feature = "ssr")]
pub fn hash_password(benutzername: &str, passwort: &str) -> String {
    // Deterministischer gesalzener 128-Bit FNV-1a Hash ohne zusätzliche Crate-Abhängigkeit
    let input = format!("budgetplaner::v1::{benutzername}::{passwort}");
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
mod store {
    use super::*;
    use std::{
        collections::HashMap,
        sync::{LazyLock, Mutex},
    };

    #[derive(Clone, Debug)]
    pub struct StoredUser {
        pub id: Uuid,
        pub benutzername: String,
        pub passwort_hash: String,
    }

    pub static USERS_MEMORY: LazyLock<Mutex<Vec<StoredUser>>> = LazyLock::new(|| {
        Mutex::new(vec![StoredUser {
            id: Uuid::new_v4(),
            benutzername: "admin".to_string(),
            passwort_hash: hash_password("admin", "admin123"),
        }])
    });

    // Session-Token -> AuthUser
    pub static SESSIONS_MEMORY: LazyLock<Mutex<HashMap<String, AuthUser>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));
}

#[cfg(feature = "ssr")]
fn set_session_cookie(token: &str, max_age_secs: i64) {
    use axum::http::header::{HeaderValue, SET_COOKIE};
    use leptos_axum::ResponseOptions;

    if let Some(res) = use_context::<ResponseOptions>() {
        let cookie = format!(
            "bp_session={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age_secs}"
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
        if let Some(val) = trimmed.strip_prefix("bp_session=") {
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

#[server(GetCurrentUser, "/api")]
pub async fn get_current_user() -> Result<Option<AuthUser>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        if let Some(token) = extract_session_token().await {
            let guard = store::SESSIONS_MEMORY
                .lock()
                .map_err(|_| ServerFnError::new("Session-Lock fehlgeschlagen"))?;
            return Ok(guard.get(&token).cloned());
        }
    }
    Ok(None)
}

#[server(LoginUser, "/api")]
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

        let expected_hash = hash_password(&username, &passwort);

        let authenticated_user = if let Some(pool) = use_context::<PgPool>() {
            let row_opt = sqlx::query(
                "SELECT id, benutzername, passwort_hash FROM benutzer WHERE benutzername = $1",
            )
            .bind(&username)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Fehler bei Anmeldung: {e}")))?;

            match row_opt {
                Some(row) => {
                    let db_hash: String = row.get("passwort_hash");
                    if db_hash == expected_hash {
                        Some(AuthUser {
                            id: row.get("id"),
                            benutzername: row.get("benutzername"),
                        })
                    } else {
                        None
                    }
                }
                None => None,
            }
        } else {
            let guard = store::USERS_MEMORY
                .lock()
                .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
            guard
                .iter()
                .find(|u| u.benutzername == username && u.passwort_hash == expected_hash)
                .map(|u| AuthUser {
                    id: u.id,
                    benutzername: u.benutzername.clone(),
                })
        };

        match authenticated_user {
            Some(user) => {
                let token = Uuid::new_v4().to_string();
                if let Ok(mut sessions) = store::SESSIONS_MEMORY.lock() {
                    sessions.insert(token.clone(), user.clone());
                }
                set_session_cookie(&token, 60 * 60 * 24 * 7);
                Ok(user)
            }
            None => Err(ServerFnError::new(
                "Ungültiger Benutzername oder falsches Passwort.",
            )),
        }
    }
}

#[server(RegisterUser, "/api")]
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
        let pw_hash = hash_password(&username, &passwort);

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
        let token = Uuid::new_v4().to_string();
        if let Ok(mut sessions) = store::SESSIONS_MEMORY.lock() {
            sessions.insert(token.clone(), user.clone());
        }
        set_session_cookie(&token, 60 * 60 * 24 * 7);
        Ok(user)
    }
}

#[server(LogoutUser, "/api")]
pub async fn logout_user() -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        if let Some(token) = extract_session_token().await {
            if let Ok(mut sessions) = store::SESSIONS_MEMORY.lock() {
                sessions.remove(&token);
            }
        }
        set_session_cookie("", 0);
    }
    Ok(())
}
