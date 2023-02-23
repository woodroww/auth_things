FROM rust:latest
RUN apt-get update
RUN apt-get install -y iputils-ping
RUN apt-get install -y vim
RUN cargo install --locked trunk
RUN rustup target add wasm32-unknown-unknown
WORKDIR frontend
COPY . .
RUN trunk build index.html
#COPY ./dist 
#ENTRYPOINT ["tail", "-f", "/dev/null"]
