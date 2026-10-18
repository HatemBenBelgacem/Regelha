#[cfg(feature = "ssr")]
use chrono::{DateTime, Local, Utc};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatTeilnehmer {
    pub id: Uuid,
    pub benutzername: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatRaum {
    pub id: Uuid,
    pub name: String,
    pub ist_gruppe: bool,
    pub teilnehmer: Vec<ChatTeilnehmer>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatNachricht {
    pub id: Uuid,
    pub chat_id: Uuid,
    pub benutzer_id: Uuid,
    pub benutzername: String,
    pub inhalt: String,
    pub anhang_name: Option<String>,
    pub anhang_typ: Option<String>,
    pub anhang_daten: Option<String>,
    pub zeitpunkt: String,
}

/// Antwort der Live-Abfrage: neue Nachrichten seit der zuletzt bekannten und wer gerade tippt.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatLiveStatus {
    pub nachrichten_version: u64,
    pub neue_nachrichten: Vec<ChatNachricht>,
    pub tippende: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatUebersicht {
    pub aktueller_benutzer_id: Uuid,
    pub alle_teilnehmer: Vec<ChatTeilnehmer>,
    pub chats: Vec<ChatRaum>,
}

#[cfg(feature = "ssr")]
mod store {
    use super::*;
    use std::sync::{LazyLock, Mutex};

    #[derive(Clone, Debug)]
    pub struct StoredChat {
        pub id: Uuid,
        pub name: String,
        pub ist_gruppe: bool,
    }

    #[derive(Clone, Debug)]
    pub struct StoredRelBenutzerChat {
        pub benutzer_id: Uuid,
        pub chat_id: Uuid,
    }

    pub static CHATS_MEMORY: LazyLock<Mutex<Vec<StoredChat>>> =
        LazyLock::new(|| Mutex::new(Vec::new()));
    pub static REL_BENUTZER_CHAT_MEMORY: LazyLock<Mutex<Vec<StoredRelBenutzerChat>>> =
        LazyLock::new(|| Mutex::new(Vec::new()));
    pub static NACHRICHTEN_MEMORY: LazyLock<Mutex<Vec<ChatNachricht>>> =
        LazyLock::new(|| Mutex::new(Vec::new()));
}

/// Prozessweiter Live-Zustand für Echtzeit-Updates (neue Nachrichten & Tipp-Anzeige).
/// Wartende Live-Abfragen werden über den `watch`-Kanal geweckt, sobald sich etwas ändert.
#[cfg(feature = "ssr")]
mod live {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{LazyLock, Mutex};
    use std::time::{Duration, Instant};
    use tokio::sync::watch;

    /// Wie lange ein Tipp-Signal ohne Erneuerung gültig bleibt.
    pub const TIPP_TIMEOUT: Duration = Duration::from_secs(5);

    pub static SIGNAL: LazyLock<watch::Sender<u64>> = LazyLock::new(|| watch::Sender::new(0));
    static VERSIONEN: LazyLock<Mutex<HashMap<Uuid, u64>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));
    static TIPPEN: LazyLock<Mutex<HashMap<Uuid, HashMap<Uuid, (String, Instant)>>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));

    fn wecken() {
        SIGNAL.send_modify(|v| *v = v.wrapping_add(1));
    }

    pub fn version(chat_id: Uuid) -> u64 {
        VERSIONEN
            .lock()
            .map(|g| g.get(&chat_id).copied().unwrap_or(0))
            .unwrap_or(0)
    }

    /// Neue Nachricht im Chat: Version erhöhen, Tipp-Status des Absenders löschen, Wartende wecken.
    pub fn nachricht_gesendet(chat_id: Uuid, benutzer_id: Uuid) {
        if let Ok(mut g) = VERSIONEN.lock() {
            *g.entry(chat_id).or_insert(0) += 1;
        }
        if let Ok(mut g) = TIPPEN.lock() {
            if let Some(chat) = g.get_mut(&chat_id) {
                chat.remove(&benutzer_id);
            }
        }
        wecken();
    }

    pub fn setze_tippen(chat_id: Uuid, benutzer_id: Uuid, benutzername: String, tippt: bool) {
        if let Ok(mut g) = TIPPEN.lock() {
            let chat = g.entry(chat_id).or_default();
            if tippt {
                chat.insert(benutzer_id, (benutzername, Instant::now()));
            } else {
                chat.remove(&benutzer_id);
            }
        }
        wecken();
    }

    /// Alle Benutzer (außer `ich`), die im Chat gerade tippen – alphabetisch sortiert.
    pub fn tippende(chat_id: Uuid, ich: Uuid) -> Vec<String> {
        let Ok(mut g) = TIPPEN.lock() else {
            return Vec::new();
        };
        let Some(chat) = g.get_mut(&chat_id) else {
            return Vec::new();
        };
        chat.retain(|_, (_, seit)| seit.elapsed() < TIPP_TIMEOUT);
        let mut namen: Vec<String> = chat
            .iter()
            .filter(|(id, _)| **id != ich)
            .map(|(_, (name, _))| name.clone())
            .collect();
        namen.sort();
        namen
    }
}

#[cfg(feature = "ssr")]
async fn ist_chat_mitglied(chat_id: Uuid, benutzer_id: Uuid) -> Result<bool, ServerFnError> {
    use sqlx::PgPool;

    if let Some(pool) = use_context::<PgPool>() {
        return Ok(sqlx::query(
            "SELECT 1 FROM rel_benutzer_chat WHERE chat_id = $1 AND benutzer_id = $2",
        )
        .bind(chat_id)
        .bind(benutzer_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("DB-Fehler: {e}")))?
        .is_some());
    }

    let rel_guard = store::REL_BENUTZER_CHAT_MEMORY
        .lock()
        .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
    Ok(rel_guard
        .iter()
        .any(|r| r.chat_id == chat_id && r.benutzer_id == benutzer_id))
}

/// Lädt die Nachrichten eines Chats – alle, oder nur die nach `nach_id` (für Live-Updates).
#[cfg(feature = "ssr")]
async fn lade_nachrichten(
    chat_id: Uuid,
    nach_id: Option<Uuid>,
) -> Result<Vec<ChatNachricht>, ServerFnError> {
    use sqlx::{PgPool, Row};

    if let Some(pool) = use_context::<PgPool>() {
        let rows = sqlx::query(
            r#"
            SELECT
                n.id,
                n.chat_id,
                n.benutzer_id,
                u.benutzername,
                n.inhalt,
                n.anhang_name,
                n.anhang_typ,
                n.anhang_daten,
                n.erstellt_am
            FROM nachricht n
            INNER JOIN benutzer u ON n.benutzer_id = u.id
            WHERE n.chat_id = $1
              AND (
                $2::uuid IS NULL
                OR (n.id <> $2 AND n.erstellt_am >= (SELECT erstellt_am FROM nachricht WHERE id = $2))
              )
            ORDER BY n.erstellt_am ASC
            "#,
        )
        .bind(chat_id)
        .bind(nach_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("DB-Fehler (nachricht): {e}")))?;

        return Ok(rows
            .into_iter()
            .map(|r| {
                let ts: DateTime<Utc> = r.get("erstellt_am");
                let local_ts: DateTime<Local> = DateTime::from(ts);
                ChatNachricht {
                    id: r.get("id"),
                    chat_id: r.get("chat_id"),
                    benutzer_id: r.get("benutzer_id"),
                    benutzername: r.get("benutzername"),
                    inhalt: r.get("inhalt"),
                    anhang_name: r.get("anhang_name"),
                    anhang_typ: r.get("anhang_typ"),
                    anhang_daten: r.get("anhang_daten"),
                    zeitpunkt: local_ts.format("%d.%m.%Y %H:%M").to_string(),
                }
            })
            .collect());
    }

    // In-Memory-Fallback
    let guard = store::NACHRICHTEN_MEMORY
        .lock()
        .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
    let im_chat = guard.iter().filter(|m| m.chat_id == chat_id);
    Ok(match nach_id {
        None => im_chat.cloned().collect(),
        Some(nid) => im_chat
            .skip_while(|m| m.id != nid)
            .skip(1)
            .cloned()
            .collect(),
    })
}

#[cfg(feature = "ssr")]
async fn load_chat_teilnehmer_db(
    pool: &sqlx::PgPool,
    chat_id: Uuid,
) -> Result<Vec<ChatTeilnehmer>, ServerFnError> {
    use sqlx::Row;

    let rows = sqlx::query(
        r#"
        SELECT u.id, u.benutzername
        FROM rel_benutzer_chat r
        INNER JOIN benutzer u ON r.benutzer_id = u.id
        WHERE r.chat_id = $1
        ORDER BY u.benutzername ASC
        "#,
    )
    .bind(chat_id)
    .fetch_all(pool)
    .await
    .map_err(|e| ServerFnError::new(format!("DB-Fehler (rel_benutzer_chat): {e}")))?;

    Ok(rows
        .into_iter()
        .map(|r| ChatTeilnehmer {
            id: r.get("id"),
            benutzername: r.get("benutzername"),
        })
        .collect())
}

#[server(GetChatUebersicht, "/api", endpoint = "get_chat_uebersicht")]
pub async fn get_chat_uebersicht() -> Result<ChatUebersicht, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::{PgPool, Row};

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            let user_rows = sqlx::query(
                "SELECT id, benutzername FROM benutzer ORDER BY benutzername ASC",
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Fehler (benutzer): {e}")))?;

            let alle_teilnehmer: Vec<ChatTeilnehmer> = user_rows
                .into_iter()
                .map(|r| ChatTeilnehmer {
                    id: r.get("id"),
                    benutzername: r.get("benutzername"),
                })
                .collect();

            let chat_rows = sqlx::query(
                r#"
                SELECT c.id, c.name, c.ist_gruppe
                FROM chat c
                INNER JOIN rel_benutzer_chat r ON c.id = r.chat_id
                WHERE r.benutzer_id = $1
                ORDER BY c.ist_gruppe DESC, c.erstellt_am DESC
                "#,
            )
            .bind(current_user.id)
            .fetch_all(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Fehler (chat): {e}")))?;

            let mut chats = Vec::with_capacity(chat_rows.len());
            for row in chat_rows {
                let cid: Uuid = row.get("id");
                let teilnehmer = load_chat_teilnehmer_db(&pool, cid).await?;
                chats.push(ChatRaum {
                    id: cid,
                    name: row.get("name"),
                    ist_gruppe: row.get("ist_gruppe"),
                    teilnehmer,
                });
            }

            return Ok(ChatUebersicht {
                aktueller_benutzer_id: current_user.id,
                alle_teilnehmer,
                chats,
            });
        }

        // In-Memory-Fallback
        let alle_teilnehmer: Vec<ChatTeilnehmer> = {
            let users_guard = crate::backend::auth::store::USERS_MEMORY
                .lock()
                .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
            users_guard
                .iter()
                .map(|u| ChatTeilnehmer {
                    id: u.id,
                    benutzername: u.benutzername.clone(),
                })
                .collect()
        };

        let chats_guard = store::CHATS_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        let rel_guard = store::REL_BENUTZER_CHAT_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;

        let my_chat_ids: Vec<Uuid> = rel_guard
            .iter()
            .filter(|r| r.benutzer_id == current_user.id)
            .map(|r| r.chat_id)
            .collect();

        let chats = chats_guard
            .iter()
            .filter(|c| my_chat_ids.contains(&c.id))
            .map(|c| {
                let member_ids: Vec<Uuid> = rel_guard
                    .iter()
                    .filter(|r| r.chat_id == c.id)
                    .map(|r| r.benutzer_id)
                    .collect();
                let teilnehmer = alle_teilnehmer
                    .iter()
                    .filter(|t| member_ids.contains(&t.id))
                    .cloned()
                    .collect();
                ChatRaum {
                    id: c.id,
                    name: c.name.clone(),
                    ist_gruppe: c.ist_gruppe,
                    teilnehmer,
                }
            })
            .collect();

        Ok(ChatUebersicht {
            aktueller_benutzer_id: current_user.id,
            alle_teilnehmer,
            chats,
        })
    }
}

#[server(OpenDirectChat, "/api", endpoint = "open_direct_chat")]
pub async fn open_direct_chat(partner_id: Uuid) -> Result<ChatRaum, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::{PgPool, Row};

        let current_user = require_current_user().await?;

        if let Some(pool) = use_context::<PgPool>() {
            let partner_row = sqlx::query(
                "SELECT id, benutzername FROM benutzer WHERE id = $1",
            )
            .bind(partner_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Fehler: {e}")))?
            .ok_or_else(|| ServerFnError::new("Chatteilnehmer nicht gefunden."))?;

            let partner_name: String = partner_row.get("benutzername");

            // Prüfen, ob bereits ein Direktchat (ist_gruppe = false) zwischen beiden Benutzern existiert
            let existing_chat = if partner_id == current_user.id {
                sqlx::query(
                    r#"
                    SELECT c.id, c.name, c.ist_gruppe
                    FROM chat c
                    INNER JOIN rel_benutzer_chat r ON c.id = r.chat_id
                    WHERE c.ist_gruppe = FALSE AND r.benutzer_id = $1
                    GROUP BY c.id
                    HAVING COUNT(r.benutzer_id) = 1
                    LIMIT 1
                    "#,
                )
                .bind(current_user.id)
                .fetch_optional(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Fehler: {e}")))?
            } else {
                sqlx::query(
                    r#"
                    SELECT c.id, c.name, c.ist_gruppe
                    FROM chat c
                    INNER JOIN rel_benutzer_chat r1 ON c.id = r1.chat_id AND r1.benutzer_id = $1
                    INNER JOIN rel_benutzer_chat r2 ON c.id = r2.chat_id AND r2.benutzer_id = $2
                    WHERE c.ist_gruppe = FALSE
                    LIMIT 1
                    "#,
                )
                .bind(current_user.id)
                .bind(partner_id)
                .fetch_optional(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Fehler: {e}")))?
            };

            if let Some(row) = existing_chat {
                let cid: Uuid = row.get("id");
                let teilnehmer = load_chat_teilnehmer_db(&pool, cid).await?;
                return Ok(ChatRaum {
                    id: cid,
                    name: row.get("name"),
                    ist_gruppe: row.get("ist_gruppe"),
                    teilnehmer,
                });
            }

            // Neuen Direktchat anlegen
            let new_chat_id = Uuid::new_v4();
            let chat_name = if partner_id == current_user.id {
                format!("{} (Notizen)", current_user.benutzername)
            } else {
                format!("{} & {}", current_user.benutzername, partner_name)
            };

            sqlx::query("INSERT INTO chat (id, name, ist_gruppe) VALUES ($1, $2, FALSE)")
                .bind(new_chat_id)
                .bind(&chat_name)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Insert (chat) fehlgeschlagen: {e}")))?;

            sqlx::query(
                "INSERT INTO rel_benutzer_chat (benutzer_id, chat_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(current_user.id)
            .bind(new_chat_id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Insert (rel) fehlgeschlagen: {e}")))?;

            if partner_id != current_user.id {
                sqlx::query(
                    "INSERT INTO rel_benutzer_chat (benutzer_id, chat_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                )
                .bind(partner_id)
                .bind(new_chat_id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Insert (rel) fehlgeschlagen: {e}")))?;
            }

            let teilnehmer = load_chat_teilnehmer_db(&pool, new_chat_id).await?;
            return Ok(ChatRaum {
                id: new_chat_id,
                name: chat_name,
                ist_gruppe: false,
                teilnehmer,
            });
        }

        // In-Memory-Fallback
        let alle_teilnehmer: Vec<ChatTeilnehmer> = {
            let users_guard = crate::backend::auth::store::USERS_MEMORY
                .lock()
                .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
            users_guard
                .iter()
                .map(|u| ChatTeilnehmer {
                    id: u.id,
                    benutzername: u.benutzername.clone(),
                })
                .collect()
        };

        let partner = alle_teilnehmer
            .iter()
            .find(|u| u.id == partner_id)
            .cloned()
            .ok_or_else(|| ServerFnError::new("Chatteilnehmer nicht gefunden."))?;

        let mut chats_guard = store::CHATS_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        let mut rel_guard = store::REL_BENUTZER_CHAT_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;

        for c in chats_guard.iter().filter(|c| !c.ist_gruppe) {
            let members: Vec<Uuid> = rel_guard
                .iter()
                .filter(|r| r.chat_id == c.id)
                .map(|r| r.benutzer_id)
                .collect();
            let matches = if partner_id == current_user.id {
                members.len() == 1 && members.contains(&current_user.id)
            } else {
                members.len() == 2
                    && members.contains(&current_user.id)
                    && members.contains(&partner_id)
            };
            if matches {
                let teilnehmer = alle_teilnehmer
                    .iter()
                    .filter(|t| members.contains(&t.id))
                    .cloned()
                    .collect();
                return Ok(ChatRaum {
                    id: c.id,
                    name: c.name.clone(),
                    ist_gruppe: false,
                    teilnehmer,
                });
            }
        }

        let new_chat_id = Uuid::new_v4();
        let chat_name = if partner_id == current_user.id {
            format!("{} (Notizen)", current_user.benutzername)
        } else {
            format!("{} & {}", current_user.benutzername, partner.benutzername)
        };

        chats_guard.push(store::StoredChat {
            id: new_chat_id,
            name: chat_name.clone(),
            ist_gruppe: false,
        });
        rel_guard.push(store::StoredRelBenutzerChat {
            benutzer_id: current_user.id,
            chat_id: new_chat_id,
        });
        if partner_id != current_user.id {
            rel_guard.push(store::StoredRelBenutzerChat {
                benutzer_id: partner_id,
                chat_id: new_chat_id,
            });
        }

        let mut teilnehmer = vec![ChatTeilnehmer {
            id: current_user.id,
            benutzername: current_user.benutzername,
        }];
        if partner_id != current_user.id {
            teilnehmer.push(partner);
        }

        Ok(ChatRaum {
            id: new_chat_id,
            name: chat_name,
            ist_gruppe: false,
            teilnehmer,
        })
    }
}

#[server(
    CreateGroupChat,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "create_group_chat"
)]
pub async fn create_group_chat(
    name: String,
    teilnehmer_ids: Vec<Uuid>,
) -> Result<ChatRaum, ServerFnError> {
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        return Err(ServerFnError::new(
            "Bitte einen Namen für den Gruppenchat eingeben.",
        ));
    }

    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let current_user = require_current_user().await?;

        let mut all_member_ids = teilnehmer_ids;
        if !all_member_ids.contains(&current_user.id) {
            all_member_ids.push(current_user.id);
        }
        all_member_ids.sort();
        all_member_ids.dedup();

        let new_chat_id = Uuid::new_v4();

        if let Some(pool) = use_context::<PgPool>() {
            sqlx::query("INSERT INTO chat (id, name, ist_gruppe) VALUES ($1, $2, TRUE)")
                .bind(new_chat_id)
                .bind(&trimmed)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Insert (chat) fehlgeschlagen: {e}")))?;

            for uid in &all_member_ids {
                sqlx::query(
                    "INSERT INTO rel_benutzer_chat (benutzer_id, chat_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                )
                .bind(uid)
                .bind(new_chat_id)
                .execute(&pool)
                .await
                .map_err(|e| ServerFnError::new(format!("DB-Insert (rel_benutzer_chat) fehlgeschlagen: {e}")))?;
            }

            let teilnehmer = load_chat_teilnehmer_db(&pool, new_chat_id).await?;
            return Ok(ChatRaum {
                id: new_chat_id,
                name: trimmed,
                ist_gruppe: true,
                teilnehmer,
            });
        }

        // In-Memory-Fallback
        let alle_teilnehmer: Vec<ChatTeilnehmer> = {
            let users_guard = crate::backend::auth::store::USERS_MEMORY
                .lock()
                .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
            users_guard
                .iter()
                .map(|u| ChatTeilnehmer {
                    id: u.id,
                    benutzername: u.benutzername.clone(),
                })
                .collect()
        };

        let mut chats_guard = store::CHATS_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        let mut rel_guard = store::REL_BENUTZER_CHAT_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;

        chats_guard.push(store::StoredChat {
            id: new_chat_id,
            name: trimmed.clone(),
            ist_gruppe: true,
        });

        for uid in &all_member_ids {
            rel_guard.push(store::StoredRelBenutzerChat {
                benutzer_id: *uid,
                chat_id: new_chat_id,
            });
        }

        let teilnehmer = alle_teilnehmer
            .into_iter()
            .filter(|t| all_member_ids.contains(&t.id))
            .collect();

        return Ok(ChatRaum {
            id: new_chat_id,
            name: trimmed,
            ist_gruppe: true,
            teilnehmer,
        });
    }

    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

#[server(GetChatNachrichten, "/api", endpoint = "get_chat_nachrichten")]
pub async fn get_chat_nachrichten(chat_id: Uuid) -> Result<Vec<ChatNachricht>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;

        let current_user = require_current_user().await?;
        if !ist_chat_mitglied(chat_id, current_user.id).await? {
            return Err(ServerFnError::new("Kein Zugriff auf diesen Chat."));
        }
        lade_nachrichten(chat_id, None).await
    }
}

/// Long-Polling-Endpunkt für Echtzeit-Updates eines Chats.
///
/// Antwortet sofort, wenn sich der Zustand gegenüber dem Client-Stand (`nachrichten_version`,
/// `tipp_signatur`) unterscheidet, und wartet sonst bis zu ~20 s auf eine Änderung.
#[server(
    ChatLiveAbfrage,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "chat_live_abfrage"
)]
pub async fn chat_live_abfrage(
    chat_id: Uuid,
    letzte_nachricht_id: Option<Uuid>,
    nachrichten_version: Option<u64>,
    tipp_signatur: String,
) -> Result<ChatLiveStatus, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use std::time::Duration;

        let current_user = require_current_user().await?;
        if !ist_chat_mitglied(chat_id, current_user.id).await? {
            return Err(ServerFnError::new("Kein Zugriff auf diesen Chat."));
        }

        let mut rx = live::SIGNAL.subscribe();
        rx.borrow_and_update();

        let mut version = live::version(chat_id);
        let mut tippende = live::tippende(chat_id, current_user.id);

        // Max. 20 Runden à 1 s – kurze Runden, damit abgelaufene Tipp-Signale auch ohne Weckruf verschwinden
        for _ in 0..20 {
            if nachrichten_version != Some(version) || tippende.join("|") != tipp_signatur {
                break;
            }
            let _ = tokio::time::timeout(Duration::from_secs(1), rx.changed()).await;
            version = live::version(chat_id);
            tippende = live::tippende(chat_id, current_user.id);
        }

        let neue_nachrichten = if nachrichten_version != Some(version) {
            lade_nachrichten(chat_id, letzte_nachricht_id).await?
        } else {
            Vec::new()
        };

        Ok(ChatLiveStatus {
            nachrichten_version: version,
            neue_nachrichten,
            tippende,
        })
    }

    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

/// Meldet, ob der aktuelle Benutzer im Chat gerade eine Nachricht schreibt.
#[server(SetChatTippstatus, "/api", endpoint = "set_chat_tippstatus")]
pub async fn set_chat_tippstatus(chat_id: Uuid, tippt: bool) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;

        let current_user = require_current_user().await?;
        if !ist_chat_mitglied(chat_id, current_user.id).await? {
            return Err(ServerFnError::new("Kein Zugriff auf diesen Chat."));
        }
        live::setze_tippen(chat_id, current_user.id, current_user.benutzername, tippt);
        Ok(())
    }
}

#[server(
    SendChatNachricht,
    "/api",
    input = leptos::server_fn::codec::Json,
    endpoint = "send_chat_nachricht"
)]
pub async fn send_chat_nachricht(
    chat_id: Uuid,
    inhalt: String,
    anhang_name: Option<String>,
    anhang_typ: Option<String>,
    anhang_daten: Option<String>,
) -> Result<ChatNachricht, ServerFnError> {
    let trimmed = inhalt.trim().to_string();
    let has_attachment = anhang_daten
        .as_ref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);

    if trimmed.is_empty() && !has_attachment {
        return Err(ServerFnError::new(
            "Bitte eine Nachricht eingeben oder eine Datei anhängen.",
        ));
    }

    #[cfg(feature = "ssr")]
    {
        use crate::backend::auth::require_current_user;
        use sqlx::PgPool;

        let current_user = require_current_user().await?;
        let now_local = Local::now();
        let nachricht_id = Uuid::new_v4();

        if let Some(pool) = use_context::<PgPool>() {
            let is_member = sqlx::query(
                "SELECT 1 FROM rel_benutzer_chat WHERE chat_id = $1 AND benutzer_id = $2",
            )
            .bind(chat_id)
            .bind(current_user.id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Fehler: {e}")))?
            .is_some();

            if !is_member {
                return Err(ServerFnError::new("Kein Zugriff auf diesen Chat."));
            }

            sqlx::query(
                r#"
                INSERT INTO nachricht (
                    id,
                    chat_id,
                    benutzer_id,
                    inhalt,
                    anhang_name,
                    anhang_typ,
                    anhang_daten
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(nachricht_id)
            .bind(chat_id)
            .bind(current_user.id)
            .bind(&trimmed)
            .bind(&anhang_name)
            .bind(&anhang_typ)
            .bind(&anhang_daten)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB-Insert (nachricht) fehlgeschlagen: {e}")))?;

            live::nachricht_gesendet(chat_id, current_user.id);

            return Ok(ChatNachricht {
                id: nachricht_id,
                chat_id,
                benutzer_id: current_user.id,
                benutzername: current_user.benutzername,
                inhalt: trimmed,
                anhang_name,
                anhang_typ,
                anhang_daten,
                zeitpunkt: now_local.format("%d.%m.%Y %H:%M").to_string(),
            });
        }

        // In-Memory-Fallback
        let msg = ChatNachricht {
            id: nachricht_id,
            chat_id,
            benutzer_id: current_user.id,
            benutzername: current_user.benutzername,
            inhalt: trimmed,
            anhang_name,
            anhang_typ,
            anhang_daten,
            zeitpunkt: now_local.format("%d.%m.%Y %H:%M").to_string(),
        };

        let mut guard = store::NACHRICHTEN_MEMORY
            .lock()
            .map_err(|_| ServerFnError::new("Speicher-Lock fehlgeschlagen"))?;
        guard.push(msg.clone());
        drop(guard);
        live::nachricht_gesendet(chat_id, msg.benutzer_id);
        return Ok(msg);
    }

    #[cfg(not(feature = "ssr"))]
    unreachable!()
}
