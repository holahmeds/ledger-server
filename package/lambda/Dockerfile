FROM rust as builder

ADD . ./
RUN cargo build --release --bin ledger-lambda --no-default-features --features="lambda"

FROM debian:buster-slim
RUN apt-get update \
    && apt-get install -y ca-certificates libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /target/release/ledger-lambda /bin

CMD ["/bin/ledger-lambda"]
