FROM nwtgck/rust-musl-builder:1.64.0 AS builder
ADD --chown=rust:rust . ./
RUN cargo build --release

FROM alpine:latest
RUN apk --no-cache add ca-certificates
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/assessment \
    /usr/local/bin/
CMD /usr/local/bin/assessment
