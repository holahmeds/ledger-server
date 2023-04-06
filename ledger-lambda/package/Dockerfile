FROM rust:1.68 as builder
RUN apt-get update \
    && apt-get install -y protobuf-compiler

ADD . ./
RUN cargo build --release --package ledger-lambda

FROM ubuntu:22.10
RUN apt-get update \
    && apt-get install -y ca-certificates libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /target/release/ledger-lambda /bin

CMD ["/bin/ledger-lambda"]
