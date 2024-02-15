FROM docker.io/lukemathwalker/cargo-chef:0.1.62-rust-1.74 AS chef

WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG PROJECT
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release -p $PROJECT 

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/${PROJECT} /usr/local/bin/quests-binary
ENTRYPOINT [ "quests-binary" ]