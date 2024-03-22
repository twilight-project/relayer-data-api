use crossbeam_channel::unbounded;
use log::warn;
use relayerarchiverlib::kafka;
use relayerarchiverlib::DatabaseArchiver;

const SNAPSHOT_TOPIC: &str = "CoreEventLogTopic";
const ARCHIVER_GROUP: &str = "Archiver";

fn main() {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .with_level(true)
        .with_line_number(true)
        .init();

    if let Err(_) = dotenv::dotenv() {
        warn!("DOTENV file not found");
    }

    let database_url = std::env::var("DATABASE_URL").expect("No database url found!");

    let (tx, rx) = unbounded();
    let (completions, _handle) =
        kafka::start_consumer(ARCHIVER_GROUP.into(), SNAPSHOT_TOPIC.into(), tx);

    let database_worker = DatabaseArchiver::from_host(database_url, completions);

    database_worker
        .run(rx)
        .expect("Archiver loop quit unexpectedly!");
}
