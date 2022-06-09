use axum::{
    extract::Json,
    handler::Handler,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    routing::post,
    Router,
};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use validator::{Validate, ValidationError};
#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "ingress=trace,tower_http=trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let producer: FutureProducer = ClientConfig::new()
        .set(
            "bootstrap.servers",
            format!(
                "{}:29092",
                std::env::var("KAFKA_HOST").unwrap_or_else(|_| "localhost".into())
            ),
        )
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    let app = Router::new().route("/", get(show_index)).route(
        "/api/v1/message",
        post(|Json(input): Json<Input>| async move {
            tracing::debug!("Received {:?}", &input);

            let delivery_status = producer
                .send(
                    FutureRecord::to("test")
                        .payload(&serde_json::to_string(&input).unwrap())
                        .key(&format!("Key {}", 1)),
                    Duration::from_secs(0),
                )
                .await;
            if let Err(e) = delivery_status {
                tracing::error!("Failed to deliver the message {:?}", e.0);
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "failed to deliver a message",
                );
            }
            tracing::debug!("Mesagge delivered successfuly");
            return (StatusCode::OK, "");
        }),
    );

    let app = app.fallback(handler_404.into_service());

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
                <ul>
                    <li><a href="/api/v1/message">/api/v1/message</a></li>
                </ul>
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

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}
