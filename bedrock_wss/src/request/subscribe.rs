use super::{EventType, Header, MessagePurpose, MessageType};

impl super::Request {
    #[must_use]
    pub fn subscribe(event: EventType) -> Self {
        Self::Subscribe(Subscribe::new(event))
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Subscribe {
    pub header: Header,
    body: Body,
}
impl Subscribe {
    pub fn new(event: EventType) -> Self {
        let header = Header::new(MessagePurpose::Subscribe, Some(MessageType::CommandRequest));
        let body = Body::new(event);
        Self { header, body }
    }
    pub const fn event(&self) -> EventType {
        self.body.event_name
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Body {
    event_name: EventType,
}

impl Body {
    const fn new(event_name: EventType) -> Self {
        Self { event_name }
    }
}
