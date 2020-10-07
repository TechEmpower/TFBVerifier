FROM rust:latest

COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml

# This is to download, compile, and cache dependencies before adding real src
RUN mkdir src
RUN touch src/lib.rs
RUN cargo build --release
RUN rm src/lib.rs

COPY src src

RUN cargo build --release

RUN apt-get update && apt-get install -yqq libluajit-5.1-dev libssl-dev luajit

WORKDIR /wrk
RUN curl -sL https://github.com/wg/wrk/archive/4.1.0.tar.gz | tar xz --strip-components=1
ENV LDFLAGS="-O3 -march=native -flto"
ENV CFLAGS="-I /usr/include/luajit-2.1 $LDFLAGS"
RUN make WITH_LUAJIT=/usr WITH_OPENSSL=/usr -j "$(nproc)"
RUN cp wrk /usr/local/bin

WORKDIR /
# Required scripts for benchmarking
COPY pipeline.lua pipeline.lua

CMD target/release/tfb_verifier
