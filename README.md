# ioserverlib

ioserverlib is a rust library for IO-based message communication. It provides
a general framework for building client-server applications with customizable
messages serialization and deserialization and communication channels over
standard `Read` and `Write` traits, including unix sockets. This library is
meant to simplify implementation of custom IPC protocols.

## Example

The example demonstrates how to spawn a server binary as a child process of the
client binary and communicate with it using standard input/output streams.

### Server

```rust
use ioserverlib::prelude::*;
use ioserverlib::server::Server;

struct MySerializer;

impl JsonSerializer for MySerializer {
    type Error = Box<dyn std::error::Error>;
    type Message = String;
}

fn main() {
    let channel = ioserverlib::channel::stdio(MySerializer);

    let mut server = Server::new(channel, |message| {
        match message {
            "ping" => Some(String::from("pong")),

            _ => {
                eprintln!("unknown command: {}", message);

                None
            }
        }
    });

    loop {
        if let Err(err) = server.update() {
            eprintln!("server error: {}", err);
        }
    }
}
```

### Client

```rust
use std::process::Command;

use ioserverlib::prelude::*;

struct MySerializer;

impl JsonSerializer for MySerializer {
    type Error = Box<dyn std::error::Error>;
    type Message = String;
}

fn main() {
    let command = Command::new("path/to/server");

    let (_, mut channel) = ioserverlib::client::process_stdio(command, MySerializer);

    channel.write(String::from("ping")).unwrap();

    assert_eq!(channel.read().unwrap(), "pong");
}
```

Author: [Nikita Podvirnyi](https://github.com/krypt0nn)\
Licensed under [MIT](LICENSE)
