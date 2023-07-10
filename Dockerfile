FROM rust:latest as builder
WORKDIR /opt
COPY src ./src
COPY Cargo.lock .
COPY Cargo.toml .
RUN cargo build --release

FROM rust:latest
WORKDIR /opt
COPY --from=builder /opt/target/release/kuretiki-chan .
CMD ["./kuretiki-chan"]