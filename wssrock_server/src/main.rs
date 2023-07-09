#![warn(
    clippy::complexity,
    clippy::correctness,
    clippy::pedantic,
    clippy::perf
)]
#![allow(clippy::module_name_repetitions)]
use std::collections::HashMap;

use bedrock_wss::{
    re_exports::{IntoEnumIterator, Uuid},
    request::EventType,
};
use clap::Parser;
use client::ClientRequest;
use futures::{channel::mpsc::UnboundedSender, stream::SplitSink, SinkExt, StreamExt};
use minecraft::McConnection;
use tokio_tungstenite::tungstenite::Message;
mod client;
mod minecraft;
type EventSubs = HashMap<EventType, Vec<futures::channel::mpsc::UnboundedSender<String>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(args.log_level)
        .with_file(true)
        .with_line_number(true)
        .init();
    let (mc_connection, _mc_addr) = minecraft::wait_for_connection(&args.minecraft_address).await?;
    let (mut mc_sink, mc_stream) = mc_connection.split();
    let (mut message_queue, mc_connection) = minecraft::message_handler(mc_stream);
    let client_listener = tokio::net::TcpListener::bind(args.client_address).await?;
    let (client_listener, mut client_messages) = client::listen(client_listener);
    let mut event_subs: EventSubs = EventType::iter().map(|t| (t, Vec::new())).collect();
    let mut response_waiters: HashMap<Uuid, UnboundedSender<String>> = HashMap::new();
    loop {
        tokio::select! {
            message = message_queue.next() => {
                if let Some((uuid, generic_response)) = handle_mc_message(message, &mut event_subs).await? {
                    if let Some(mut response_waiter) = response_waiters.remove(&uuid) {
                        response_waiter.send(generic_response).await?;
                    } else {
                        tracing::error!("reponse with no waiter {generic_response}");
                    }
                } else {
                    break
                }
            }
            request = client_messages.next() => {
                match request {
                    Some(request) =>  {
                        let request_id = request.request.header().request_id;
                        if let Some(want_response) = handle_client_message(request, &mut event_subs, &mut mc_sink).await? {
                            response_waiters.insert(request_id, want_response);
                        }
                    }
                    None => break,
                }
            }
        }
    }
    client_listener.abort();
    mc_connection.abort();
    tracing::error!("client: {:#?}", client_listener.await);
    tracing::error!("server: {:#?}", mc_connection.await);
    Ok(())
}

async fn handle_client_message(
    client_request: ClientRequest,
    event_subs: &mut EventSubs,
    mc_sink: &mut SplitSink<McConnection, Message>,
) -> anyhow::Result<Option<UnboundedSender<String>>> {
    mc_sink
        .send(Message::Text(client_request.request.to_json()?))
        .await?;
    let ClientRequest {
        request,
        response_vector,
    } = client_request;
    match request {
        bedrock_wss::request::Request::Subscribe(subscribe) => {
            event_subs
                .get_mut(&subscribe.event())
                .unwrap()
                .push(response_vector);
            Ok(None)
        }
        bedrock_wss::request::Request::Command(_) => Ok(Some(response_vector)),
    }
}
async fn handle_mc_message(
    message: Option<minecraft::MinecraftMessage>,
    event_subs: &mut EventSubs,
) -> anyhow::Result<Option<(Uuid, String)>> {
    let Some(message) = message else {
        anyhow::bail!("minecraft message queue closed unexpectedly");
    };
    let uuid = message.uuid()?;
    match message {
        minecraft::MinecraftMessage::Subscription { event, content } => {
            let subs = event_subs.get_mut(&event).unwrap();
            subs.retain(|c| !c.is_closed());
            for sub in event_subs.get_mut(&event).unwrap() {
                if !sub.is_closed() {
                    sub.send(content.clone()).await?;
                }
            }
            Ok(None)
        }
        minecraft::MinecraftMessage::Generic(m) => Ok(Some((uuid, m))),
    }
}

#[derive(clap::Parser)]
struct Cli {
    #[arg(short, long, default_value_t = tracing::Level::DEBUG)]
    log_level: tracing::Level,
    #[arg(short, long, default_value_t = String::from(bedrock_wss::DEFAULT_WSS_PORT))]
    minecraft_address: String,
    #[arg(short, long, default_value_t = String::from(bedrock_wss::DEFAULT_CLIENT_PORT))]
    client_address: String,
}
