# ============================================================
# Stage 1: Build Rust backend
# ============================================================
FROM rust:1.88.0-slim-bookworm AS backend-builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
ENV CARGO_BUILD_JOBS=2

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./

# Stub out source so cargo can cache deps
RUN mkdir -p src/bin && echo "fn main(){}" > src/main.rs && echo "fn main(){}" > src/bin/migrate.rs
RUN cargo build --release || true
RUN rm -rf src

# Copy real source and rebuild (only recompiles our code, deps cached)
COPY src/ src/
COPY migrations/ migrations/
RUN cargo build --release

# ============================================================
# Stage 2: Build React+Vite frontend
# ============================================================
FROM node:20-slim AS frontend-builder

WORKDIR /app
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm install

COPY frontend/index.html frontend/vite.config.ts frontend/tsconfig.json frontend/tsconfig.app.json frontend/tsconfig.node.json ./
COPY frontend/src/ src/

RUN npm run build

# ============================================================
# Stage 3: Runtime
# ============================================================
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=backend-builder /app/target/release/ab-backend /usr/local/bin/ab-backend
COPY --from=backend-builder /app/target/release/migrate /usr/local/bin/migrate
COPY --from=frontend-builder /app/dist /app/frontend
COPY migrations/ /app/migrations/

WORKDIR /app
ENV RUST_LOG=info
ENV PORT=8080
ENV FRONTEND_DIR=/app/frontend
EXPOSE 8080

CMD migrate && ab-backend
