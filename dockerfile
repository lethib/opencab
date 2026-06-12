# ------------------------------------------------------------------------------
# STAGE 1: Frontend Dependencies Cache Layer
# ------------------------------------------------------------------------------
# Using alpine variant for smaller base image and faster download
# This stage is optimized for maximum cache reuse of node_modules
FROM oven/bun:1-alpine AS frontend-deps
LABEL stage=frontend-deps
WORKDIR /app

# Install frontend dependencies with optimal caching strategy
# Copy only lock files first to maximize Docker layer cache efficiency
COPY frontend/package.json frontend/bun.lock ./

# Install with frozen lockfile for reproducible builds
# Use --no-cache to avoid storing package cache in layer
RUN bun install --frozen-lockfile --no-cache && \
    # Remove any unnecessary files to reduce layer size
    rm -rf /tmp/* /var/tmp/* && \
    # Verify critical dependencies are installed
    [ -d "node_modules" ] || (echo "Frontend dependencies installation failed" && exit 1)

# ------------------------------------------------------------------------------
# STAGE 2: Frontend Build Stage
# ------------------------------------------------------------------------------
# Separate stage for frontend build to enable parallel execution with backend
FROM oven/bun:1-alpine AS frontend-builder
LABEL stage=frontend-builder
WORKDIR /app

# Copy dependencies from cache layer
COPY --from=frontend-deps /app/node_modules ./node_modules
COPY --from=frontend-deps /app/package.json ./package.json

# Copy frontend source code (separate from deps for better caching)
COPY frontend/ ./

# Build frontend with production optimizations
RUN bun run build && \
    # Verify build output exists
    [ -d "dist" ] || (echo "Frontend build failed - dist directory not found" && exit 1) && \
    # Clean up build artifacts and temp files to reduce layer size
    rm -rf node_modules/.cache /tmp/* /var/tmp/* && \
    # Show build output size for monitoring
    du -sh dist/

# ------------------------------------------------------------------------------
# STAGE 3: Rust Dependencies Chef Setup
# ------------------------------------------------------------------------------
# Using specific Rust version with slim variant for faster downloads
FROM lukemathwalker/cargo-chef:latest-rust-1.88.0-slim AS chef
LABEL stage=chef
WORKDIR /app

# Install additional system dependencies if needed
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev && \
    # Clean up apt cache to reduce layer size
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# ------------------------------------------------------------------------------
# STAGE 4: Rust Build Planning
# ------------------------------------------------------------------------------
# Generate dependency recipe for maximum cache efficiency
FROM chef AS planner
LABEL stage=planner

# Copy only Cargo files for dependency analysis
COPY Cargo.toml Cargo.lock ./
COPY migration/ migration/
COPY src/ src/

# Generate recipe.json with all dependency information
RUN cargo chef prepare --recipe-path recipe.json

# ------------------------------------------------------------------------------
# STAGE 5: Rust Dependencies Build
# ------------------------------------------------------------------------------
# Build only Rust dependencies in separate layer for optimal caching
FROM chef AS rust-deps
LABEL stage=rust-deps

# Copy recipe and Cargo.toml files (needed by cargo chef cook)
COPY --from=planner /app/recipe.json recipe.json
COPY Cargo.toml Cargo.lock ./
COPY migration/ migration/

# Create stub test file required by Cargo.toml manifest validation
RUN mkdir -p tests && touch tests/bdd.rs

# Build dependencies with release optimizations and specific target
# This layer will be cached unless dependencies change
RUN cargo chef cook --release --target x86_64-unknown-linux-gnu --recipe-path recipe.json && \
    # Clean up cargo cache and build artifacts to reduce layer size
    rm -rf ~/.cargo/registry/cache/* ~/.cargo/git/checkouts/* /tmp/* /var/tmp/*

# ------------------------------------------------------------------------------
# STAGE 6: Rust Application Build
# ------------------------------------------------------------------------------
# Build the actual application code
FROM chef AS rust-builder
LABEL stage=rust-builder

# Copy pre-built dependencies
COPY --from=rust-deps /app/target target

# Copy application source code
COPY Cargo.toml Cargo.lock ./
COPY migration/ migration/
COPY src/ src/
COPY frontend/public/favicon/apple-touch-icon.png frontend/public/favicon/apple-touch-icon.png
RUN mkdir -p tests && touch tests/bdd.rs

# Build application with optimizations
# Using explicit target for consistent builds across architectures
RUN cargo build --release --target x86_64-unknown-linux-gnu && \
    # Build migrate binary
    cargo build --release --bin migrate --target x86_64-unknown-linux-gnu && \
    # Build send_access_token binary
    cargo build --release --bin send_access_token --target x86_64-unknown-linux-gnu && \
    # Strip binaries to reduce size (remove debug symbols)
    strip target/x86_64-unknown-linux-gnu/release/opencab && \
    strip target/x86_64-unknown-linux-gnu/release/migrate && \
    strip target/x86_64-unknown-linux-gnu/release/send_access_token && \
    # Verify binaries were built successfully
    [ -f "target/x86_64-unknown-linux-gnu/release/opencab" ] || \
        (echo "Rust build failed - binary not found" && exit 1) && \
    [ -f "target/x86_64-unknown-linux-gnu/release/migrate" ] || \
        (echo "Migrate binary build failed - binary not found" && exit 1) && \
    [ -f "target/x86_64-unknown-linux-gnu/release/send_access_token" ] || \
        (echo "Send access token binary build failed - binary not found" && exit 1)

# ------------------------------------------------------------------------------
# STAGE 7: Final Runtime Image (Distroless for Security & Size)
# ------------------------------------------------------------------------------
# Using distroless cc-debian12 for minimal attack surface and small size
# Includes glibc and other C libraries needed for Rust binaries
FROM gcr.io/distroless/cc-debian12:nonroot AS runtime
LABEL stage=runtime
LABEL maintainer="DevOps Team"
LABEL version="1.0"
LABEL description="OpenCab Application - Optimized Production Build"

# Set working directory
WORKDIR /app

# Create non-root user for security (distroless includes 'nonroot' user)
# The 'nonroot' user has UID 65532 and GID 65532
COPY --from=frontend-builder --chown=65532:65532 /app/dist ./frontend/dist/
COPY --from=rust-builder --chown=65532:65532 \
    /app/target/x86_64-unknown-linux-gnu/release/opencab ./opencab
COPY --from=rust-builder --chown=65532:65532 \
    /app/target/x86_64-unknown-linux-gnu/release/migrate ./migrate
COPY --from=rust-builder --chown=65532:65532 \
    /app/target/x86_64-unknown-linux-gnu/release/send_access_token ./send_access_token
COPY --chown=65532:65532 config/ ./config/

# Set optimal defaults for production
ENV RUST_LOG=info
ENV RUST_BACKTRACE=0

EXPOSE 5150
ENTRYPOINT ["/app/opencab"]
