FROM alpine:3.14

RUN mkdir /app
WORKDIR /app
COPY ./target/x86_64-unknown-linux-musl/release/ngamahi-id-gen /app/server
COPY ./configs /app/configs

EXPOSE 8080


CMD ["/app/server"]

# before build:
# apt-get install musl-tools
# rustup target add x86_64-unknown-linux-musl
# cargo build --target x86_64-unknown-linux-musl --release
