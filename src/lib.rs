//! A very general library for IO based messages communication.
//!
//! # Documentation
//!
//! The library has 2 major parts:
//!
//! 1. Serializer
//! 2. Channel
//!
//! ## Serializer
//!
//! Serializer is a special struct (trait) which can read and write message
//! type from a bytes stream. You should manually implement this trait to some
//! struct and provide it with a message and error types.
//!
//! If feature `json-serializer` is enabled (it is enabled by default), then
//! you can use `JsonSerializer` trait which only has message and error types.
//! The serialization/deserialization logic is implemented using `serde_json`
//! crate.
//!
//! ## Channel
//!
//! Channel is a special struct (trait) which allows you to read and write
//! messages from generic reader and writer (rust's `Read` and `Write` traits).
//! There are many different traits related to channels. The main one is
//! `OwnedChannel`.
//!
//! The channel mod implements some handy functions for making channels from
//! the current process's stdio streams or unix sockets.
//!
//! ## Client and server
//!
//! An actual communication can be implemented using channels only. The library
//! provides some simple functions to create a client channel, a `Server` struct
//! to implement a messages listener, and a related `deamon` function to spawn
//! this `Server` struct in a background thread.
//!
//! In general you'd want to either:
//!
//! - Make two binaries: client and server. In server binary use `Server` struct
//!   to handle incoming messages, process them and return answers if needed.
//!   In client binary use `process_stdio` function to spawn the server binary
//!   and use stdin/stdout channel to communicate with it.
//! - Make client and server binaries again, but instead of using stdin/stdout
//!   channel you can use a unix socket. This can allow you to implement a
//!   multiple parties communication.
//!
//! # Example
//!
//! In this example we will spawn the server binary as a child of the client
//! binary process.
//!
//! ## Server
//!
//! ```ignore
//! use ioserverlib::prelude::*;
//! use ioserverlib::server::Server;
//!
//! // Implement our custom messages serializer. It must be compatible with the
//! // client's serializer struct.
//! struct MySerializer;
//!
//! impl JsonSerializer for MySerializer {
//!     // Any type which can be converted from `serde_json::Error`
//!     // and `std::io::Error`.
//!     type Error = Box<dyn std::error::Error>;
//!
//!     // Any `serde::Serialize` + `serde::de::DeserializeOwned` type.
//!     type Message = String;
//! }
//!
//! // Create communication channel using current process's stdin and stdout.
//! let channel = ioserverlib::channel::stdio(MySerializer);
//!
//! let mut server = Server::new(channel, |message| {
//!     match message {
//!         "ping" => Some(String::from("pong")),
//!
//!         // (!) use stderr because stdout is used for IPC.
//!         _ => {
//!             eprintln!("unknown command: {message}");
//!
//!             None
//!         }
//!     }
//! });
//!
//! // Listen to incoming messages in a loop and handle them.
//! loop {
//!     if let Err(err) = server.update() {
//!         // (!) use stderr because stdout is used for IPC.
//!         eprintln!("server error: {err}");
//!     }
//! }
//! ```
//!
//! ## Client
//!
//! ```ignore
//! use std::process::Command;
//!
//! use ioserverlib::prelude::*;
//!
//! // Implement our custom messages serializer. It must be compatible with the
//! // server's serializer struct.
//! struct MySerializer;
//!
//! impl JsonSerializer for MySerializer {
//!     // Any type which can be converted from `serde_json::Error`
//!     // and `std::io::Error`.
//!     type Error = Box<dyn std::error::Error>;
//!
//!     // Any `serde::Serialize` + `serde::de::DeserializeOwned` type.
//!     type Message = String;
//! }
//!
//! // Create communication channel with the spawned server binary process.
//! let command = Command::new("path/to/server");
//!
//! let (_, mut channel) = ioserverlib::client::process_stdio(command, MySerializer);
//!
//! channel.write(String::from("ping")).unwrap();
//!
//! assert_eq!(channel.read().unwrap(), "pong");
//! ```

pub mod serializer;
pub mod channel;
pub mod server;
pub mod client;

pub mod prelude {
    pub use super::serializer::Serializer;

    #[cfg(feature = "json-serializer")]
    pub use super::serializer::JsonSerializer;

    pub use super::channel::{
        ReadOnlyChannel,
        WriteOnlyChannel,
        ReadWriteChannel,
        OwnedChannel
    };
}
