# 🚶🚶🚶 rust-ingress

This is a POC for ingress solution using Apache Kafka with two services for [ingressing](./ingress/) ang [storing](./worker/) messages into Postgres.
The code is a happy-path spagetti to test whether this is even going to work.

## Running

```bash
docker-compose up
```

## Why Apache Kafka?

- High performance with proven 100k RPS
- Fault tolerance

## Why Rust?

I stumbled upon [fede1024/rust-rdkafka](https://github.com/fede1024/rust-rdkafka) which promised a pretty good [performance.](https://github.com/fede1024/kafka-benchmark)

The [ingres](./ingress/) service is using relatively new [tokio-rs/axum](https://github.com/tokio-rs/axum) framework, which is [speculated](https://github.com/piaoger/webframework-bench) to have better performance than Go and offers slightly better ergonomics over the previous HTTP Server frameworks.

## Why Dockerfile is so complicated?

It contains a build optimization from [Packaging a Rust web service using Docker](https://blog.logrocket.com/packaging-a-rust-web-service-using-docker/)