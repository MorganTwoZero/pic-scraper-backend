FROM node:lts-alpine AS frontend
WORKDIR /app

COPY frontend/package*.json ./
RUN --mount=type=cache,target=/root/.npm \
    npm ci

COPY frontend .
ARG VITE_APP_BACKEND_URL=https://morgan20.ru/api
RUN npm run build

FROM clux/muslrust:stable AS chef
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    cargo install cargo-chef --locked

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY backend ./backend
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ENV SQLX_OFFLINE=true

COPY --from=planner /volume/recipe.json recipe.json
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

COPY Cargo.toml Cargo.lock ./
COPY .sqlx ./.sqlx
COPY backend ./backend
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    cargo build --release --target x86_64-unknown-linux-musl --bin api --bin scraper

FROM gcr.io/distroless/static:nonroot AS final
COPY --from=builder --chown=nonroot:nonroot /volume/target/x86_64-unknown-linux-musl/release/api /api
COPY --from=builder --chown=nonroot:nonroot /volume/target/x86_64-unknown-linux-musl/release/scraper /scraper
COPY backend/migrations ./backend/migrations
COPY --from=frontend /app/dist ./frontend/dist
