use jsonrpsee::server::ServerBuilder;
use log::info;
use std::{net::SocketAddr, time::Duration};
use structopt::StructOpt;
use tokio::time::sleep;
use twilight_relayerAPI::{rpc, ws};

#[derive(Debug, StructOpt)]
#[structopt(name = "Relayer API", about = "Twilight Relayer API server")]
struct Opt {
    #[structopt(
        short = "-l",
        long,
        default_value("0.0.0.0:8989"),
        help = "Address the server will listen on"
    )]
    listen_addr: SocketAddr,
    #[structopt(
        short = "-w",
        long,
        default_value("0.0.0.0:8990"),
        help = "Websocket address the server will listen on"
    )]
    ws_listen_addr: SocketAddr,
}

#[tokio::main]
async fn main() {
    let opts = Opt::from_args();
    dotenv::dotenv().expect("dotenv file not found!");

    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .with_level(true)
        .with_line_number(true)
        .init();

    let addrs: &[SocketAddr] = &[opts.listen_addr];
    let database_url = std::env::var("DATABASE_URL").expect("No database url found!");

    info!("Starting RPC server on {:?}", opts.listen_addr);
    let server = ServerBuilder::new()
        .build(addrs)
        .await
        .expect("Failed to build API server");

    let methods = rpc::init_methods(&database_url);
    let _handle = server.start(methods).expect("Failed to start API server");

    let ws_addrs: &[SocketAddr] = &[opts.ws_listen_addr];
    info!("Starting WS server on {:?}", opts.ws_listen_addr);
    let ws_server = ServerBuilder::new()
        .build(ws_addrs)
        .await
        .expect("Failed to build websocket server");

    let ws_methods = ws::init_methods(&database_url);
    let _ws_handle = ws_server
        .start(ws_methods)
        .expect("Failed to start websocket server");

    //TODO: need an exit handler.
    loop {
        sleep(Duration::from_secs(5)).await;
    }

    //handle.stop().expect("Oopsie!");
}
