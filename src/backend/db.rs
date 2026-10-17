use leptos::logging::log;
use sqlx::{migrate::MigrateDatabase, postgres::PgPoolOptions, PgPool, Postgres, Row};

pub async fn init_db_pool() -> Option<PgPool> {
    if let Ok(database_url) = std::env::var("DATABASE_URL") {
        match Postgres::database_exists(&database_url).await {
            Ok(false) => {
                log!("Datenbank existiert noch nicht – erstelle Datenbank automatisch...");
                if let Err(err) = Postgres::create_database(&database_url).await {
                    log!("Warnung: Datenbank konnte nicht automatisch angelegt werden ({err}).");
                } else {
                    log!("Datenbank erfolgreich angelegt.");
                }
            }
            Ok(true) => {}
            Err(err) => {
                log!("Warnung: Prüfung der Datenbank fehlgeschlagen ({err}).");
            }
        }

        match PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
        {
            Ok(pool) => {
                if let Err(err) = run_schema_init(&pool).await {
                    log!("Warnung: Tabellen konnten nicht initialisiert werden: {err}");
                } else {
                    log!("PostgreSQL-Verbindung und Tabellen (benutzer, buchungen, fixkosten, aufgaben, chat, rel_benutzer_chat, nachricht) bereit.");
                }
                Some(pool)
            }
            Err(err) => {
                log!("Warnung: Konnte keine Verbindung zu PostgreSQL herstellen ({err}). Nutze In-Memory-Speicher.");
                None
            }
        }
    } else {
        log!("Hinweis: Keine DATABASE_URL gesetzt. Server nutzt In-Memory-Speicher.");
        None
    }
}

async fn run_schema_init(pool: &PgPool) -> Result<(), sqlx::Error> {
    // 1. Tabelle `benutzer` zuerst erstellen (1-Seite der N:1-Relation)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS benutzer (
            id UUID PRIMARY KEY,
            benutzername TEXT UNIQUE NOT NULL,
            passwort_hash TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Standard-Benutzer "admin" (Passwort: "admin123") anlegen bzw. Hash aktualisieren
    let default_admin_id = crate::backend::auth::default_admin_uuid();
    let default_hash = crate::backend::auth::hash_password("admin", "admin123");
    sqlx::query(
        r#"
        INSERT INTO benutzer (id, benutzername, passwort_hash)
        VALUES ($1, $2, $3)
        ON CONFLICT (benutzername) DO UPDATE SET passwort_hash = EXCLUDED.passwort_hash;
        "#,
    )
    .bind(default_admin_id)
    .bind("admin")
    .bind(default_hash)
    .execute(pool)
    .await?;

    // Zusätzliche Beispiel-Chatteilnehmer anlegen, falls noch nicht vorhanden
    for (uid_u128, uname, upw) in [
        (0x00000000_0000_4000_8000_000000000002u128, "anna", "anna123"),
        (0x00000000_0000_4000_8000_000000000003u128, "lukas", "lukas123"),
    ] {
        let uid = uuid::Uuid::from_u128(uid_u128);
        let uhash = crate::backend::auth::hash_password(uname, upw);
        sqlx::query(
            r#"
            INSERT INTO benutzer (id, benutzername, passwort_hash)
            VALUES ($1, $2, $3)
            ON CONFLICT (benutzername) DO NOTHING;
            "#,
        )
        .bind(uid)
        .bind(uname)
        .bind(uhash)
        .execute(pool)
        .await?;
    }

    // Tatsächliche ID des "admin"-Benutzers abrufen (für Migration bestehender Datensätze)
    let admin_row = sqlx::query("SELECT id FROM benutzer WHERE benutzername = 'admin'")
        .fetch_one(pool)
        .await?;
    let admin_id: uuid::Uuid = admin_row.get("id");

    // 2. Tabelle `buchungen` mit N:1-Fremdschlüssel `benutzer_id` zur Tabelle `benutzer`
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS buchungen (
            id UUID PRIMARY KEY,
            buchungstext TEXT NOT NULL,
            betrag DOUBLE PRECISION NOT NULL,
            art TEXT NOT NULL,
            erstellt_am DATE NOT NULL DEFAULT CURRENT_DATE,
            benutzer_id UUID NOT NULL REFERENCES benutzer(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        ALTER TABLE buchungen
        ADD COLUMN IF NOT EXISTS erstellt_am DATE NOT NULL DEFAULT CURRENT_DATE;
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        ALTER TABLE buchungen
        ADD COLUMN IF NOT EXISTS benutzer_id UUID REFERENCES benutzer(id) ON DELETE CASCADE;
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query("UPDATE buchungen SET benutzer_id = $1 WHERE benutzer_id IS NULL;")
        .bind(admin_id)
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE buchungen ALTER COLUMN benutzer_id SET NOT NULL;")
        .execute(pool)
        .await?;

    // 3. Tabelle `fixkosten` mit N:1-Fremdschlüssel `benutzer_id` zur Tabelle `benutzer`
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS fixkosten (
            id UUID PRIMARY KEY,
            buchungstext TEXT NOT NULL,
            betrag DOUBLE PRECISION NOT NULL,
            benutzer_id UUID NOT NULL REFERENCES benutzer(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        ALTER TABLE fixkosten
        ADD COLUMN IF NOT EXISTS benutzer_id UUID REFERENCES benutzer(id) ON DELETE CASCADE;
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query("UPDATE fixkosten SET benutzer_id = $1 WHERE benutzer_id IS NULL;")
        .bind(admin_id)
        .execute(pool)
        .await?;

    sqlx::query("ALTER TABLE fixkosten ALTER COLUMN benutzer_id SET NOT NULL;")
        .execute(pool)
        .await?;

    // 4. Tabelle `aufgaben` (Felder: beschreibung, datum, erledigt) mit N:1-Relation zur Tabelle `benutzer`
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS aufgaben (
            id UUID PRIMARY KEY,
            beschreibung TEXT NOT NULL,
            datum DATE NOT NULL DEFAULT CURRENT_DATE,
            erledigt BOOLEAN NOT NULL DEFAULT FALSE,
            benutzer_id UUID NOT NULL REFERENCES benutzer(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        ALTER TABLE aufgaben
        ADD COLUMN IF NOT EXISTS erledigt BOOLEAN NOT NULL DEFAULT FALSE;
        "#,
    )
    .execute(pool)
    .await?;

    // 5. Tabelle `chat`
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS chat (
            id UUID PRIMARY KEY,
            name TEXT NOT NULL,
            ist_gruppe BOOLEAN NOT NULL DEFAULT FALSE,
            erstellt_am TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    // 6. Tabelle `rel_benutzer_chat` (M:N-Beziehung zwischen `benutzer` und `chat`)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS rel_benutzer_chat (
            benutzer_id UUID NOT NULL REFERENCES benutzer(id) ON DELETE CASCADE,
            chat_id UUID NOT NULL REFERENCES chat(id) ON DELETE CASCADE,
            beitritt_am TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            PRIMARY KEY (benutzer_id, chat_id)
        );
        "#,
    )
    .execute(pool)
    .await?;

    // 7. Tabelle `nachricht` (Benutzer - Nachricht 1:N, Chat - Nachricht 1:N)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS nachricht (
            id UUID PRIMARY KEY,
            chat_id UUID NOT NULL REFERENCES chat(id) ON DELETE CASCADE,
            benutzer_id UUID NOT NULL REFERENCES benutzer(id) ON DELETE CASCADE,
            inhalt TEXT NOT NULL DEFAULT '',
            anhang_name TEXT,
            anhang_typ TEXT,
            anhang_daten TEXT,
            erstellt_am TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
