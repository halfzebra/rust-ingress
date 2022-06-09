use axum::{
    extract::{Extension, Json},
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
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use validator::{Validate, ValidationError};

#[derive(Debug)]
struct State {
    received: u64,
    processed: u64,
    errored: u64,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "received: {}, processed: {}, errored: {}",
            self.received, self.processed, self.errored
        )
    }
}

impl State {
    fn new() -> Self {
        State {
            received: 0,
            processed: 0,
            errored: 0,
        }
    }
}

type Counter = Arc<RwLock<State>>;

#[tokio::main]
async fn main() {
    let kafka_host = std::env::var("KAFKA_HOST").unwrap_or_else(|_| "localhost".into());
    let db = Counter::new(State::new().into());

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "ingress=trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::debug!("Using kafka_host {}", &kafka_host);

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", format!("{}:29092", kafka_host))
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    let app = Router::new()
        .route("/", get(show_index))
        .route(
            "/api/v1/message",
            post(
                |Json(input): Json<Input>, Extension(db): Extension<Counter>| async move {
                    tracing::debug!("Received {:?}", &input);
                    let mut c = db.write().unwrap();

                    c.received += 1;

                    let clone = Arc::clone(&db);

                    // don't wait for Kafka to respond, just return the status code to the user
                    tokio::spawn(async move {
                        match producer
                            .send(
                                FutureRecord::to("test")
                                    .payload(&serde_json::to_string(&input).unwrap())
                                    .key(&format!("Key {}", 1)),
                                Duration::from_secs(0),
                            )
                            .await
                        {
                            Ok(_) => {
                                tracing::debug!("Mesagge delivered successfuly");
                                let mut p = clone.write().unwrap();
                                p.processed += 1;
                            }
                            Err(e) => {
                                tracing::error!("Failed to deliver the message {:?}", e);
                                let mut p = clone.write().unwrap();
                                p.errored += 1;
                            }
                        };
                    });

                    return (StatusCode::OK, "");
                },
            )
            .get(|| async { (StatusCode::NOT_FOUND, "nothing to see here") }),
        )
        .layer(Extension(db));

    let app = app.fallback(handler_404.into_service());

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::debug!("listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn show_index(Extension(db): Extension<Counter>) -> Html<String> {
    let i = db.read().unwrap();

    Html(format!(
        r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                <section>
                    <h2>Endpoints</h2>
                    <ul>
                        <li><a href="/api/v1/message">/api/v1/message</a></li>
                    </ul>
                    <h2>Messaging stats</h2>
                    <span>{}<span>
                </section>
            </body>
        </html>
        "#,
        &i,
    ))
}

#[derive(Deserialize, Validate, Serialize, Debug)]
#[allow(dead_code)]
#[serde(deny_unknown_fields)]
struct Input {
    #[validate(custom = "validate_timestamp")]
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

fn validate_timestamp(ts: &str) -> Result<(), ValidationError> {
    ts.parse::<u64>()
        .map_err(|_e| ValidationError::new("Failed to parse timestamp"))
        .and_then(|n| {
            if 0 < n && n <= 2147_483_647 {
                return Ok(());
            }
            Err(ValidationError::new(
                "number is outside of UNIX Epoch range",
            ))
        })
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn timestamp() {
        assert_eq!(validate_timestamp("1"), Ok(()));
        assert_eq!(validate_timestamp("21474836473").is_err(), true);
    }
}
