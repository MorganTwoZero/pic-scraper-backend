FROM node:lts-alpine as frontend_builder
WORKDIR /app
COPY ./frontend/vite.config.js .
COPY ./frontend/package*.json .
RUN npm install

FROM frontend_builder as frontend
COPY ./frontend .
ARG VITE_APP_BACKEND_URL
RUN npm run build

FROM clux/muslrust:stable AS planner
RUN cargo install cargo-chef
COPY ./Cargo.lock .
COPY ./Cargo.toml .
COPY ./backend ./backend
ADD https://github.com/protocolbuffers/protobuf/releases/download/v23.3/protoc-23.3-linux-x86_64.zip ./protoc
RUN cargo chef prepare --recipe-path recipe.json

FROM clux/muslrust:stable AS cacher
RUN cargo install cargo-chef
COPY --from=planner /volume/recipe.json recipe.json
ADD https://github.com/protocolbuffers/protobuf/releases/download/v23.3/protoc-23.3-linux-x86_64.zip ./protoc
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

FROM clux/muslrust:stable AS builder
COPY ./backend/migrations ./backend/migrations
COPY ./.sqlx ./.sqlx
COPY ./Cargo.lock .
COPY ./Cargo.toml .
COPY ./backend ./backend
ENV SQLX_OFFLINE true
COPY --from=cacher /volume/target target
COPY --from=cacher /root/.cargo /root/.cargo
RUN cargo build --release --bin pic-scraper-backend --target x86_64-unknown-linux-musl

# Need cacerts
FROM gcr.io/distroless/static:nonroot as final
COPY --from=builder --chown=nonroot:nonroot /volume/target/x86_64-unknown-linux-musl/release/pic-scraper-backend /pic-scraper-backend
COPY ./backend/migrations ./backend/migrations
COPY --from=frontend /app/dist ./frontend/dist
ENV APP_ENVIRONMENT prod
ENTRYPOINT ["/pic-scraper-backend"]