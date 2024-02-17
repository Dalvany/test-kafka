use log::{info, warn, LevelFilter};
use rdkafka::{
    config::RDKafkaLogLevel,
    consumer::{Consumer, ConsumerContext, StreamConsumer},
    ClientConfig, ClientContext, Message,
};

struct Context;

impl ClientContext for Context {
    fn stats(&self, statistics: rdkafka::Statistics) {
        let mut lag = 0;

        if let Some(topic) = statistics.topics.get("topic") {
            for partition in topic.partitions.values() {
                lag += partition.consumer_lag;
            }
        }

        info!("Messages remaining to consume : {}", lag as f64);
    }
}

impl ConsumerContext for Context {}

#[tokio::main]
async fn main() {
    env_logger::Builder::default()
        .filter(None, LevelFilter::Info)
        .filter(Some("librdkafka"), LevelFilter::Trace)
        .filter(Some("rdkafka::consumer"), LevelFilter::Debug)
        .init();

    let context = Context;

    let mut config = ClientConfig::new();

    #[cfg(feature = "stats")]
    config.set("statistics.interval.ms", "7000");

    let consumer: StreamConsumer<Context> = config
        .set("group.id", "my-group")
        .set("bootstrap.servers", "127.0.0.1:29092")
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        //.set("max.poll.interval.ms", "6000")
        .set("enable.auto.commit", "true")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create_with_context(context)
        .expect("Consumer creation failed");

    consumer
        .subscribe(&["topic"])
        .expect("Can't subscribe to specified topics");

    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(750));
    let mut count = 0;

    loop {
        interval.tick().await;
        count += 1;
        match consumer.recv().await {
            Err(error) => warn!("Fail to read message {count} : {error:?}"),
            Ok(message) => {
                let key = message.key();
                let payload = match message.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        warn!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };
                info!("Key : {key:?}");
                info!("Payload : {payload}");
            }
        }
    }
}
