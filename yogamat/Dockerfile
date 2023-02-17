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
RUN cargo chef cook --profile wasm-release --recipe-path recipe.json
# build application
COPY . .
RUN cargo build --profile wasm-release --target wasm32-unknown-unknown
RUN wasm-bindgen --out-dir ./webapp/ --target web --no-typescript \
    ../target/wasm32-unknown-unknown/wasm-release/yoga_matt.wasm
WORKDIR app/webapp
RUN wasm-opt -Oz -o yoga_matt_bg.wasm yoga_matt_bg.wasm

FROM debian:buster-slim AS runtime
WORKDIR app
COPY --from=builder /app/target/release/s
COPY configuration configuration
ENV APP_ENVIRONMENT local
ENTRYPOINT ["./server"]
