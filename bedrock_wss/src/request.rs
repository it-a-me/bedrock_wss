pub mod command;
mod subscribe;
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Request {
    Subscribe(subscribe::Subscribe),
    Command(command::CommandRequest),
}
impl Request {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        match self {
            Self::Subscribe(subscribe) => serde_json::to_string(subscribe),
            Self::Command(c) => serde_json::to_string(c),
        }
    }
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        match self {
            Self::Subscribe(subscribe) => serde_json::to_string_pretty(subscribe),
            Self::Command(c) => serde_json::to_string_pretty(c),
        }
    }
    pub fn header(&self) -> &Header {
        match self {
            Self::Subscribe(subscribe) => &subscribe.header,
            Self::Command(command) => &command.header,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub request_id: uuid::Uuid,
    message_purpose: MessagePurpose,
    message_type: Option<MessageType>,
    version: u32,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
enum MessagePurpose {
    Event,
    Subscribe,
    #[serde(rename = "CommandRequest")]
    CommandRequest,
    CommandResponse,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
enum MessageType {
    CommandRequest,
}

impl Header {
    fn new(message_purpose: MessagePurpose, message_type: Option<MessageType>) -> Self {
        Self {
            request_id: uuid::Uuid::new_v4(),
            message_purpose,
            message_type,
            version: 1,
        }
    }
    pub fn parse(json: &str) -> Result<Self, serde_json::Error> {
        Ok(serde_json::from_str(json)?)
    }
}

#[derive(
    serde::Serialize,
    serde::Deserialize,
    Copy,
    Clone,
    PartialEq,
    Eq,
    clap::ValueEnum,
    Hash,
    strum::AsRefStr,
    strum::EnumIter,
    strum::EnumString,
)]
#[serde(rename_all = "PascalCase")]
pub enum EventType {
    PlayerMessage,
}
