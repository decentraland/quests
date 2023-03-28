FROM gcr.io/distroless/cc-debian11
ARG PROJECT
COPY /target/release/${PROJECT} ./quests-binary
ENTRYPOINT [ "./quests-binary" ]
