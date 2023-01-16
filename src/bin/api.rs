use jsonrpsee::server::ServerBuilder;
use std::{net::SocketAddr, time::Duration};
use structopt::StructOpt;
use tokio::time::sleep;
use twilight_relayerAPI::rpc;

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
}

#[tokio::main]
async fn main() {
    let opts = Opt::from_args();

    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .with_level(true)
        .with_line_number(true)
        .init();

    let addrs: &[SocketAddr] = &[opts.listen_addr];

    let server = ServerBuilder::new()
        .build(addrs)
        .await
        .expect("Failed to build API server");

    let methods = rpc::init_methods();

    let _handle = server.start(methods).expect("Failed to start API server");

    //TODO: need an exit handler.
    loop {
        sleep(Duration::from_secs(5)).await;
    }

    //handle.stop().expect("Oopsie!");
}
