use auxon_sdk::api::{AttrKey, AttrVal, Nanoseconds};
use dlt_core::dlt::{self, ControlType, LogLevel};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    /// Should the ecu id be used as part of timeline identity and naming? Defaults to true.
    pub timeline_from_ecu_id: Option<bool>,

    /// Should the session be used as part of timeline identity? (not naming) Defaults to true.
    pub timeline_from_session_id: Option<bool>,

    /// Should the application id field be used as part of timeline identity and naming? Defaults to false.
    pub timeline_from_application_id: Option<bool>,

    /// Should the context id field be used as part of timeline identity and naming? Defaults to false.
    pub timeline_from_context_id: Option<bool>,
}

#[derive(Eq, PartialEq, Hash, Default)]
pub struct TimelineKey {
    ecu_id: Option<String>,
    session_id: Option<String>,
    application_id: Option<String>,
    context_id: Option<String>,
}

impl TimelineKey {
    pub fn for_message(msg: &dlt::Message, config: &Config) -> Self {
        let mut key = TimelineKey::default();
        if config.timeline_from_ecu_id.unwrap_or(true) {
            key.ecu_id = msg.header.ecu_id.clone();
        }

        if config.timeline_from_session_id.unwrap_or(true) {
            key.session_id = msg.header.ecu_id.clone();
        }

        if config.timeline_from_application_id.unwrap_or(false) {
            key.application_id = msg
                .extended_header
                .as_ref()
                .map(|eh| eh.application_id.clone());
        }

        if config.timeline_from_context_id.unwrap_or(false) {
            key.context_id = msg.extended_header.as_ref().map(|eh| eh.context_id.clone());
        }

        key
    }

    pub fn timeline_name(&self) -> String {
        let s = self
            .ecu_id
            .as_deref()
            .clone()
            .into_iter()
            .chain(self.application_id.as_deref().clone().into_iter())
            .chain(self.context_id.as_deref().clone().into_iter())
            .collect::<Vec<_>>()
            .join(".");

        if s.is_empty() {
            "unnamed".to_string()
        } else {
            s
        }
    }

    pub fn timeline_attrs(&self) -> Vec<(&'static str, AttrVal)> {
        let mut attrs = vec![];

        if let Some(ecu_id) = self.ecu_id.as_ref() {
            attrs.push(("timeline.ecu_id", ecu_id.into()));
        }

        if let Some(session_id) = self.session_id.as_ref() {
            attrs.push(("timeline.session_id", session_id.into()));
        }

        if let Some(application_id) = self.application_id.as_ref() {
            attrs.push(("timeline.application_id", application_id.into()));
        }

        if let Some(context_id) = self.context_id.as_ref() {
            attrs.push(("timeline.context_id", context_id.into()));
        }

        attrs
    }
}

pub fn dlt_message_to_event_name(msg: &dlt::Message) -> String {
    match msg.extended_header.as_ref() {
        Some(extended_header) => match &extended_header.message_type {
            dlt::MessageType::Log(_) => "log".to_string(),
            dlt::MessageType::ApplicationTrace(_) => "application_trace".to_string(),
            dlt::MessageType::NetworkTrace(_) => "network_trace".to_string(),
            dlt::MessageType::Control(_) => "control".to_string(),
            dlt::MessageType::Unknown(_) => "unknown".to_string(),
        },
        None => match msg.payload {
            dlt::PayloadContent::Verbose(_) => "verbose".to_string(),
            dlt::PayloadContent::NonVerbose(_, _) => "non_verbose".to_string(),
            dlt::PayloadContent::ControlMsg(_, _) => "control".to_string(),
        },
    }
}

pub fn dlt_message_to_event_attrs(msg: &dlt::Message) -> Vec<(AttrKey, AttrVal)> {
    let mut attrs: Vec<(AttrKey, AttrVal)> = vec![];

    gather_header_attrs(msg, &mut attrs);

    if let Some(extended_header) = &msg.extended_header {
        gather_extended_header_attrs(&mut attrs, extended_header);
    }

    gather_payload(msg, &mut attrs);

    attrs
}

fn gather_header_attrs(msg: &dlt::Message, attrs: &mut Vec<(AttrKey, AttrVal)>) {
    if let Some(ecu_id) = &msg.header.ecu_id {
        attrs.push(("event.ecu_id".into(), ecu_id.clone().into()));
    }

    if let Some(session_id) = &msg.header.session_id {
        attrs.push(("event.session_id".into(), (*session_id).into()));
    }

    if let Some(timestamp) = &msg.header.timestamp {
        attrs.push((
            "event.timestamp".into(),
            Nanoseconds::from(*timestamp as u64).into(),
        ));
    }
}

fn gather_extended_header_attrs(
    attrs: &mut Vec<(AttrKey, AttrVal)>,
    extended_header: &dlt::ExtendedHeader,
) {
    attrs.push((
        "event.application_id".into(),
        extended_header.application_id.clone().into(),
    ));

    attrs.push((
        "event.context_id".into(),
        extended_header.context_id.clone().into(),
    ));

    gather_message_type_attrs(extended_header, attrs);
}

fn gather_message_type_attrs(
    extended_header: &dlt::ExtendedHeader,
    attrs: &mut Vec<(AttrKey, AttrVal)>,
) {
    match &extended_header.message_type {
        dlt::MessageType::Log(log_level) => {
            attrs.push(("event.message_type".into(), "log".into()));
            attrs.push((
                "event.log_level".into(),
                log_level_to_str(*log_level).into(),
            ));
        }
        dlt::MessageType::ApplicationTrace(trace_type) => {
            attrs.push(("event.message_type".into(), "application_trace".into()));
            attrs.push((
                "event.application_trace_type".into(),
                trace_type.as_ref().to_lowercase().into(),
            ));
        }
        dlt::MessageType::NetworkTrace(trace_type) => {
            attrs.push(("event.message_type".into(), "network_trace".into()));
            attrs.push((
                "event.network_trace_type".into(),
                trace_type.as_ref().to_lowercase().into(),
            ));
        }
        dlt::MessageType::Control(control_type) => {
            attrs.push(("event.message_type".into(), "control".into()));
            attrs.push((
                "event.control_type".into(),
                control_type_to_str(control_type).into(),
            ));
        }
        dlt::MessageType::Unknown(_) => {
            attrs.push(("event.message_type".into(), "unknown".into()));
        }
    }
}

fn gather_payload(msg: &dlt::Message, attrs: &mut Vec<(AttrKey, AttrVal)>) {
    match &msg.payload {
        dlt::PayloadContent::Verbose(args) => {
            attrs.push(("event.payload_type".into(), "verbose".into()));

            // Special case a single non-named arg as "event.payload"
            if args.len() == 1 && args[0].name.is_none() {
                if let Some(attr_val) = value_to_attr_val(args[0].value.clone()) {
                    attrs.push(("event.payload".into(), attr_val));
                }
            } else {
                for (arg_id, arg) in args.iter().enumerate() {
                    let Some(attr_val) = value_to_attr_val(arg.value.clone()) else {
                        continue;
                    };

                    let attr_key = if let Some(name) = &arg.name {
                        if name.is_empty() {
                            format!("event.payload.{arg_id}")
                        } else {
                            format!("event.payload.{name}")
                        }
                    } else {
                        format!("event.payload.{arg_id}")
                    };
                    attrs.push((attr_key.into(), attr_val));
                }
            }
        }

        dlt::PayloadContent::NonVerbose(message_id, _payload) => {
            // TODO interpret message_id and payload using an "external description"
            attrs.push(("event.payload_type".into(), "non_verbose".into()));
            attrs.push(("event.message_id".into(), (*message_id).into()));
        }

        dlt::PayloadContent::ControlMsg(control_type, _control_bytes) => {
            attrs.push(("event.payload_type".into(), "control".into()));
            attrs.push((
                "event.control_type".into(),
                control_type_to_str(&control_type).into(),
            ));
        }
    }
}

fn log_level_to_str(log_level: LogLevel) -> &'static str {
    match log_level {
        LogLevel::Fatal => "fatal",
        LogLevel::Error => "error",
        LogLevel::Warn => "warn",
        LogLevel::Info => "info",
        LogLevel::Debug => "debug",
        LogLevel::Verbose => "verbose",
        LogLevel::Invalid(_) => "invalid",
    }
}

fn control_type_to_str(control_type: &ControlType) -> &'static str {
    match control_type {
        ControlType::Request => "request",
        ControlType::Response => "response",
        ControlType::Unknown(_) => "unknown",
    }
}

fn value_to_attr_val(value: dlt::Value) -> Option<AttrVal> {
    match value {
        dlt::Value::Bool(x) => {
            if x == 0 {
                Some(false.into())
            } else {
                Some(true.into())
            }
        }
        dlt::Value::U8(x) => Some(x.into()),
        dlt::Value::U16(x) => Some(x.into()),
        dlt::Value::U32(x) => Some(x.into()),
        dlt::Value::U64(x) => Some(x.into()),
        dlt::Value::U128(x) => {
            if x < i128::MAX as u128 {
                Some((x as i128).into())
            } else {
                //tracing::warn!("Dropping integer that is too large for Modality");
                None
            }
        }
        dlt::Value::I8(x) => Some(x.into()),
        dlt::Value::I16(x) => Some(x.into()),
        dlt::Value::I32(x) => Some(x.into()),
        dlt::Value::I64(x) => Some(x.into()),
        dlt::Value::I128(x) => Some(x.into()),
        dlt::Value::F32(x) => Some(x.into()),
        dlt::Value::F64(x) => Some(x.into()),
        dlt::Value::StringVal(x) => Some(x.into()),
        dlt::Value::Raw(_) => None,
    }
}
