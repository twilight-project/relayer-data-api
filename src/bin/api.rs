use jsonrpsee::server::ServerBuilder;
use log::info;
use std::{net::SocketAddr, time::Duration};
use structopt::StructOpt;
use tokio::time::sleep;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use twilight_relayerAPI::{rpc, ws};

#[derive(Debug, StructOpt)]
#[structopt(name = "Relayer API", about = "Twilight Relayer API server")]
struct Opt {
    #[structopt(
        short = "-p",
        long = "--public-rpc",
        default_value("0.0.0.0:8987"),
        help = "Endpoint for the public API."
    )]
    public_rpc: SocketAddr,
    #[structopt(
        short = "-s",
        long = "--private-rpc",
        default_value("0.0.0.0:8989"),
        help = "Endpoint for the private API."
    )]
    private_rpc: SocketAddr,
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

    let database_url = std::env::var("DATABASE_URL").expect("No database url found!");
    info!("Database backend: {}", database_url);

    let cors = CorsLayer::new()
        .allow_headers(Any)
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers([hyper::header::CONTENT_TYPE]);

    // TODO: env var
    let middleware = ServiceBuilder::new().layer(cors);

    info!("Starting public RPC server on {:?}", opts.public_rpc);
    let addrs: &[SocketAddr] = &[opts.public_rpc];
    let public_server = ServerBuilder::new()
        .set_middleware(middleware.clone())
        .build(addrs)
        .await
        .expect("Failed to build public API server");

    let methods = rpc::init_public_methods(&database_url);
    let _pub_handle = public_server
        .start(methods)
        .expect("Failed to start API server");

    info!("Starting private RPC server on {:?}", opts.private_rpc);
    let addrs: &[SocketAddr] = &[opts.private_rpc];
    let private_server = ServerBuilder::new()
        .set_middleware(middleware.clone())
        .build(addrs)
        .await
        .expect("Failed to build private API server");

    let methods = rpc::init_private_methods(&database_url);
    let _priv_handle = private_server
        .start(methods)
        .expect("Failed to start API server");

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
