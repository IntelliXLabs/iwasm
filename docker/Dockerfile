FROM rust:1.82.0-bullseye AS build

WORKDIR /app/iwasm
COPY . .
ENV CARGO_HOME=/root/.cargo
RUN --mount=type=cache,target="/root/.cargo" \
    --mount=type=cache,target="/app/iwasm/target" \
    make build

# runtime slim image debian
FROM debian:bullseye-slim AS runtime
COPY --from=build /app/iwasm/lib /app/iwasm/lib