# =============================================================================
# Stage 1: chef – Rust + Node.js 22 + cargo-leptos + cargo-chef
# (ändert sich selten, bleibt deshalb lange im Cache)
# =============================================================================
FROM rustlang/rust:nightly-bookworm AS chef

# System-Abhängigkeiten + Node.js 22 (Tailwind CSS v4 benötigt Node >= 20) installieren
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    ca-certificates \
    gnupg \
    && curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# WASM-Target installieren
RUN rustup target add wasm32-unknown-unknown

# cargo-leptos und cargo-chef (vorab kompiliertes Binary via cargo-binstall, sonst aus dem Quellcode)
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
    && cargo binstall cargo-leptos cargo-chef -y --force || cargo install --locked cargo-leptos cargo-chef

WORKDIR /app

# =============================================================================
# Stage 2: planner – erstellt recipe.json mit allen Abhängigkeiten.
# Das Rezept ändert sich nur, wenn Cargo.toml oder Cargo.lock sich ändern,
# nicht bei jeder Code-Änderung.
# =============================================================================
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# =============================================================================
# Stage 3: builder – Abhängigkeiten vorab bauen, dann Fullstack Release-Build
# =============================================================================
FROM chef AS builder

# Abhängigkeiten vorab kompilieren (bleibt im Cache, solange sich das Rezept nicht ändert).
# Die Flags entsprechen dem, was `cargo leptos build --release` verwendet
# (bin-features = ssr, lib-features = hydrate, lib-profile-release = wasm-release in Cargo.toml).
# Weicht ein Flag ab, baut cargo die Abhängigkeiten später trotzdem neu.
#  - Server:        Profil release, Feature ssr
#  - WASM-Client:   Profil wasm-release, Target wasm32-unknown-unknown, Feature hydrate
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --no-default-features --features ssr --recipe-path recipe.json
RUN cargo chef cook --profile wasm-release --target wasm32-unknown-unknown --no-default-features --features hydrate --recipe-path recipe.json

# Node-Abhängigkeiten (Tailwind CSS v4 & DaisyUI v5) installieren
COPY package.json package-lock.json* ./
RUN npm install --ignore-scripts

# Projektdateien kopieren (erst jetzt, damit Code-Änderungen die vorab gebauten Abhängigkeiten nicht neu auslösen)
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY style ./style
COPY public ./public

# Node_modules/.bin in PATH aufnehmen, damit cargo-leptos Tailwind + DaisyUI findet
ENV PATH="/app/node_modules/.bin:${PATH}"

# Fullstack Release-Build (Server-Binary + WASM-Bundle + Tailwind/DaisyUI CSS)
RUN cargo leptos build --release -vv

# =============================================================================
# Stage 4: Runtime (Schlankes Produktions-Image für Railway)
# =============================================================================
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Gebautes Server-Binary und statische Site-Assets (WASM/JS/CSS) kopieren
COPY --from=builder /app/target/release/regelha /app/regelha
COPY --from=builder /app/target/site /app/target/site
COPY --from=builder /app/Cargo.toml /app/Cargo.toml

# Standard-Konfiguration für Produktion (Railway überschreibt PORT automatisch)
ENV RUST_LOG="info,regelha=info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="target/site"
ENV PORT="8080"
# SESSION_SECRET (Pflicht, mind. 32 Zeichen) NICHT hier setzen, sondern als Railway-Variable,
# z. B. erzeugt mit `openssl rand -hex 32`. Ohne Secret beendet sich der Server beim Start.

EXPOSE 8080

CMD ["/app/regelha"]
