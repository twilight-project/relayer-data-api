use crossbeam_channel::{unbounded, Sender};
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
use log::{error, info};
use std::thread::{self, JoinHandle};
use twilight_relayer_rust::db::Event;

// > 500 offset behind, we'll batch update.
const CATCHUP_INTERVAL: i64 = 500;
pub type Completion = (i32, i64);

pub fn start_consumer(
    group: String,
    topic: String,
    tx: Sender<(Completion, Vec<Event>, bool)>,
) -> (Sender<Completion>, JoinHandle<()>) {
    let (tx_consumed, rx_consumed) = unbounded::<Completion>();

    let handle = std::thread::spawn(move || {
        let broker_host = std::env::var("BROKER").expect("missing environment variable BROKER");

        info!("Connecting to kafka at host: {}", broker_host);
        let broker = vec![broker_host.to_owned()];

        let mut con = Consumer::from_hosts(broker)
            .with_group(group)
            .with_topic(topic.clone())
            .with_fallback_offset(FetchOffset::Earliest)
            .with_offset_storage(GroupOffsetStorage::Kafka)
            .create()
            .unwrap();

        con.client_mut().load_metadata_all().unwrap();

        let mut connection_status = true;
        while connection_status {
            let sender_clone = tx.clone();
            let mss = con.poll().unwrap();
            let latest = con
                .client_mut()
                .fetch_topic_offsets(&topic, kafka::client::FetchOffset::Latest)
                .unwrap();
            let latest = latest[0].offset;
            if !mss.is_empty() {
                for ms in mss.iter() {
                    let mut max_offset = 0i64;

                    let events: Vec<Event> = ms
                        .messages()
                        .iter()
                        .map(|m| {
                            max_offset = max_offset.max(m.offset);

                            let msg_data = String::from_utf8_lossy(&m.value);
                            let message: Event = match serde_json::from_str(&msg_data) {
                                Ok(event) => event,
                                Err(e) => {
                                    println!("Invalid message! {:?} {}\n", e, msg_data);
                                    // continue;
                                    Event::Stop(e.to_string())
                                }
                            };
                            message
                        })
                        .collect();

                    let token = (ms.partition(), max_offset);
                    let catchup = latest - max_offset > CATCHUP_INTERVAL;

                    match sender_clone.send((token, events, catchup)) {
                        Ok(_) => {}
                        Err(_arg) => {
                            connection_status = false;
                            break;
                        }
                    }
                }

                while !rx_consumed.is_empty() {
                    match rx_consumed.recv() {
                        Ok((partition, offset)) => {
                            let e = con.consume_message(&topic, partition, offset);

                            if e.is_err() {
                                error!("Kafka connection failed {:?}", e);
                                connection_status = false;
                                break;
                            }

                            let e = con.commit_consumed();
                            if e.is_err() {
                                error!("Kafka connection failed {:?}", e);
                                connection_status = false;
                                break;
                            }
                        }
                        Err(e) => {
                            connection_status = false;
                            error!("The consumed channel is closed: {:?}", e);
                            break;
                        }
                    }
                }
            }
        }
        con.commit_consumed().unwrap();
        thread::park();
    });

    (tx_consumed, handle)
}
