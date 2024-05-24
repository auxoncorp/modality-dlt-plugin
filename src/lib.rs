pub mod convert;
pub mod send;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt as _};
use auxon_sdk::plugin_utils::serde::from_str;

#[derive(Serialize, Deserialize)]
pub struct CommonConfig {
    /// Should the ecu id be used as part of timeline identity and naming? Defaults to true.
    #[serde(default, deserialize_with = "from_str")]
    pub timeline_from_ecu_id: Option<bool>,

    /// Should the session be used as part of timeline identity? (not naming) Defaults to true.
    #[serde(default, deserialize_with = "from_str")]
    pub timeline_from_session_id: Option<bool>,

    /// Should the application id field be used as part of timeline identity and naming? Defaults to false.
    #[serde(default, deserialize_with = "from_str")]
    pub timeline_from_application_id: Option<bool>,

    /// Should the context id field be used as part of timeline identity and naming? Defaults to false.
    #[serde(default, deserialize_with = "from_str")]
    pub timeline_from_context_id: Option<bool>,
}

/// Read a single, complete DLT message from `stream`, and parse it.
pub async fn read_dlt_message<S>(
    stream: &mut S,
) -> Result<dlt_core::parse::ParsedMessage, anyhow::Error>
where
    S: AsyncRead + Unpin,
{
    // dlt_core only works on a byte buffer. So here we interpret the
    // framing part of the protocol just enough to determine the
    // message size, then read it into a buffer, and pass that down to
    // dlt_core.

    // Read the first byte, from which we can calculate the header size
    let header_type_byte = stream.read_u8().await?;
    let headers_len = dlt_core::dlt::calculate_all_headers_length(header_type_byte) as usize;

    // Read the whole header
    let mut header_buf = vec![0u8; headers_len];
    header_buf[0] = header_type_byte;
    stream.read_exact(&mut header_buf[1..]).await?;
    let (_, header) = dlt_core::parse::dlt_standard_header(&header_buf)?;

    // Read the payload. Put it in a single buffer together with the header
    let total_message_size = headers_len + header.payload_length as usize;
    let mut msg_buf = vec![0u8; total_message_size];
    msg_buf[0..headers_len].copy_from_slice(&header_buf);
    stream.read_exact(&mut msg_buf[headers_len..]).await?;

    let (remaining_data, dlt_msg) = dlt_core::parse::dlt_message(&msg_buf, None, false)?;
    if !remaining_data.is_empty() {
        return Err(anyhow!(
            "Remaining data after loading DLT message: {remaining_data:?}"
        ));
    }

    Ok(dlt_msg)
}

/// Try to read DLT storage header from `stream`. Return an error if we couldn't.
pub async fn consume_dlt_storage_header<S>(stream: &mut S) -> Result<(), anyhow::Error>
where
    S: AsyncRead + Unpin,
{
    // dlt_core only works on a byte buffer. So, we're going to read
    // a storage-header's worth of data from the stream, then use the library
    // to verify it has the right shape.
    let mut storage_header_buf = [0u8; 16];
    stream
        .read_buf(&mut storage_header_buf.as_mut_slice())
        .await?;

    let (remaining_data, read_size) = dlt_core::parse::skip_storage_header(&storage_header_buf)?;
    if !remaining_data.is_empty() || read_size != 16 {
        return Err(anyhow!("Invalid DLT storage header"));
    }

    Ok(())
}
