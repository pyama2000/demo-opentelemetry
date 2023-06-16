FROM rust:1.70.0 AS builder
WORKDIR /app
COPY . /app
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/item_service /
ENTRYPOINT ["./item_service"]
