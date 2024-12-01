FROM rust:1.82.0-bullseye

WORKDIR /app/iwasm
COPY . .
ENV CARGO_HOME=/root/.cargo
RUN --mount=type=cache,target="/root/.cargo" \
    --mount=type=cache,target="/app/iwasm/target" \
    make build
