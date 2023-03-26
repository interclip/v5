FROM rust:1 as builder
WORKDIR /app
COPY . .
RUN cargo install --path .

FROM ubuntu:22.04 as runner
RUN apt-get update && \
    apt-get install -y libssl1.1 && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/interclip-server /usr/local/bin/interclip-server
ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000
CMD ["interclip-server"]
