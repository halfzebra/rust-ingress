# 🚶🚶🚶 rust-ingress

This is a POC for ingress solution using Apache Kafka with two services for [ingressing](./ingress/) and [storing](./worker/) messages into Postgres.
The code is a happy-path spaghetti to test whether this is even going to work.

![Blank diagram](https://user-images.githubusercontent.com/3983879/172737556-8266fab6-f2b5-4181-993f-c924ea832c9f.png)

## Running

Prerequisites: _Docker_

```bash
docker-compose up
```

## Testing

For a simple test:

```bash
curl --location --request POST 'http://0.0.0.0:8000/api/v1/message' \
    --header 'Content-Type: application/json' \
    --data-raw '{
        "ts": "1530228282",
        "sender": "testy-test-service",
        "message": {
            "foo": "bar",
            "baz": "bang"
        },
        "sent-from-ip": "1.2.3.4",
        "priority": 2
    }'
```

Prerequisites: _Node.js_

```bash
# install load testing tooling
npm install -s artillery@latest

# start blasting
artillery run load-test.yaml
```

## Why Apache Kafka?

High performance(proven 100k RPS) and fault tolerance.
Note: Fault tolerance is _not really_ explored in this project as it is not even using a Kafka cluster.


## Why Rust?

I stumbled upon [fede1024/rust-rdkafka](https://github.com/fede1024/rust-rdkafka) which promised a pretty good [performance.](https://github.com/fede1024/kafka-benchmark)

The [ingres](./ingress/) service is using a relatively new [tokio-rs/axum](https://github.com/tokio-rs/axum) framework, which is [speculated](https://github.com/piaoger/webframework-bench) to have better performance than Go and offers slightly better ergonomics over the previous HTTP Server frameworks.

## Why Dockerfile is so complicated?

It contains a build optimization from [Packaging a Rust web service using Docker.](https://blog.logrocket.com/packaging-a-rust-web-service-using-docker/)