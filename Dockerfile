FROM rust:slim as build

RUN apt-get update && \
    apt-get install -y \
        git \
        openssh-server \
        openssh-client

ENV META_TOKEN=""
ENV REDIS_URL=""
ARG SSH_KEY

WORKDIR /app
COPY . .
RUN --mount=type=ssh cargo build --release


FROM debian:11-slim
WORKDIR /app
RUN apt-get update && \
    apt-get install -y ca-certificates curl
COPY --from=build /app/target/release/vin-webhook ./vin-webhook
COPY --from=build /app/download-image.sh ./download-image.sh
CMD ["./vin-webhook"]