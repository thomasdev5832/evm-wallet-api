# Estágio 1: Compilação
FROM rust:1.88 as builder

# Instala dependências do sistema (necessárias para ethers-rs, openssl, etc.)
RUN apt-get update && apt-get install -y pkg-config libssl-dev

WORKDIR /app
COPY . .

# Cache de dependências (truque para builds mais rápidos)
RUN cargo fetch
RUN cargo build --release

# Estágio 2: Imagem final leve
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y openssl ca-certificates

WORKDIR /app
COPY --from=builder /app/target/release/evm-wallet-api .
COPY --from=builder /app/.env .  

# Porta que sua API usa
EXPOSE 3000 
CMD ["./evm-wallet-api"]