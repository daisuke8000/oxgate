# Build stage
FROM rust:1.92-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev postgresql-dev

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy main to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build application
RUN touch src/main.rs && \
    cargo build --release

# Runtime stage
FROM alpine:3.21

# Install runtime dependencies
RUN apk add --no-cache libgcc postgresql-libs ca-certificates

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/oxgate /app/oxgate

# Copy migrations
COPY --from=builder /app/migrations /app/migrations

# Create non-root user
RUN addgroup -g 1000 oxgate && \
    adduser -D -u 1000 -G oxgate oxgate && \
    chown -R oxgate:oxgate /app

USER oxgate

EXPOSE 8080

CMD ["/app/oxgate"]
