# --- yew

FROM rust:latest as yew
RUN apt-get update
RUN apt-get install -y iputils-ping
RUN apt-get install -y vim
RUN cargo install --locked trunk
RUN rustup target add wasm32-unknown-unknown
COPY frontend frontend
WORKDIR frontend
RUN trunk build index.html

# --- backend

# https://github.com/LukeMathWalker/cargo-chef
FROM rust:latest AS chef
RUN cargo install cargo-chef
COPY backend backend

FROM chef AS planner
WORKDIR backend
# compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner backend/recipe.json recipe.json
# build our project dependencies - this is the caching Docker layer
RUN cargo chef cook --release --recipe-path recipe.json
# build application
# COPY . .
RUN cargo build --release --bin server

FROM debian:buster-slim AS runtime
WORKDIR app
RUN apt-get update -y \
	&& apt-get install -y --no-install-recommends openssl ca-certificates \
	&& apt-get autoremove -y \
	&& apt-get clean -y \
	&& rm -rf /var/lib/apt/lists/*
COPY --from=builder /backend/target/release/server server
COPY --from=yew /frontend/dist dist
COPY backend/configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./server"]
