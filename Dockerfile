FROM rust

ENV META_TOKEN=""
ENV REDIS_URL=""

WORKDIR /app
COPY . .
RUN cargo build --release
ENTRYPOINT ["./target/release/vin-webhook"]