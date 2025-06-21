use std::io::{Read, Write, Stdin, Stdout, Stderr};

#[cfg(unix)]
use std::os::unix::net::UnixStream;

use crate::serializer::Serializer;

/// A single-direction channel which can read messages using provided serializer.
pub trait ReadOnlyChannel<R: Read> {
    fn try_read<W, S>(&mut self, serializer: &S) -> Result<Option<S::Message>, S::Error>
    where
        W: Write,
        S: Serializer<R, W>;

    fn read<W, S>(&mut self, serializer: &S) -> Result<S::Message, S::Error>
    where
        W: Write,
        S: Serializer<R, W>;
}

/// A single-direction channel which can write messages using provided serializer.
pub trait WriteOnlyChannel<W: Write> {
    fn write<R: Read, S: Serializer<R, W>>(
        &mut self,
        serializer: &S,
        message: S::Message
    ) -> Result<(), S::Error>;
}

/// A bi-directional channel which can read and write messages using provided serializer.
pub trait ReadWriteChannel<R: Read, W: Write>: ReadOnlyChannel<R> + WriteOnlyChannel<W> {}

impl<R: Read, W: Write, T> ReadWriteChannel<R, W> for T
where
    T: ReadOnlyChannel<R> + WriteOnlyChannel<W> {}

/// A bi-directional channel which can read and write messages using owned
/// serializer.
pub trait OwnedChannel<R: Read, W: Write, S: Serializer<R, W>> {
    fn reader(&mut self) -> &mut R;
    fn writer(&mut self) -> &mut W;
    fn serializer(&self) -> &S;

    fn try_read(&mut self) -> Result<Option<S::Message>, S::Error>;
    fn read(&mut self) -> Result<S::Message, S::Error>;
    fn write(&mut self, message: S::Message) -> Result<(), S::Error>;
}

/// Get read-only channel from the stdin stream.
#[inline]
pub fn stdin() -> ReadChannel<Stdin> {
    ReadChannel::new(std::io::stdin())
}

/// Get write-only channel from the stdout stream.
#[inline]
pub fn stdout() -> WriteChannel<Stdout> {
    WriteChannel::new(std::io::stdout())
}

/// Get write-only channel from the stderr stream.
#[inline]
pub fn stderr() -> WriteChannel<Stderr> {
    WriteChannel::new(std::io::stderr())
}

/// Get owned read-write channel from the stdin and stdout streams and provided
/// serializer.
#[inline]
pub fn stdio<S>(serializer: S) -> UniChannel<Stdin, Stdout, S>
where
    S: Serializer<Stdin, Stdout>
{
    UniChannel::new(std::io::stdin(), std::io::stdout(), serializer)
}

/// Get owned read-write channel from the stdin and stderr streams and provided
/// serializer.
#[inline]
pub fn stdie<S>(serializer: S) -> UniChannel<Stdin, Stderr, S>
where
    S: Serializer<Stdin, Stdout>
{
    UniChannel::new(std::io::stdin(), std::io::stderr(), serializer)
}

/// Get owned read-write unix socket channel from the provided path and
/// serializer.
#[cfg(unix)]
#[inline]
pub fn unix_socket<S>(
    path: impl AsRef<std::path::Path>,
    serializer: S
) -> std::io::Result<BiChannel<UnixStream, S>>
where
    S: Serializer<UnixStream, UnixStream>
{
    let socket = UnixStream::connect(path)?;

    Ok(BiChannel::new(socket, serializer))
}

/// Read-only channel abstraction over the generic reader.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReadChannel<R>(R);

impl<R> ReadChannel<R> {
    #[inline(always)]
    pub const fn new(reader: R) -> Self {
        Self(reader)
    }

    #[inline(always)]
    pub fn into_inner(self) -> R {
        self.0
    }
}

impl<R: Read> ReadOnlyChannel<R> for ReadChannel<R> {
    #[inline]
    fn try_read<W: Write, S: Serializer<R, W>>(
        &mut self,
        serializer: &S
    ) -> Result<Option<S::Message>, S::Error> {
        serializer.try_read(&mut self.0)
    }

    #[inline]
    fn read<W: Write, S: Serializer<R, W>>(
        &mut self,
        serializer: &S
    ) -> Result<S::Message, S::Error> {
        serializer.read(&mut self.0)
    }
}

impl<R: Read> From<R> for ReadChannel<R> {
    #[inline(always)]
    fn from(value: R) -> Self {
        Self(value)
    }
}

impl<R: Read> AsRef<R> for ReadChannel<R> {
    #[inline(always)]
    fn as_ref(&self) -> &R {
        &self.0
    }
}

impl<R: Read> AsMut<R> for ReadChannel<R> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut R {
        &mut self.0
    }
}

/// Write-only channel abstraction over the generic writer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WriteChannel<W>(W);

impl<W> WriteChannel<W> {
    #[inline(always)]
    pub const fn new(writer: W) -> Self {
        Self(writer)
    }

    #[inline(always)]
    pub fn into_inner(self) -> W {
        self.0
    }
}

impl<W: Write> WriteOnlyChannel<W> for WriteChannel<W> {
    #[inline]
    fn write<R: Read, S: Serializer<R, W>>(
        &mut self,
        serializer: &S,
        message: S::Message
    ) -> Result<(), S::Error> {
        serializer.write(&mut self.0, message)
    }
}

impl<W: Write> From<W> for WriteChannel<W> {
    #[inline(always)]
    fn from(value: W) -> Self {
        Self(value)
    }
}

impl<W: Write> AsRef<W> for WriteChannel<W> {
    #[inline(always)]
    fn as_ref(&self) -> &W {
        &self.0
    }
}

impl<W: Write> AsMut<W> for WriteChannel<W> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut W {
        &mut self.0
    }
}

/// Owned read-write channel abstraction over generic reader and writer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UniChannel<R, W, S> {
    reader: ReadChannel<R>,
    writer: WriteChannel<W>,
    serializer: S
}

impl<R, W, S> UniChannel<R, W, S> {
    #[inline]
    pub const fn new(reader: R, writer: W, serializer: S) -> Self {
        Self {
            reader: ReadChannel::new(reader),
            writer: WriteChannel::new(writer),
            serializer
        }
    }

    #[inline]
    pub fn into_inner(self) -> (R, W, S) {
        (self.reader.into_inner(), self.writer.into_inner(), self.serializer)
    }

    #[inline]
    pub fn transpose(self) -> UniChannel<W, R, S> {
        let (reader, writer, serializer) = self.into_inner();

        UniChannel::new(writer, reader, serializer)
    }
}

impl<R, W, S> OwnedChannel<R, W, S> for UniChannel<R, W, S>
where
    R: Read,
    W: Write,
    S: Serializer<R, W>
{
    #[inline]
    fn reader(&mut self) -> &mut R {
        self.reader.as_mut()
    }

    #[inline]
    fn writer(&mut self) -> &mut W {
        self.writer.as_mut()
    }

    #[inline(always)]
    fn serializer(&self) -> &S {
        &self.serializer
    }

    #[inline]
    fn try_read(&mut self) -> Result<Option<S::Message>, S::Error> {
        self.reader.try_read(&self.serializer)
    }

    #[inline]
    fn read(&mut self) -> Result<S::Message, S::Error> {
        self.reader.read(&self.serializer)
    }

    #[inline]
    fn write(&mut self, message: S::Message) -> Result<(), S::Error> {
        self.writer.write(&self.serializer, message)
    }
}

/// Owned read-write channel abstraction over the generic reader and writer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BiChannel<T, S> {
    io: T,
    serializer: S
}

impl<T, S> BiChannel<T, S> {
    #[inline(always)]
    pub const fn new(io: T, serializer: S) -> Self {
        Self {
            io,
            serializer
        }
    }

    #[inline(always)]
    pub fn into_inner(self) -> (T, S) {
        (self.io, self.serializer)
    }
}

impl<T, S> OwnedChannel<T, T, S> for BiChannel<T, S>
where
    T: Read + Write,
    S: Serializer<T, T>
{
    #[inline(always)]
    fn reader(&mut self) -> &mut T {
        &mut self.io
    }

    #[inline(always)]
    fn writer(&mut self) -> &mut T {
        &mut self.io
    }

    #[inline(always)]
    fn serializer(&self) -> &S {
        &self.serializer
    }

    #[inline]
    fn try_read(&mut self) -> Result<Option<S::Message>, S::Error> {
        self.serializer.try_read(&mut self.io)
    }

    #[inline]
    fn read(&mut self) -> Result<S::Message, S::Error> {
        self.serializer.read(&mut self.io)
    }

    #[inline]
    fn write(&mut self, message: S::Message) -> Result<(), S::Error> {
        self.serializer.write(&mut self.io, message)
    }
}
