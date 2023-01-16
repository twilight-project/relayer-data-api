use super::RelayerContext;
use jsonrpsee::{core::error::Error, server::logger::Params};

pub(super) fn hello_method(_params: Params<'_>, _ctx: &RelayerContext) -> Result<String, Error> {
    Ok("Hello, world!".into())
}
