# CLAUDE.md – Regelha

Persönliche Haushalts- und Organisations-App: Budget-Dashboard, Buchungen, Fixkosten,
Schulden, Aufgaben, Kontakte und Chat. Fullstack in Rust mit **Leptos 0.8** (SSR + Hydration), **Axum 0.8**,
**sqlx 0.8 / PostgreSQL**, **Tailwind CSS v4 + DaisyUI v5**. Deployment auf Railway.

## Sprache und Stil

- Mit dem Benutzer **Deutsch** sprechen, einfach und klar.
- Oberfläche, Kommentare und Fachbegriffe auf Deutsch (`Buchung`, `Fixkosten`, `Aufgabe`, `Nachricht`).
  Server-Funktionen heissen gemischt (`GetBuchungen`, `AddAufgabe`) – bestehendes Muster beibehalten.
- Commit-Nachrichten im Stil `feat: …` / `fix: …` mit deutscher Beschreibung.

## Wichtige Befehle

Immer ein Feature angeben (`ssr` für Server, `hydrate` für Browser).

```bash
cargo check --features ssr                                              # Server
cargo check --lib --features hydrate --target wasm32-unknown-unknown   # Browser (WASM)
cargo clippy --features ssr
cargo leptos watch                                                      # lokal starten (Port 3000)
cargo leptos build --release                                            # Release-Build
npm install                                                             # Tailwind + DaisyUI
```

- Ohne `DATABASE_URL` läuft die App mit **In-Memory-Speicher** (Beispieldaten in `LazyLock<Mutex<…>>`).
  Mit `DATABASE_URL` legt sie die Datenbank bei Bedarf an und erstellt die Tabellen selbst.
- Es gibt **keinen `migrations/`-Ordner**: das Schema steht in `src/backend/db.rs::run_schema_init`
  (`CREATE TABLE IF NOT EXISTS …`). Schemaänderungen dort rückwärtskompatibel ergänzen
  (`ALTER TABLE … ADD COLUMN IF NOT EXISTS`), bestehende Daten nicht löschen.
- Testbenutzer im In-Memory-Modus: `admin/admin123`, `anna/anna123`, `lukas/lukas123`.

## Aufbau

```text
src/
├── main.rs            Axum-Server, PORT von Railway, PgPool als Leptos-Kontext
├── lib.rs             Module, hydrate()
├── app.rs             shell(), Routen: / (Startseite mit allen Modulen) · /dashboard · /buchungen · /fixkosten · /monatskosten · /schulden · /aufgaben · /fasten · /kontakte · /chat
├── backend/
│   ├── mod.rs         register_server_functions() – jede neue #[server]-Funktion hier eintragen!
│   ├── db.rs          Pool, Datenbank anlegen, Tabellen erstellen          [ssr]
│   ├── auth.rs        Anmeldung, Sitzung, Passwort-Hash
│   ├── budget.rs      Buchungen, Fixkosten, Dashboard-Kennzahlen
│   ├── aufgaben.rs    Aufgaben
│   ├── kontakte.rs    Kontakte (Vorname, Nachname)
│   ├── schulden.rs    Schulden (Datum, Grund, Betrag, Kontakt 1:N)
│   ├── rueckzahlungen.rs  Rückzahlungen (Datum, Betrag, Kontakt N:1), Reiter unter Schulden
│   ├── fasten.rs      Fasten (Beginn, Ende als Zeitpunkt; Dauer wird berechnet)
│   └── chat.rs        Direkt- und Gruppenchats, Live-Abfrage, Tipp-Anzeige
└── components/        app_layout, header, sideboard, startseite, dashboard, buchungen, fixkosten, monatskosten, schulden, rueckzahlungen, aufgaben, fasten, kontakte, chat, login
style/tailwind.css     Tailwind-Einstieg, Farben (Primär #66023C)
```

## Regeln

- **Jede neue Server-Funktion** in `backend/mod.rs::register_server_functions` registrieren,
  sonst findet der Server sie nicht.
- Jede Server-Funktion unterstützt **beide Speicher**: `if let Some(pool) = use_context::<PgPool>()`
  → SQL, sonst In-Memory-Liste. Beide Zweige gleich verhalten lassen.
- Hydration: Server- und Browser-Ausgabe müssen gleich sein (früherer Fehler: doppelt gebautes Layout
  beim SSR). Browser-only-Code hinter `#[cfg(feature = "hydrate")]` oder in `Effect`.
- Mobilansicht mitdenken (Chat und Layout sind für Smartphones optimiert).
- Nach Änderungen mindestens `cargo check --features ssr` und, wenn möglich, die WASM-Prüfung ausführen.
  Neue Warnungen beheben.

## Cloud-Umgebung (Claude Code im Web)

- Rust, Node 22, `psql` und Docker sind installiert. Das WASM-Ziel ist evtl. nicht installiert:
  `rustup target add wasm32-unknown-unknown`.
- Kein PostgreSQL-Server nötig: ohne `DATABASE_URL` läuft alles im In-Memory-Modus.
- Der Container ist flüchtig: Arbeit committen und auf den vorgegebenen Branch pushen.
- Deployment: `Dockerfile` + `railway.toml` (Healthcheck auf `/`).
