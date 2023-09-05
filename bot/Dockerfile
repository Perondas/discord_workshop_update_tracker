FROM rust:bullseye AS chef

RUN cargo install cargo-chef
WORKDIR /bot

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /bot/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin bot

FROM debian:bullseye AS runtime

WORKDIR /bot

COPY --from=builder /bot/target/release/bot /usr/local/bin
ENTRYPOINT ["/usr/local/bin/bot"]