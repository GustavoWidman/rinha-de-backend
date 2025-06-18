# Multi-stage build para otimização
FROM rust:1.87-slim as builder

# Instalar dependências do sistema
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copiar manifesto e dependências primeiro (para cache de build)
COPY Cargo.toml Cargo.lock ./
COPY load-test ./load-test
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm src/main.rs

# Copiar código fonte
COPY src ./src
COPY script.sql ./script.sql

# Build da aplicação (aproveitando cache das dependências)
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Instalar dependências de runtime
RUN apt-get update && apt-get install -y \
    ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copiar binário da aplicação
COPY --from=builder /app/target/release/rinha-de-backend ./

# Usuário não-root para segurança
RUN useradd -r -s /bin/false appuser

# Criar diretório compartilhado e configurar permissões APÓS criar o usuário
RUN mkdir -p /shared && chmod 777 /shared && chown appuser:appuser /shared

USER appuser

EXPOSE 8080

CMD ["./rinha-de-backend"]
