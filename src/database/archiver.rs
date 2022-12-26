use crossbeam_channel::Receiver;
use twilight_relayer_rust::db::EventLog;


pub struct DatabaseArchiver {
    database_url: String,
}

impl DatabaseArchiver {
    pub fn from_host(database_url: String) -> DatabaseArchiver {
        DatabaseArchiver {
            database_url
        }
    }

    pub fn run(self, rx: Receiver<EventLog>) {
        loop {
            while let Ok(msg) = rx.recv() {
                println!("TJDEBUG got a msg {:#?}", msg);
            }
        }
    }
}
