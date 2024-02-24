FROM rust:1.68 as builder
RUN apt-get update \
    && apt-get install -y protobuf-compiler

ADD . ./
RUN cargo build --release

FROM ubuntu:24.04
RUN apt-get update \
    && apt-get install -y ca-certificates libpq5 curl \
    && rm -rf /var/lib/apt/lists/*

ENV CONFIGURATION_DIRECTORY=/config

EXPOSE 8000

HEALTHCHECK CMD curl --fail http://localhost:8000/health || exit 1

COPY --from=builder /target/release/ledger-server /bin

ENTRYPOINT ["/bin/ledger-server"]
