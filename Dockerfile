FROM rust as chef
RUN cargo install cargo-chef

WORKDIR /app

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder 
ARG PROJECT
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release -p $PROJECT 

FROM gcr.io/distroless/cc-debian11 as runtime
ARG PROJECT
COPY --from=builder /app/target/release/${PROJECT} /usr/local/bin/quests-binary
# COPY --from=builder /app/configuration.toml /usr/local/bin
ENTRYPOINT [ "quests-binary" ]
