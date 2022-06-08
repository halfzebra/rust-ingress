use axum::{extract::Json, response::Html, routing::get, Router};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::get_rdkafka_version;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use validator::{Validate, ValidationError};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "example_form=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dbg!(get_rdkafka_version());

    let app = Router::new().route(
        "/",
        get(show_index).post(|Json(input): Json<Input>| async move {
            dbg!(&input);

            let producer: &FutureProducer = &ClientConfig::new()
                .set("bootstrap.servers", "localhost:29092")
                .set("message.timeout.ms", "5000")
                .create()
                .expect("Producer creation error");

            dbg!(&serde_json::to_string(&input).unwrap());

            let delivery_status = producer
                .send(
                    FutureRecord::to("test")
                        .payload(&serde_json::to_string(&input).unwrap())
                        .key(&format!("Key {}", 1)),
                    Duration::from_secs(0),
                )
                .await;
            dbg!(delivery_status);
        }),
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::debug!("listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn show_index() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                Hello
            </body>
        </html>
        "#,
    )
}

#[derive(Deserialize, Validate, Serialize, Debug)]
#[allow(dead_code)]
#[serde(deny_unknown_fields)]
struct Input {
    ts: String,
    sender: String,
    message: serde_json::Value,
    #[serde(rename(serialize = "sent-from-ip", deserialize = "sent-from-ip"))]
    #[validate(custom = "validate_ip")]
    ip: String,
    priority: u8,
}

fn validate_ip(ip: &str) -> Result<(), ValidationError> {
    if !validator::validate_ip(ip) {
        return Err(ValidationError::new("invalid IP address"));
    }

    Ok(())
}
