FROM alpine:3.17.0

# vosk-api package is currently in testing
RUN echo "http://dl-cdn.alpinelinux.org/alpine/edge/testing/" >> /etc/apk/repositories
RUN apk add git curl pkgconfig openssl-dev gcc musl-dev rustup alsa-lib-dev freetype-dev fontconfig-dev vosk-api

RUN rustup-init -t x86_64-unknown-linux-musl --default-toolchain nightly --profile minimal -y

WORKDIR /usr/src/app

COPY . .

RUN /root/.cargo/bin/cargo build --all-features --release
