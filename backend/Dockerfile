FROM rust:1 AS chef
RUN cargo install cargo-chef
WORKDIR app

FROM chef AS planner
COPY . .
# compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# build our project dependencies - this is the caching Docker layer
RUN cargo chef cook --release --recipe-path recipe.json
# build application
COPY . .
RUN cargo build --release --bin server

FROM debian:buster-slim AS runtime
WORKDIR app
COPY --from=builder /app/target/release/app /usr/local/bin
ENTRYPOINT ["/usr/local/bin/server"]
