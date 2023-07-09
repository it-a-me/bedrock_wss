use std::io::{Read, Write};

use bedrock_wss::request::{command::Origin, Request};
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let message = match args.command {
        Command::Subscribe { event } => Request::subscribe(event),
        Command::Command { command } => Request::command(command, Origin::Player),
    };
    eprintln!("{}", message.to_json_pretty()?);
    let mut connection = std::net::TcpStream::connect(args.port)?;
    eprintln!("connected");
    let message = postcard::to_allocvec(&message)?;
    connection.write(&message)?;
    connection.shutdown(std::net::Shutdown::Write)?;
    eprintln!("awaiting response");
    let mut buf = Vec::new();
    for byte in connection.bytes() {
        let byte = byte?;
        buf.push(byte);
        if let Ok(printable) = String::from_utf8(buf.clone()) {
            print!("{printable}");
            buf.clear();
        } else if buf.len() > 20 {
            panic!("invalid utf8")
        }
    }
    Ok(())
}

#[derive(clap::Parser)]
struct Cli {
    #[arg(short, long, default_value_t = String::from(bedrock_wss::DEFAULT_CLIENT_PORT))]
    port: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand, Clone)]
enum Command {
    Subscribe {
        event: bedrock_wss::request::EventType,
    },
    Command {
        command: String,
    },
}
