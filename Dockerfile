# syntax=docker/dockerfile:1.4
FROM rust as builder 
ARG PROJECT
WORKDIR /app

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install cargo-strip

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/app/target \
    cargo build --release -p $PROJECT && \
    cargo strip && \
    mv /app/target/release/${PROJECT} /app

FROM gcr.io/distroless/cc-debian11 as runtime
ARG PROJECT
COPY --from=builder /app/${PROJECT} /usr/local/bin/quests-binary
# COPY --from=builder /app/configuration.toml /usr/local/bin
ENTRYPOINT [ "quests-binary" ]
