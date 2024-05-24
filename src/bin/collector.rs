use auxon_sdk::{init_tracing, plugin_utils::ingest::Config};
use modality_dlt::{
    read_dlt_message,
    send::{HasCommonConfig, Sender},
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tracing::info;

#[derive(Serialize, Deserialize)]
struct CollectorConfig {
    /// The DLT host to connect to (with TCP).
    ///
    ///If not given, defaults to "localhost".
    host: Option<String>,

    /// The TCP port to connect to on the DLT host.
    ///
    /// If not given, defaults to 3490.
    port: Option<u16>,

    #[serde(flatten)]
    common: modality_dlt::CommonConfig,
}

impl HasCommonConfig for CollectorConfig {
    fn common_config(&self) -> &modality_dlt::CommonConfig {
        &self.common
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing!();
    let config = Config::<CollectorConfig>::load("MODALITY_DLT_")?;

    let dlt_host = config.plugin.host.as_deref().unwrap_or("localhost");
    let dlt_port = config.plugin.port.unwrap_or(3490);
    let mut dlt_stream = TcpStream::connect((dlt_host, dlt_port)).await?;
    info!(%dlt_host, %dlt_port, "Connected to DLT server");

    let client = config.connect_and_authenticate().await?;
    info!("Connected to Modality backend");
    let mut sender = Sender::new(client, config);

    loop {
        let parsed_msg = read_dlt_message(&mut dlt_stream).await?;
        sender.handle_message(parsed_msg).await?;
    }
}
