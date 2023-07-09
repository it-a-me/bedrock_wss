use std::net::SocketAddr;

use bedrock_wss::{
    re_exports::Uuid,
    request::{EventType, Header},
};
use futures::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    stream::SplitStream,
    SinkExt, StreamExt,
};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::instrument;

pub type McConnection = WebSocketStream<TcpStream>;
#[instrument]
pub async fn wait_for_connection(addr: &str) -> anyhow::Result<(McConnection, SocketAddr)> {
    tracing::info!("initializing minecraft web socket on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("waiting for connection");
    let (connection, mc_addr) = listener.accept().await?;
    let web_socket = tokio_tungstenite::accept_async(connection).await?;
    tracing::info!("initialized connection with {mc_addr}");
    Ok((web_socket, mc_addr))
}
pub fn message_handler(
    mc_stream: SplitStream<McConnection>,
) -> (
    UnboundedReceiver<MinecraftMessage>,
    tokio::task::JoinHandle<anyhow::Result<()>>,
) {
    let (sender, receiver) = futures::channel::mpsc::unbounded();
    (
        receiver,
        tokio::task::spawn(message_task(mc_stream, sender)),
    )
}

#[instrument]
async fn message_task(
    mut mc_stream: SplitStream<McConnection>,
    mut message_queue: UnboundedSender<MinecraftMessage>,
) -> anyhow::Result<()> {
    while let Some(message) = mc_stream.next().await {
        let message = message?;
        let Message::Text(message) = message else {
            tracing::warn!("minecraft sent a nonjson payload {}", message);
            continue;
        };
        match MinecraftMessage::try_from(message) {
            Ok(m) => message_queue.send(m).await?,
            Err(e) => tracing::error!("{e}"),
        }
    }
    Ok(())
}
pub enum MinecraftMessage {
    Subscription { event: EventType, content: String },
    Generic(String),
}
impl MinecraftMessage {
    pub fn content(&self) -> &str {
        match &self {
            Self::Subscription { event: _, content } | Self::Generic(content) => content,
        }
    }
    pub fn uuid(&self) -> anyhow::Result<Uuid> {
        let parsed = json::parse(self.content())?;
        Ok(Header::parse(&parsed["header"].dump())?.request_id)
    }
}
impl TryFrom<String> for MinecraftMessage {
    type Error = anyhow::Error;
    fn try_from(content: String) -> Result<Self, Self::Error> {
        let parsed = json::parse(&content)?;
        let purpose = parsed["header"]["messagePurpose"]
            .as_str()
            .ok_or(anyhow::anyhow!(
                "missing purpose in json payload \n{content}"
            ))?;
        if purpose == "event" {
            let event = parsed["header"]["eventName"]
                .as_str()
                .ok_or(anyhow::anyhow!(
                    "missing event name on minecraft event json\n{content}"
                ))?;
            let event = EventType::try_from(event)?;
            Ok(Self::Subscription { event, content })
        } else {
            Ok(Self::Generic(content))
        }
    }
}
