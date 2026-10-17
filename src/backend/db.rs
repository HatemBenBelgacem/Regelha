use leptos::logging::log;
use sqlx::{migrate::MigrateDatabase, postgres::PgPoolOptions, PgPool, Postgres};

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
                    log!("PostgreSQL-Verbindung und Tabellen (buchungen, fixkosten) bereit.");
                }
                Some(pool)
            }
            Err(err) => {
                log!("Warnung: Konnte keine Verbindung zu PostgreSQL herstellen ({err}). Nutze In-Memory-Speicher.");
                None
            }
        }
    } else {
        log!("Hinweis: Keine DATABASE_URL gesetzt. Server nutzt In-Memory-Speicher für Buchungen & Fixkosten.");
        None
    }
}

async fn run_schema_init(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS buchungen (
            id UUID PRIMARY KEY,
            buchungstext TEXT NOT NULL,
            betrag DOUBLE PRECISION NOT NULL,
            art TEXT NOT NULL,
            erstellt_am DATE NOT NULL DEFAULT CURRENT_DATE
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Falls die Tabelle bereits ohne `erstellt_am` existierte, Spalte nachziehen
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
        CREATE TABLE IF NOT EXISTS fixkosten (
            id UUID PRIMARY KEY,
            buchungstext TEXT NOT NULL,
            betrag DOUBLE PRECISION NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
