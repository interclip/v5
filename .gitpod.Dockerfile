FROM gitpod/workspace-full

RUN cargo install diesel_cli --no-default-features --features postgres
