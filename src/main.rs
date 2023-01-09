use crossbeam_channel::unbounded;
use twilight_relayerAPI::kafka;
use twilight_relayerAPI::DatabaseArchiver;

const SNAPSHOT_TOPIC: &str = "CoreEventLogTopic";
const ARCHIVER_GROUP: &str = "Archiver";

fn main() {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .with_level(true)
        .with_line_number(true)
        .init();

    dotenv::dotenv().expect("No environment file found");

    let database_url = std::env::var("DATABASE_URL").expect("No database url found!");

    let (tx, rx) = unbounded();
    let _handle = kafka::start_consumer(ARCHIVER_GROUP.into(), SNAPSHOT_TOPIC.into(), tx);

    let database_worker = DatabaseArchiver::from_host(database_url);

    database_worker.run(rx).expect("Archiver loop quit unexpectedly!");
}
