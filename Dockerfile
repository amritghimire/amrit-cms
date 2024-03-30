FROM rust:1.77.0-alpine3.19 as chef

RUN apk add --no-cache alpine-sdk
RUN apk add openssl-dev
RUN rustup target add wasm32-unknown-unknown

RUN cargo install cargo-chef
RUN cargo install --locked wasm-bindgen-cli
RUN wget -qO- https://github.com/trunk-rs/trunk/releases/download/v0.19.1/trunk-aarch64-unknown-linux-musl.tar.gz | tar -xzf-
RUN mv ./trunk /usr/bin/

WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef as builder

# This is important, see https://github.com/rust-lang/docker-rust/issues/85
ENV RUSTFLAGS="-C target-feature=-crt-static"
# if needed, add additional dependencies here
RUN apk add --no-cache libgcc
RUN apk add --no-cache musl-dev
WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

ENV SQLX_OFFLINE true
ENV RUN_MODE production
RUN cd frontend/client && trunk build --release
RUN cargo build --release

FROM alpine:3.19 as runtime

ARG DATABASE_URL
ARG PORT=8080



RUN apk add --no-cache libgcc
RUN apk add --update openssl ca-certificates && \
    rm -rf /var/cache/apk/*


COPY --from=builder /app/target/release/api_server api_server
COPY config config
COPY --from=builder /app/frontend/client/dist assets

ENV DATABASE_URL $DATABASE_URL
ENV APP_APPLICATION__PORT $PORT
ENV RUN_MODE production
ENV APP_FRONTEND__ASSETS assets

ENTRYPOINT ["./api_server"]