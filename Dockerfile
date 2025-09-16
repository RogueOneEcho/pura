# Build binary
FROM rust:alpine AS builder
RUN apk add --no-cache cargo-edit libc-dev
# Build just the dependencies with version 0.0.0 so they're cached
WORKDIR /app
COPY Cargo.toml Cargo.lock /app/
RUN mkdir -p src && echo 'fn main() {}' > /app/src/main.rs
RUN cargo fetch
RUN cargo build --release --locked
COPY src /app/src
# Set the version
ARG VERSION=0.0.0
RUN cargo set-version $VERSION
# Build the release binary
RUN cargo build --release

# Build final image with minimal dependencies
FROM alpine:latest
EXPOSE 2632/tcp
COPY --from=builder /app/target/release/pura /bin/pura
WORKDIR /
ENTRYPOINT ["pura"]
