FROM rust:1 AS builder
WORKDIR app
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
ENTRYPOINT ["./server"]
