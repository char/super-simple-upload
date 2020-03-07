FROM rust:1.41 as builder

WORKDIR /usr/src/super-simple-upload
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/super-simple-upload /usr/local/bin/super-simple-upload

CMD ["super-simple-upload"]
