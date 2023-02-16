# https://github.com/LukeMathWalker/cargo-chef
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
RUN apt-get update -y \
	&& apt-get install -y --no-install-recommends openssl ca-certificates \
	&& apt-get autoremove -y \
	&& apt-get clean -y \
	&& rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/server server
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["server"]
