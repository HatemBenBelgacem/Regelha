# =============================================================================
# Stage 1: Builder (Rust + cargo-leptos + Node.js für Tailwind CSS v4 & DaisyUI)
# =============================================================================
FROM rustlang/rust:nightly-bookworm AS builder

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

# WASM-Target und cargo-leptos (vorab kompiliertes Binary via cargo-binstall) installieren
RUN rustup target add wasm32-unknown-unknown
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
    && cargo binstall cargo-leptos -y --force || cargo install --locked cargo-leptos

WORKDIR /app

# Node-Abhängigkeiten (Tailwind CSS v4 & DaisyUI v5) installieren
COPY package.json package-lock.json* ./
RUN npm install --ignore-scripts

# Projektdateien kopieren
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY style ./style
COPY public ./public

# Node_modules/.bin in PATH aufnehmen, damit cargo-leptos Tailwind + DaisyUI findet
ENV PATH="/app/node_modules/.bin:${PATH}"

# Fullstack Release-Build (Server-Binary + WASM-Bundle + Tailwind/DaisyUI CSS)
RUN cargo leptos build --release -vv

# =============================================================================
# Stage 2: Runtime (Schlankes Produktions-Image für Railway)
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

EXPOSE 8080

CMD ["/app/regelha"]
