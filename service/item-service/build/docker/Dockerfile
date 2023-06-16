FROM rust:1.70.0 AS builder
WORKDIR /app
COPY . /app
RUN cargo build --release

FROM busybox:1.36.1-uclibc AS binary

FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/item_service /
COPY --from=binary /bin/wget /usr/bin/wget
ENTRYPOINT ["./item_service"]
