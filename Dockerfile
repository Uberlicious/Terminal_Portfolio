# --- Stage 1: Build the CSS ---
FROM node:22-slim AS css-builder
WORKDIR /app
COPY package.json pnpm-lock.yaml ./
# Install pnpm and dependencies
RUN corepack enable && corepack prepare pnpm@latest --activate && pnpm install --frozen-lockfile --ignore-scripts
# Copy Tailwind config and styles
COPY styles/ ./styles/
COPY templates/ ./templates/
# Ensure assets directory exists and generate the production CSS
RUN mkdir -p assets && pnpm build:css

# --- Stage 2: Build the Rust Backend ---
FROM rust:1.88-slim AS builder
WORKDIR /app
# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
# Copy the source code and templates
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY templates/ ./templates/
# Build for release
RUN cargo build --release

# --- Stage 3: Final Runtime Image ---
FROM debian:bookworm-slim
WORKDIR /app
# Install runtime dependencies (like SSL certificates if needed)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
# Copy the binary from the builder stage
COPY --from=builder /app/target/release/terminal-portfolio .
# Copy static assets and templates
COPY assets/ ./assets/
COPY templates/ ./templates/
# Copy the compiled CSS from the css-builder stage (overwrites local main.css if it exists)
COPY --from=css-builder /app/assets/main.css ./assets/main.css

# Expose the port the app runs on
EXPOSE 8000
# Run the application
CMD ["./terminal-portfolio"]
