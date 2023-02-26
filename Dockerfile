FROM rust:1.67.1-alpine3.17 as chef

RUN apk add --no-cache alpine-sdk
RUN cargo install cargo-chef
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
RUN cargo build --release

FROM alpine:3.17.2 as runtime

ARG DATABASE_URL
ARG PORT=8080



RUN apk add --no-cache libgcc
RUN apk add --update openssl ca-certificates && \
    rm -rf /var/cache/apk/*


COPY --from=builder /app/target/release/api_server api_server
COPY config config

ENV DATABASE_URL $DATABASE_URL
ENV APP_APPLICATION_PORT $PORT
ENV RUN_MODE production

ENTRYPOINT ["./api_server"]