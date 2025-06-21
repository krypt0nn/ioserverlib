# ioserverlib

ioserverlib is a rust library for IO-based message communication. It provides
a general framework for building client-server applications with customizable
messages serialization and deserialization and communication channels over
standard `Read` and `Write` traits, including unix sockets. This library is
meant to simplify implementation of custom IPC protocols.

## Example

The example demonstrates how to spawn a server binary as a child process of the
client binary and communicate with it using standard input/output streams.

```rust
use std::process::Command;

use ioserverlib::prelude::*;
use ioserverlib::server::Server;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
enum Message {
    Ping,
    Pong
}

struct Serializer;

impl JsonSerializer for Serializer {
    type Error = Box<dyn std::error::Error>;
    type Message = Message;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);

    match args.next().as_deref() {
        Some("client") => {
            let mut command = Command::new(std::env::current_exe()?);

            let command = command.arg("server");

            let (mut child, mut channel) = ioserverlib::client::process_stdio(command, Serializer)?;

            channel.write(Message::Ping)?;

            dbg!(channel.read()?);

            child.kill()?;
        }

        Some("server") => {
            let channel = ioserverlib::channel::stdio(Serializer);

            let mut server = Server::new(channel, |message| {
                match message {
                    Message::Ping => Some(Message::Pong),
                    Message::Pong => None
                }
            });

            loop {
                if let Err(err) = server.update() {
                    eprintln!("server error: {err}");
                }
            }
        }

        Some(command) => eprintln!("unknown command: {command}"),
        _ => eprintln!("missing command: client or server")
    }

    Ok(())
}
```

> $ cargo run \-\- client
>
> [src/main.rs:32:13] channel.read()? = Pong

> $ echo '"Ping"' | cargo run \-\- server
>
> "Pong"

Author: [Nikita Podvirnyi](https://github.com/krypt0nn)\
Licensed under [MIT](LICENSE)
