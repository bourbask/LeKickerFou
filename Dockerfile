# Build stage
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static

# Set working directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy real source code
COPY src ./src

# Build the actual binary
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache ca-certificates tzdata

# Create app user
RUN addgroup -g 1000 app && \
    adduser -D -s /bin/sh -u 1000 -G app app

# Set working directory
WORKDIR /app

# Copy binary from build stage
COPY --from=builder /app/target/release/lekickerfou /app/lekickerfou

# Change ownership
RUN chown -R app:app /app

# Switch to app user
USER app

# Expose port (if needed)
# EXPOSE 8080

# Set entrypoint
ENTRYPOINT ["./lekickerfou"]
