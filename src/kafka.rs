use crossbeam_channel::Sender;
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
use std::thread::{self, JoinHandle};
use twilight_relayer_rust::db::EventLog;


pub fn start_consumer(group: String, topic: String, tx: Sender<EventLog>) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let broker = vec![std::env::var("BROKER")
            .expect("missing environment variable BROKER")
            .to_owned()];

        let mut con = Consumer::from_hosts(broker)
            .with_group(group)
            .with_topic_partitions(topic, &[0])
            .with_fallback_offset(FetchOffset::Earliest)
            .with_offset_storage(GroupOffsetStorage::Kafka)
            .create()
            .unwrap();

        let mut connection_status = true;
        while connection_status {
            let sender_clone = tx.clone();
            let mss = con.poll().unwrap();
            if !mss.is_empty() {
                for ms in mss.iter() {
                    for m in ms.messages() {
                        let message = EventLog {
                            offset: m.offset,
                            key: String::from_utf8_lossy(&m.key).to_string(),
                            value: serde_json::from_str(&String::from_utf8_lossy(&m.value))
                                .unwrap(),
                        };
                        match sender_clone.send(message) {
                            Ok(_) => { }
                            Err(_arg) => {
                                connection_status = false;
                                break;
                            }
                        }
                    }
                    let _ = con.consume_messageset(ms);
                }
                con.commit_consumed().unwrap();
            }
        }
        con.commit_consumed().unwrap();
        thread::park();
    })
}
