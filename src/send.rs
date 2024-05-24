use std::collections::HashMap;

use auxon_sdk::{api::TimelineId, plugin_utils::ingest::Config};
use dlt_core::parse::ParsedMessage;
use tracing::warn;

use crate::{
    convert::{dlt_message_to_event_attrs, dlt_message_to_event_name, TimelineKey},
    CommonConfig,
};

pub struct Sender<C: HasCommonConfig> {
    client: auxon_sdk::plugin_utils::ingest::Client,
    config: Config<C>,
    known_timelines: HashMap<TimelineKey, TimelineId>,
    current_timeline: Option<TimelineId>,
    event_ordering: u128,
}

pub trait HasCommonConfig {
    fn common_config(&self) -> &CommonConfig;
}

impl<C: HasCommonConfig> Sender<C> {
    pub fn new(client: auxon_sdk::plugin_utils::ingest::Client, config: Config<C>) -> Self {
        Self {
            client,
            config,
            known_timelines: Default::default(),
            current_timeline: None,
            event_ordering: 0,
        }
    }

    pub async fn handle_message(
        &mut self,
        parsed_msg: ParsedMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
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

        let tl_key = TimelineKey::for_message(&msg, self.config.plugin.common_config());
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
