FROM rust as builder 
ARG PROJECT
WORKDIR /app
COPY . .
RUN cargo build --release -p $PROJECT 

FROM gcr.io/distroless/cc-debian11 as runtime
ARG PROJECT
COPY --from=builder /app/target/release/${PROJECT} /usr/local/bin/quests-binary
# COPY --from=builder /app/configuration.toml /usr/local/bin
ENTRYPOINT [ "quests-binary" ]
