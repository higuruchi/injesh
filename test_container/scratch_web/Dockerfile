FROM rust:alpine3.15 AS builder

WORKDIR /scratch_web

COPY Cargo.toml Cargo.toml
COPY src/ src/
RUN RUSTFLAGS='-C target-feature=+crt-static' cargo build

# ------------------------------

FROM scratch

COPY --from=builder /scratch_web/target/debug/scratch_web /

ENTRYPOINT ["/scratch_web"]
