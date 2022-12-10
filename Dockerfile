FROM alpine:3.17.0
RUN apk add git curl pkgconfig openssl-dev gcc musl-dev rustup alsa-lib-dev freetype-dev fontconfig-dev
RUN rustup-init -t x86_64-unknown-linux-musl --default-toolchain nightly --profile minimal -y

WORKDIR /usr/src/app

COPY . .
RUN /root/.cargo/bin/cargo build --all-features --release

## Need to build docker for musl
