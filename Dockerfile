# Multi-stage build for Observation Tools Server
# Stage 1: Build the Rust application
FROM rust:1.83-bookworm AS builder

WORKDIR /build

# Copy the entire workspace (needed for workspace dependencies)
COPY . .

# Build the server binary in release mode
RUN cargo build --release --bin observation-tools

# Stage 2: Create the runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /build/target/release/observation-tools /app/observation-tools

# Copy static files
COPY --from=builder /build/crates/observation-tools-server/static /app/static

# Create a directory for data storage
RUN mkdir -p /data

# Set environment variables with defaults
ENV HOST=0.0.0.0
ENV PORT=3000
ENV DATA_DIR=/data
ENV STATIC_DIR=/app/static
ENV RUST_LOG=info

# Expose the default port
EXPOSE 3000

# Run the server
CMD ["/app/observation-tools"]
