# https://github.com/LukeMathWalker/cargo-chef
FROM rust:1 AS chef
RUN cargo install cargo-chef
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-opt --locked
RUN cargo install -f wasm-bindgen-cli
# bevy sound need these
RUN apt-get update -y \
	&& apt-get install -y --no-install-recommends libasound2-dev libudev-dev \
	&& apt-get autoremove -y \
	&& apt-get clean -y \
	&& rm -rf /var/lib/apt/lists/*
WORKDIR app

FROM chef AS planner
COPY . .
# compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# build our project dependencies - this is the caching Docker layer
RUN cargo chef cook --profile wasm-release --target wasm32-unknown-unknown --recipe-path recipe.json
# build application
COPY . .
RUN cargo build --profile wasm-release --target wasm32-unknown-unknown
RUN wasm-bindgen --out-dir ./webapp/ --target web --no-typescript \
    ./target/wasm32-unknown-unknown/wasm-release/yogamat.wasm
# WORKDIR app/webapp
RUN wasm-opt -Oz -o yogamat_bg.wasm ./webapp/yogamat_bg.wasm

FROM debian:buster-slim AS runtime
WORKDIR app
COPY --from=builder /app/webapp/yogamat_bg.wasm yogamat_bg.wasm
# COPY configuration configuration
ENTRYPOINT ["tail", "-f", "/dev/null"]
