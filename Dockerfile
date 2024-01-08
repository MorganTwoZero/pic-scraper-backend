FROM node:lts-alpine as frontend_builder
WORKDIR /app
COPY ./frontend/vite.config.js .
COPY ./frontend/package*.json .
RUN npm install

FROM frontend_builder as frontend
COPY ./frontend .
ARG VITE_APP_BACKEND_URL
RUN npm run build

FROM lukemathwalker/cargo-chef:0.1.62-rust-1.74-slim-bookworm AS planner
COPY ./Cargo.lock .
COPY ./Cargo.toml .
COPY ./backend ./backend
RUN cargo chef prepare --recipe-path recipe.json

FROM lukemathwalker/cargo-chef:0.1.62-rust-1.74-slim-bookworm AS cacher
COPY --from=planner /volume/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

FROM lukemathwalker/cargo-chef:0.1.62-rust-1.74-slim-bookworm AS builder
COPY ./backend/migrations ./backend/migrations
COPY ./.sqlx ./.sqlx
COPY ./Cargo.lock .
COPY ./Cargo.toml .
COPY ./backend ./backend
ENV SQLX_OFFLINE true
COPY --from=cacher /volume/target target
COPY --from=cacher /root/.cargo /root/.cargo
RUN cargo build --release --bin api --target x86_64-unknown-linux-musl
RUN cargo build --release --bin scraper --target x86_64-unknown-linux-musl

# Need cacerts
FROM gcr.io/distroless/static:nonroot as final
COPY --from=builder --chown=nonroot:nonroot /volume/target/x86_64-unknown-linux-musl/release/api /api
COPY --from=builder --chown=nonroot:nonroot /volume/target/x86_64-unknown-linux-musl/release/scraper /scraper
COPY ./backend/migrations ./backend/migrations
COPY --from=frontend /app/dist ./frontend/dist
