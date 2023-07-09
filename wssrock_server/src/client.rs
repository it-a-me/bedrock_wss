use std::{collections::HashMap, net::SocketAddr};

use bedrock_wss::request::Request;
use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    SinkExt, StreamExt,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};

pub fn listen(
    listener: TcpListener,
) -> (
    JoinHandle<anyhow::Result<()>>,
    UnboundedReceiver<ClientRequest>,
) {
    let (message_sender, message_receiver) = unbounded();
    let joinhandle = tokio::spawn(listener_task(listener, message_sender));
    (joinhandle, message_receiver)
}
async fn listener_task(
    listener: TcpListener,
    sender: UnboundedSender<ClientRequest>,
) -> anyhow::Result<()> {
    tracing::info!("listening for clients");
    let mut connections: HashMap<SocketAddr, JoinHandle<anyhow::Result<()>>> = HashMap::new();
    loop {
        let (client, addr) = listener.accept().await?;
        tracing::info!("client connected from {addr}");
        let client_connection = request_handler(client, addr, sender.clone()).await?;
        if connections.insert(addr, client_connection).is_some() {
            anyhow::bail!("tried to connect over {addr} but a client was already connected");
        }
        let client_addrs = connections.keys().copied().collect::<Vec<_>>();
        for addr in client_addrs {
            if connections[&addr].is_finished() {
                connections.remove(&addr).unwrap().await??;
            }
        }
    }
}
pub struct ClientRequest {
    pub request: Request,
    pub response_vector: UnboundedSender<String>,
}

impl ClientRequest {
    fn new(request: Request) -> (Self, UnboundedReceiver<String>) {
        let (response_vector, receiver) = unbounded();
        (
            Self {
                request,
                response_vector,
            },
            receiver,
        )
    }
}

async fn request_handler(
    mut client: TcpStream,
    addr: SocketAddr,
    mut sender: UnboundedSender<ClientRequest>,
) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    let request = {
        let mut bytes = Vec::new();
        client.read_to_end(&mut bytes).await?;
        postcard::from_bytes(&bytes)?
    };
    let (client_request, mut responses) = ClientRequest::new(request);
    let read_handle = tokio::spawn(async move {
        sender.send(client_request).await?;
        while let Some(response) = responses.next().await {
            if client.write(response.as_bytes()).await.is_err() {
                break;
            }
        }
        tracing::info!("client disconnected from {addr}");
        Ok::<_, anyhow::Error>(())
    });
    Ok(read_handle)
}
