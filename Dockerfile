FROM rust:latest as builder
RUN apt-get update
RUN apt-get install -y iputils-ping
RUN apt-get install -y vim
RUN cargo install --locked trunk
RUN rustup target add wasm32-unknown-unknown
ENV APP_ENVIRONMENT production
COPY frontend frontend
WORKDIR frontend
RUN trunk build index.html
WORKDIR /
COPY common common
COPY backend backend
WORKDIR backend
RUN cargo build --release --bin server

FROM debian:buster-slim AS runtime
WORKDIR /
RUN apt-get update -y \
	&& apt-get install -y --no-install-recommends openssl ca-certificates \
	&& apt-get autoremove -y \
	&& apt-get clean -y \
	&& rm -rf /var/lib/apt/lists/*
COPY --from=builder backend/target/release/server ./app/server
COPY --from=builder frontend/dist ./app/dist
COPY configuration configuration
ENV APP_ENVIRONMENT production
WORKDIR app
ENTRYPOINT ["./server"]
#ENTRYPOINT ["tail", "-f", "/dev/null"]
