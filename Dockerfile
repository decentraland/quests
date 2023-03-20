FROM gcr.io/distroless/cc-debian11 as runtime
ARG PROJECT
COPY target/release/${PROJECT} /usr/local/bin/quests-binary
ENTRYPOINT [ "quests-binary" ]
