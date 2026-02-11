use crossbeam_channel::unbounded;
use relayerarchiverlib::kafka;
use relayerarchiverlib::DatabaseArchiver;

// const SNAPSHOT_TOPIC: &str = "CoreEventLogTopic";
// const ARCHIVER_GROUP: &str = "Archiver_Redis";

fn main() {
    if let Err(_) = dotenv::dotenv() {
        eprintln!("DOTENV file not found");
    }

    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .with_level(true)
        .with_line_number(true)
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("No database url found!");
    let redis_url = std::env::var("ORDERBOOK_REDIS").expect("No redis url found!");

    let snapshot_topic = std::env::var("CORE_EVENT_LOG").unwrap_or("CoreEventLogTopic".to_string());
    let archiver_group =
        std::env::var("ARCHIVER_KAFKA_GROUP").unwrap_or("Archiver_Redis".to_string());
    let (tx, rx) = unbounded();
    let (completions, _handle) = kafka::start_consumer(archiver_group, snapshot_topic, tx);

    let database_worker = DatabaseArchiver::from_host(&database_url, &redis_url, completions);

    database_worker
        .run(rx)
        .expect("Archiver loop quit unexpectedly!");
}
