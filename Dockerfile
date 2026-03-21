# =============================================================================
# Stage 1 — Builder
# Compiles the release binary inside the official Rust Alpine image.
# This stage is discarded entirely after compilation; nothing from it
# (compiler, registry cache, source code) ends up in the final image.
# =============================================================================
FROM rust:alpine AS builder

WORKDIR /app

# musl-dev   — C linker required by some crates (reqwest, ring, etc.)
# pkgconfig  — helps crates locate system libraries at build time
RUN apk add --no-cache musl-dev pkgconfig

# Copy manifests first so Docker can cache the dependency-compile layer.
# If only src/ changes, this layer is reused and `cargo build` only
# recompiles your own code — not the entire dependency tree.
COPY Cargo.toml Cargo.lock ./

# Compile a dummy main to cache all dependencies as a separate layer.
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Now copy real source and do the final build.
# `touch` forces Cargo to recognise the source as newer than the cached stub.
COPY src ./src
RUN touch src/main.rs && cargo build --release

# =============================================================================
# Stage 2 — Runtime
# Minimal Alpine image — just enough to run the binary.
# Final image is typically 10–20 MB vs ~1 GB for the builder.
# =============================================================================
FROM alpine:3.20 AS runtime

# ca-certificates — required for HTTPS connections to the AniList API
# libgcc          — runtime support lib some musl builds still reference
RUN apk add --no-cache ca-certificates libgcc

# Run as a non-root user (Docker best practice)
RUN adduser --disabled-password --gecos "" --no-create-home botuser
USER botuser

# Copy only the compiled binary from the builder stage
COPY --from=builder /app/target/release/anilist /usr/local/bin/anilist

ENTRYPOINT ["/usr/local/bin/anilist"]