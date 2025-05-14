FROM rust:1.86 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM ubuntu:rolling
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/rust-web-template /usr/local/bin/rust-web-template
CMD ["rust-web-template"]
