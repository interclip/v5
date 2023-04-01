# Builder stage
FROM rust:1 as builder
WORKDIR /app
COPY . .
RUN cargo install --path .

# MySQL setup stage
FROM mysql:8.0 as mysql-setup
COPY scripts/db.sql /docker-entrypoint-initdb.d/db.sql
ENV MYSQL_ROOT_PASSWORD=""
ENV MYSQL_DATABASE="iclip"

# Runner stage
FROM ubuntu:22.04 as runner
RUN apt-get update && \
    apt-get install -y libssl1.1 mysql-client && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/interclip-server /usr/local/bin/interclip-server
ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000

# Start services
COPY scripts/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
CMD ["/entrypoint.sh"]