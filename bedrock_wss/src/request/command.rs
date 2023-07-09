use super::{Header, MessagePurpose, MessageType};

impl super::Request {
    #[must_use]
    pub fn command(command: String, origin: Origin) -> Self {
        Self::Command(CommandRequest::new(command, origin))
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct CommandRequest {
    body: Body,
    pub header: Header,
}
impl CommandRequest {
    #[must_use]
    pub fn new(command: String, origin: Origin) -> Self {
        let header = Header::new(
            MessagePurpose::CommandRequest,
            Some(MessageType::CommandRequest),
        );
        let body = Body::new(command, origin);
        Self { body, header }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Body {
    command_line: String,
    origin: CommandOrigin,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct CommandOrigin {
    r#type: Origin,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Origin {
    Player,
}

impl Body {
    const fn new(command: String, origin: Origin) -> Self {
        Self {
            command_line: command,
            origin: CommandOrigin { r#type: origin },
        }
    }
}
