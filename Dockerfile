FROM rust:1-alpine as builder

WORKDIR /app

COPY . .

RUN apk add --no-cache musl-dev
RUN DATABASE_URL=sqlite://archk.db cargo build --release --verbose

FROM alpine:latest

COPY --from=builder /app/target/release/archk-api /usr/local/bin/archk-api

EXPOSE 8000
ENV RUST_LOG=archk_api=info,tower_http=info
CMD ["archk-api"]