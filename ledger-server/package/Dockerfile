FROM rust:1.68 as builder
RUN apt-get update \
    && apt-get install -y protobuf-compiler

ADD . ./
RUN cargo build --release

FROM ubuntu:22.10
RUN apt-get update \
    && apt-get install -y ca-certificates libpq5 \
    && rm -rf /var/lib/apt/lists/*

ENV CONFIGURATION_DIRECTORY=/config

EXPOSE 8000

COPY --from=builder /target/release/ledger-server /bin

ENTRYPOINT ["/bin/ledger-server"]
