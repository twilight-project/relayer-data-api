use jsonrpsee::{core::error::Error, server::logger::Params, RpcModule};
use serde::Serialize;

mod methods;

type HandlerType<R> =
    Box<dyn 'static + Fn(Params<'_>, &RelayerContext) -> Result<R, Error> + Send + Sync>;

pub struct RelayerContext;

fn register_method<R: Serialize + 'static>(
    module: &mut RpcModule<RelayerContext>,
    name: &'static str,
    method: HandlerType<R>,
) {
    if let Err(e) = module.register_method(name, method) {
        panic!("API failed to register {}! {:?}", name, e);
    }
}

pub fn init_methods() -> RpcModule<RelayerContext> {
    let mut module = RpcModule::new(RelayerContext {});

    register_method(&mut module, "hello_method", Box::new(methods::hello_method));

    module
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonrpsee::{
        core::{
            client::ClientT,
            params::{ArrayParams, ObjectParams},
        },
        http_client::HttpClientBuilder,
        server::ServerBuilder,
    };

    #[tokio::test]
    async fn test_hello() {
        let mut server = ServerBuilder::new()
            .build("0.0.0.0:8979")
            .await
            .expect("Builder failed");

        let handle = server
            .start(init_methods())
            .expect("Server failed to start");

        let client = HttpClientBuilder::default()
            .build("http://127.0.0.1:8979")
            .expect("Client builder failed");

        let response: String = client
            .request("hello_method", ObjectParams::new())
            .await
            .expect("Client call failed");

        assert_eq!("Hello, world!".to_string(), response);
    }
}
