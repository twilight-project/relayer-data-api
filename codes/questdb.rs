use crate::relayer::CloseTrade;
use crate::relayer::Side;
use std::io::prelude::*;
use std::net::TcpStream;
use std::sync::Mutex;
use std::time::SystemTime;

//** for insertion */
lazy_static! {
    pub static ref QUESTDB_INFLUX: Mutex<TcpStream> =
        Mutex::new(connect().expect("No connection found for QuestDB"));
}
pub fn connect() -> Result<TcpStream, std::io::Error> {
    dotenv::dotenv().expect("Failed loading dotenv");
    let questdb_url = std::env::var("QUESTDB_INFLUX_URL")
        .expect("missing environment variable QUESTDB_INFLUX_URL");
    return match TcpStream::connect(questdb_url) {
        Ok(stream) => Ok(stream),
        Err(arg) => Err(std::io::Error::new(std::io::ErrorKind::Other, arg)),
    };
}

pub fn send_candledata_in_questdb(data: CloseTrade) -> Result<(), std::io::Error> {
    // let data = b"recentorders side=6i,price=1814.47,amount=287122.05005 1556813561098000000\n";
    let mut stream = QUESTDB_INFLUX.lock().unwrap();
    let query = format!(
        "recentorders side={}i,price={},amount={} {}\n",
        (match data.side {
            Side::SELL => 0,
            Side::BUY => 1,
        }),
        data.price,
        data.positionsize,
        data.timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string()
    );
    // println!("{:#?}", stream);
    match stream.write(query.as_bytes()) {
        Ok(x) => {
            if x < query.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    format!("Sent {}/{} bytes", x, query.len()),
                ));
            }
        }
        Err(arg) => return Err(std::io::Error::new(std::io::ErrorKind::Other, arg)),
    }

    match stream.flush() {
        Ok(_) => {}
        Err(arg) => return Err(std::io::Error::new(std::io::ErrorKind::Other, arg)),
    }
    drop(stream);
    Ok(())
}

pub fn send_candledata_in_questdb_pending(data: CloseTrade) -> Result<(), std::io::Error> {
    // let data = b"recentorders side=6i,price=1814.47,amount=287122.05005 1556813561098000000\n";
    let mut stream = connect().unwrap();
    let query = format!(
        "recentorders side={}i,price={},amount={} {}\n",
        (match data.side {
            Side::SELL => 0,
            Side::BUY => 1,
        }),
        data.price,
        data.positionsize,
        data.timestamp
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string()
    );
    // println!("{:#?}", stream);
    match stream.write(query.as_bytes()) {
        Ok(x) => {
            if x < query.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    format!("Sent {}/{} bytes", x, query.len()),
                ));
            }
        }
        Err(arg) => return Err(std::io::Error::new(std::io::ErrorKind::Other, arg)),
    }

    match stream.flush() {
        Ok(_) => {}
        Err(arg) => return Err(std::io::Error::new(std::io::ErrorKind::Other, arg)),
    }
    drop(stream);
    Ok(())
}


//when processing new trader order request  or updating any pending order on limit price
let side = match ordertx.position_type {
    PositionType::SHORT => Side::SELL,
    PositionType::LONG => Side::BUY,
};
update_recent_orders(CloseTrade {
    side: side,
    positionsize: ordertx.positionsize,
    price: ordertx.entryprice,
    timestamp: std::time::SystemTime::now(),
});

// when processing open order for settlement or liquidation 
//Note:  short will buy the btc and long will sell
 // adding candle data
 let side = match ordertx.position_type {
    PositionType::SHORT => Side::BUY,
    PositionType::LONG => Side::SELL,
};
update_recent_orders(CloseTrade {
    side: side,
    positionsize: ordertx.positionsize,
    price: ordertx.entryprice,
    timestamp: std::time::SystemTime::now(),
});


// send_candledata_in_questdb is getting used for inserting data in quest db (which is using influxDB protocol)
pub fn update_recent_orders(value: CloseTrade) {
    let threadpool = THREADPOOL.lock().unwrap();
    let value_clone = value.clone();
    threadpool.execute(move || {
        let mut local_storage = RECENTORDER.lock().unwrap();
        if local_storage.len() > 500 {
            local_storage.pop_back();
        }
        local_storage.push_front(value);
        drop(local_storage);
    });
    drop(threadpool);
    let threadpool_questdb = THREADPOOL_QUESTDB.lock().unwrap();
    threadpool_questdb.execute(move || {
        let mut currect_pending_chart = CURRENTPENDINGCHART.lock().unwrap();
        let value_safe_cloned = value_clone.clone();
        match send_candledata_in_questdb(value_safe_cloned.clone()) {
            Ok(_) => {
                currect_pending_chart.push_front(value_safe_cloned.clone());
                if currect_pending_chart.len() > 2 {
                    currect_pending_chart.pop_back();
                }
            }
            Err(arg) => {
                println!("QuestDB not reachable!!, {:#?}", arg);
                println!("trying to establish new connection...");
                match questdb::connect() {
                    Ok(value) => {
                        let mut stream = QUESTDB_INFLUX.lock().unwrap();
                        *stream = value;
                        drop(stream);
                        println!("new connection established");
                        thread::sleep(time::Duration::from_millis(1000));
                        for i in 0..currect_pending_chart.len() {
                            send_candledata_in_questdb_pending(
                                currect_pending_chart.pop_back().unwrap(),
                            )
                            .expect("error in loop");
                        }
                        QueueResolver::executor(String::from("questdb_queue"));
                        send_candledata_in_questdb_pending(value_safe_cloned)
                            .expect("error after queue resolver");
                    }
                    Err(arg) => {
                        println!("QuestDB not reachable!!, {:#?}", arg);
                        QueueResolver::pending(
                            move || {
                                send_candledata_in_questdb_pending(value_safe_cloned)
                                    .expect("error inside queue resolver");
                            },
                            String::from("questdb_queue"),
                        );
                    }
                }
                // *stream = questdb::connect().expect("No connection found for QuestDB");
            }
        }
        drop(currect_pending_chart);
    });
    drop(threadpool_questdb);
    // threadpool.execute(move || {
    // });
}




//** for query */
//query for reading data from questDB

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
// #[serde(rename_all = "camelCase")]
pub struct CandleAdvance {
    pub low: f64,
    pub high: f64,
    pub open: f64,
    pub close: f64,
    pub sell_volume: f64,
    pub buy_volume: f64,
    pub trades: i64,
    #[serde(rename = "startedAt")]
    pub startedat: String,
    #[serde(rename = "updatedAt")]
    pub updatedat: String,
}

pub fn get_candle_advance(
    sample_by: String,
    limit: i32,
    pagination: i32,
) -> Result<CandlesAdvance, std::io::Error> {
    let public_threadpool = PUBLIC_THREADPOOL.lock().unwrap();
    let (sender, receiver) = mpsc::channel();
    public_threadpool.execute(move || {
        let start_row=limit*pagination;
        let last_row=limit*pagination+limit;
        let query = format!(" Select t3.TradesCount,t3.startedAt,t3.updatedAt,t3.open,t3.close,t3.min,t3.max,coalesce(t3.sell_volume , 0) as Sell_Volume, coalesce(t4.buy_volume , 0) as Buy_Volume from ( 

            Select t1.*,t2.sell_volume from ( 
                SELECT timestamp, first(price) AS open, last(price) AS close, min(price), max(price) ,count as TradesCount,first(timestamp) as startedAt,last(timestamp) as updatedAt
                    FROM recentorders WHERE timestamp > dateadd('d', -30, now())
                      SAMPLE BY {} ALIGN TO CALENDAR) t1 
          
                LEFT OUTER JOIN (
          
                SELECT timestamp,sum(amount) AS sell_volume 
                    FROM recentorders WHERE side=0 AND timestamp > dateadd('d', -30, now())
                          SAMPLE BY {} ALIGN TO CALENDAR) t2 ON t1.timestamp = t2.timestamp ) t3 
          LEFT OUTER JOIN ( 
          
          SELECT timestamp, sum(amount) AS buy_volume 
                FROM recentorders WHERE side=1 AND timestamp > dateadd('d', -30, now())
                      SAMPLE BY {} ALIGN TO CALENDAR) t4 ON t3.timestamp = t4.timestamp Limit {},{} ;",&sample_by,&sample_by,&sample_by,start_row,last_row);
        let mut client = QUESTDB_POOL_CONNECTION.get().unwrap();
        let mut candle_data: Vec<CandleAdvance> = Vec::new();
        match client.query(&query, &[]) {
            Ok(data) => {
                for row in data {
                    // let ttime: std::time::SystemTime = row.get("timestamp");
                    candle_data.push(CandleAdvance {
                        low: row.get("min"),
                        high: row.get("max"),
                        open: row.get("open"),
                        close: row.get("close"),
                        sell_volume: row.get("Sell_Volume"),
                        buy_volume: row.get("Buy_Volume"),
                        trades: row.get("TradesCount"),
                        startedat:ServerTime::new(row.get("startedAt")).epoch,
                        updatedat:ServerTime::new(row.get("updatedAt")).epoch,
                    });
                }
                sender.send(Ok(candle_data)).unwrap();
            }
            Err(arg) => sender
                .send(Err(std::io::Error::new(std::io::ErrorKind::Other, arg)))
                .unwrap(),
        }
    });
    drop(public_threadpool);
    // println!("{:#?}", receiver.recv().unwrap());
    match receiver.recv().unwrap() {
        Ok(value) => {
            return Ok(CandlesAdvance { candles: value });
        }
        Err(arg) => {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, arg));
        }
    };
}


// inside lazy static
lazy_static! {
pub static ref QUESTDB_POOL_CONNECTION: r2d2::Pool<PostgresConnectionManager<NoTls>> = {
    dotenv::dotenv().expect("Failed loading dotenv");
    // POSTGRESQL_URL
    let postgresql_url =
        std::env::var("QUESTDB_URL").expect("missing environment variable POSTGRESQL_URL");
    let manager = PostgresConnectionManager::new(
        // TODO: PLEASE MAKE SURE NOT TO USE HARD CODED CREDENTIALS!!!
        postgresql_url.parse().unwrap(),
        NoTls,
    );
    r2d2::Pool::new(manager).unwrap()
};
}