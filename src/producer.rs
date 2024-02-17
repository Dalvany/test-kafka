use std::time::Duration;

use log::{error, info};
use rdkafka::{
    message::{Header, OwnedHeaders},
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};

#[tokio::main]
async fn main() {
    let mut i = 0;
    let producer: &FutureProducer = &ClientConfig::new()
        .set("bootstrap.servers", "127.0.0.1:29092")
        .set("allow.auto.create.topics", "true")
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(500));

    loop {
        interval.tick().await;
        i += 1;
        let delivery_status = producer
            .send(
                FutureRecord::to("topic")
                    .payload(&format!("Message {}", i))
                    .key(&format!("Key {}", i))
                    .headers(OwnedHeaders::new().insert(Header {
                        key: "header_key",
                        value: Some("header_value"),
                    })),
                Duration::from_secs(0),
            )
            .await;

        match delivery_status {
            Ok(_) => info!("Message {i} delivered"),
            Err(error) => error!("Fail to send message {i} : {error:?}"),
        }
    }
}
