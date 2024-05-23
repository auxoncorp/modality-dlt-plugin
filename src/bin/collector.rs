use std::collections::HashMap;

use anyhow::anyhow;
use auxon_sdk::{api::TimelineId, init_tracing, plugin_utils::ingest::Config};
use dlt_core::parse::ParsedMessage;
use modality_dlt::{dlt_message_to_event_attrs, dlt_message_to_event_name, TimelineKey};
use serde::{Deserialize, Serialize};
use tokio::{io::AsyncReadExt as _, net::TcpStream};
use tracing::warn;

#[derive(Serialize, Deserialize)]
struct CollectorConfig {
    /// The DLT host to connect to.
    ///
    ///If not given, defaults to "localhost".
    host: Option<String>,

    /// The port to connect to on the DLT host.
    ///
    /// If not given, defaults to 3490.
    port: Option<u16>,

    #[serde(flatten)]
    common: modality_dlt::Config,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing!();
    let config = Config::<CollectorConfig>::load("MODALITY_DLT_")?;

    let dlt_host = config.plugin.host.as_deref().unwrap_or("localhost");
    let dlt_port = config.plugin.port.unwrap_or(3490);
    let mut dlt_stream = TcpStream::connect((dlt_host, dlt_port)).await?;

    let client = config.connect_and_authenticate().await?;
    let mut sender = Sender::new(client, config);

    loop {
        let parsed_msg = read_dlt_message(&mut dlt_stream).await?;
        sender.handle_message(parsed_msg).await?;
    }
}

/// Read a single, complete DLT message from `stream`, and parse it.
async fn read_dlt_message(
    stream: &mut TcpStream,
) -> Result<dlt_core::parse::ParsedMessage, anyhow::Error> {
    // dlt_core only works on a byte buffer. So here we interpret the
    // framing part of the protocol just enough to determine the
    // message size, then read it into a buffer, and pass that down to
    // dlt_core.

    // Read the first byte, from which we can calulate the header size
    let header_type_byte = stream.read_u8().await?;
    let headers_len = dlt_core::dlt::calculate_all_headers_length(header_type_byte) as usize;

    // Read the whole header
    let mut header_buf = vec![0u8; headers_len];
    header_buf[0] = header_type_byte;
    stream.read_exact(&mut header_buf[1..]).await?;
    let (_, header) = dlt_core::parse::dlt_standard_header(&header_buf)?;

    // Read the payload. But it in a single buffer together with the header
    let total_message_size = headers_len + header.payload_length as usize;
    let mut msg_buf = vec![0u8; total_message_size];
    (&mut msg_buf[0..headers_len]).copy_from_slice(&header_buf);
    stream.read_exact(&mut msg_buf[headers_len..]).await?;

    let (remaining_data, dlt_msg) = dlt_core::parse::dlt_message(&msg_buf, None, false)?;
    if !remaining_data.is_empty() {
        return Err(anyhow!(
            "Remaining data after loading DLT message: {remaining_data:?}"
        ));
    }

    Ok(dlt_msg)
}

struct Sender {
    client: auxon_sdk::plugin_utils::ingest::Client,
    config: Config<CollectorConfig>,
    known_timelines: HashMap<TimelineKey, TimelineId>,
    current_timeline: Option<TimelineId>,
    event_ordering: u128,
}

impl Sender {
    fn new(
        client: auxon_sdk::plugin_utils::ingest::Client,
        config: Config<CollectorConfig>,
    ) -> Self {
        Self {
            client,
            config,
            known_timelines: Default::default(),
            current_timeline: None,
            event_ordering: 0,
        }
    }

    async fn handle_message(&mut self, parsed_msg: ParsedMessage) -> Result<(), Box<dyn std::error::Error>> {
        let msg = match parsed_msg {
            ParsedMessage::Item(msg) => msg,
            ParsedMessage::Invalid => {
                warn!("Dropping invalid message");
                return Ok(());
            }
            ParsedMessage::FilteredOut(_) => {
                return Ok(());
            }
        };

        let tl_key = TimelineKey::for_message(&msg, &self.config.plugin.common);
        match self.known_timelines.get(&tl_key) {
            Some(tl_id) => {
                // It's a known timeline; switch to it if necessary
                if self.current_timeline != Some(*tl_id) {
                    self.client.switch_timeline(*tl_id).await?;
                    self.current_timeline = Some(*tl_id);
                }
            }
            None => {
                // We've never seen this timeline before; allocate an
                // id, and send its attrs.
                let tl_id = TimelineId::allocate();

                self.client.switch_timeline(tl_id).await?;
                self.current_timeline = Some(tl_id);

                self.client
                    .send_timeline_attrs(tl_key.timeline_name().as_str(), tl_key.timeline_attrs())
                    .await?;
                self.known_timelines.insert(tl_key, tl_id);
            }
        };

        let ev_name = dlt_message_to_event_name(&msg);
        let ev_attrs = dlt_message_to_event_attrs(&msg);
        self.client
            .send_event(
                &ev_name,
                self.event_ordering,
                ev_attrs.iter().map(|(k, v)| (k.as_ref(), v.clone())),
            )
            .await?;

        self.event_ordering += 1;
        Ok(())
    }
}
