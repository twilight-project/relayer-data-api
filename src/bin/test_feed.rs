use chrono::prelude::*;
use kafka::producer::{Producer, Record, RequiredAcks};
use rand::Rng;
use relayerarchiverlib::{rpc, ws};
use std::{net::SocketAddr, time::Duration};
use tokio::time::sleep;
use twilight_relayer_rust::db::Event;


#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("dotenv file not found!");
    let mut rng = rand::thread_rng();
    let mut current_price: f64 = rng.gen_range(60000.0..80000.0);

    let broker_host = std::env::var("BROKER").expect("missing environment variable BROKER");
    let topic = std::env::var("TRADERORDER_EVENT_LOG").expect("No topic!!!!");

    let broker = vec![broker_host.to_owned()];
    let mut kafka = Producer::from_hosts(broker)
        .with_ack_timeout(Duration::from_secs(1))
        .with_required_acks(RequiredAcks::One)
        .create()
        .unwrap();


    loop {
        current_price += rng.gen_range(-50.0..50.0);
        let system_time = Utc::now().to_rfc3339();

        println!("New price: {}, {}", current_price, system_time);
        let event = Event::CurrentPriceUpdate(current_price, system_time);
        let serialized = serde_json::to_vec(&event).expect("Phuque");
        let record = Record::from_key_value(&topic, "CurrentPriceUpdate", serialized);

        kafka.send(&record).expect("NOOOOO!!");

        sleep(Duration::from_secs(1)).await;
    }

}
