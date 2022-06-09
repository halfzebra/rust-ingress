use rdkafka::config::ClientConfig;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::message::Message;
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() {
    println!("Worker started");

    let (client, connection) = tokio_postgres::connect(
        &format!(
            "host={} user=postgres password=postgres",
            std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".into())
        ),
        NoTls,
    )
    .await
    .unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
            std::process::exit(1);
        }
    });

    let consumer: StreamConsumer = ClientConfig::new()
        .set(
            "bootstrap.servers",
            format!(
                "{}:29092",
                std::env::var("KAFKA_HOST").unwrap_or_else(|_| "localhost".into())
            ),
        )
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .set("group.id", "rust-rdkafka-roundtrip-example")
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&vec!["test"])
        .expect("Can't subscribe to specified topics");

    loop {
        match consumer.recv().await {
            Err(e) => {
                dbg!("Kafka error: {}", e);
            }
            Ok(m) => {
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        dbg!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };
                // dbg!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                //           m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());

                let v: serde_json::Value = serde_json::from_str(payload).unwrap();

                let res = client
                    .query("INSERT INTO messages (payload) VALUES ($1)", &[&v])
                    .await;

                // dbg!(rows);

                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        };
    }
}
