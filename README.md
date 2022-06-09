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

Here's resourse usage for a run sending 20k requests over 60 seconds on WSL2(probably the most inefficient):

```
CONTAINER ID   NAME                          CPU %     MEM USAGE / LIMIT     MEM %     NET I/O           BLOCK I/O   PIDS
08e0e50ea4b9   rust-axum-kafka-ingress-1     0.12%     101.8MiB / 12.44GiB   0.80%     21MB / 16.6MB     0B / 0B     17
d3af0d3e5b91   rust-axum-kafka-worker-1      1.68%     15.89MiB / 12.44GiB   0.12%     12.8MB / 11.1MB   0B / 0B     17
1448a81750a8   rust-axum-kafka-kafka-1       1.61%     424.3MiB / 12.44GiB   3.33%     10.4MB / 8.59MB   0B / 0B     86
3c6f10adab04   rust-axum-kafka-db-1          0.88%     24.11MiB / 12.44GiB   0.19%     7.44MB / 6.32MB   0B / 0B     8
46f32ebaa1be   rust-axum-kafka-zookeeper-1   0.07%     94.45MiB / 12.44GiB   0.74%     81.9kB / 65.2kB   0B / 0B     70
```

## Why Apache Kafka?

High performance(proven 100k RPS) and fault tolerance.
Note: Fault tolerance is _not really_ explored in this project as it is not even using a Kafka cluster.


## Why Rust?

I stumbled upon [fede1024/rust-rdkafka](https://github.com/fede1024/rust-rdkafka) which promised a pretty good [performance.](https://github.com/fede1024/kafka-benchmark)

The [ingres](./ingress/) service is using a relatively new [tokio-rs/axum](https://github.com/tokio-rs/axum) framework, which is [speculated](https://github.com/piaoger/webframework-bench) to have better performance than Go and offers slightly better ergonomics over the previous HTTP Server frameworks.

## Why Dockerfile is so complicated?

It contains a build optimization from [Packaging a Rust web service using Docker.](https://blog.logrocket.com/packaging-a-rust-web-service-using-docker/)

## TODO:

- the [worker](./worker/) needs to batch inserts as it is getting pretty slow
- run Kafka as a cluster
- the [ingress](./ingress/) is okay CPU-wise, but suspiciously high memory usage(needs investigation)
- add processing result output in the worker
- run load tests on a better hardware on Linux
- organize code better, add tests