use std::io::{Read, Write};
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::serializer::Serializer;
use crate::channel::OwnedChannel;

/// Spawn new thread with a new `Server` struct which will listen to incoming
/// messages in a loop, process them using the `messages_handler` and use
/// `errors_handler` on any occuring errors.
///
/// If `errors_handler` returns `true`, then the thread will be closed.
pub fn daemon<R, W, S, C, H, E>(
    channel: C,
    messages_handler: H,
    errors_handler: E
) -> Daemon
where
    R: Read + Send + 'static,
    W: Write + Send + 'static,
    S: Serializer<R, W> + Send + 'static,
    C: OwnedChannel<R, W, S> + Send + 'static,
    H: Fn(S::Message) -> Option<S::Message> + Send + 'static,
    E: Fn(S::Error) -> bool + Send + 'static
{
    let mut server = Server::new(channel, messages_handler);

    let alive = Arc::new(AtomicBool::new(true));

    {
        let alive = alive.clone();

        std::thread::spawn(move || {
            while alive.load(Ordering::Relaxed) {
                if let Err(err) = server.update() {
                    if (errors_handler)(err) {
                        alive.store(false, Ordering::Release);

                        break;
                    }
                }
            }
        });
    }

    Daemon(alive)
}

pub struct Server<R, W, S, C, H> {
    _reader: PhantomData<R>,
    _writer: PhantomData<W>,
    _serializer: PhantomData<S>,
    channel: C,
    handler: H
}

impl<R, W, S, C, H> Server<R, W, S, C, H>
where
    R: Read,
    W: Write,
    S: Serializer<R, W>,
    C: OwnedChannel<R, W, S>,
    H: Fn(S::Message) -> Option<S::Message>
{
    #[inline(always)]
    pub const fn new(channel: C, handler: H) -> Self {
        Self {
            _reader: PhantomData,
            _writer: PhantomData,
            _serializer: PhantomData,
            channel,
            handler
        }
    }

    #[inline(always)]
    pub fn channel(&mut self) -> &mut C {
        &mut self.channel
    }

    pub fn update(&mut self) -> Result<(), S::Error> {
        let message = self.channel.read()?;

        if let Some(response) = (self.handler)(message) {
            self.channel.write(response)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Daemon(Arc<AtomicBool>);

impl Daemon {
    /// Check if the underlying server thread is still running.
    #[inline]
    pub fn is_alive(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    /// Kill the underlying server thread.
    #[inline]
    pub fn kill(self) {
        self.0.store(false, Ordering::Release);
    }
}
