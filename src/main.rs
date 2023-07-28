mod api;
mod client;
mod error;
mod pandoc;

use std::borrow::Cow;

pub use error::{Error, Result};

#[tokio::main]
async fn main() -> crate::Result<()> {
    // Start tracing
    tracing_subscriber::fmt().compact().init();

    let (address, port) = address_from_env();
    tracing::info!("Server will bind to {address}:{port}");
    let server_handle = api::spawn_server((address, port))?;
    
    server_handle.await.map_err(Into::into)
}

fn address_from_env() -> (Cow<'static, str>, u16) {
    let address = std::env::var("ADDRESS")
        .map(Into::into)
        .unwrap_or("127.0.0.1".into());

    let port = std::env::var("PORT")
        .map(|port| port.parse())
        .into_iter()
        .flatten()
        .next()
        .unwrap_or(8080);

    (address, port)
}
