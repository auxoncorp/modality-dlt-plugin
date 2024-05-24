#![allow(unused)]
use std::path::PathBuf;

use auxon_sdk::{init_tracing, plugin_utils::ingest::Config};
use clap::Parser;
use modality_dlt::{
    consume_dlt_storage_header, read_dlt_message,
    send::{HasCommonConfig, Sender},
};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpStream,
};
use tracing::info;

#[derive(Serialize, Deserialize)]
struct ImporterConfig {
    #[serde(flatten)]
    common: modality_dlt::CommonConfig,
}

impl HasCommonConfig for ImporterConfig {
    fn common_config(&self) -> &modality_dlt::CommonConfig {
        &self.common
    }
}

#[derive(clap::Parser)]
struct ImporterOpts {
    dlt_file: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing!();
    let config = Config::<ImporterConfig>::load("MODALITY_DLT_")?;
    let opts = ImporterOpts::parse();

    let client = config.connect_and_authenticate().await?;
    let mut sender = Sender::new(client, config);
    info!("Connected to Modality");

    info!(
        file = %opts.dlt_file.display(),
        "Importing DLT messages from file"
    );
    let dlt_file = tokio::fs::File::open(opts.dlt_file).await?;
    let mut reader = BufReader::new(dlt_file);

    let mut message_count = 0;
    loop {
        if reader.fill_buf().await?.is_empty() {
            break;
        }
        consume_dlt_storage_header(&mut reader).await?;

        let parsed_msg = read_dlt_message(&mut reader).await?;
        sender.handle_message(parsed_msg).await?;
        message_count += 1;
    }

    info!(%message_count, "Finished importing");

    Ok(())
}
