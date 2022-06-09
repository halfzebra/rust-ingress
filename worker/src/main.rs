use rdkafka::config::ClientConfig;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::message::Message;
use tokio_postgres::NoTls;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "ingress=trace,tower_http=trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::debug!("Worker started");

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
            tracing::error!("connection error: {}", e);
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
            Err(e) => tracing::error!("Kafka error: {}", e),
            Ok(m) => {
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        tracing::error!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };

                let v: serde_json::Value = serde_json::from_str(payload).unwrap();

                let res = client
                    .query("INSERT INTO messages (payload) VALUES ($1)", &[&v])
                    .await;

                tracing::debug!("{:?}", res);

                match consumer.commit_message(&m, CommitMode::Async) {
                    Ok(_) => tracing::debug!("Mesagge committed"),
                    Err(e) => tracing::error!("Failed to commit {}", e),
                };
            }
        };
    }
}
