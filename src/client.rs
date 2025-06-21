use std::io::{BufReader, BufWriter};
use std::process::{Command, Child, Stdio, ChildStdin, ChildStdout, ChildStderr};

use crate::serializer::Serializer;
use crate::channel::UniChannel;

/// Spawn new process using the provided command and messages serializer,
/// return child handler and a read-write channel which holds the spawned
/// process's stdin and stdout streams. This channel can be used for IPC if the
/// spawned process runs a `Server`.
#[allow(clippy::type_complexity)]
pub fn process_stdio<S: Serializer<BufReader<ChildStdout>, BufWriter<ChildStdin>>>(
    command: &mut Command,
    serializer: S
) -> std::io::Result<(
    Child,
    UniChannel<BufReader<ChildStdout>, BufWriter<ChildStdin>, S>
)> {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let Some(stdin) = child.stdin.take() else {
        return Err(std::io::Error::other("spawned process stdin pipe is missing"));
    };

    let Some(stdout) = child.stdout.take() else {
        return Err(std::io::Error::other("spawned process stdout pipe is missing"));
    };

    let channel = UniChannel::new(
        BufReader::new(stdout),
        BufWriter::new(stdin),
        serializer
    );

    Ok((child, channel))
}

/// Spawn new process using the provided command and messages serializer,
/// return child handler and a read-write channel which holds the spawned
/// process's stdin and stderr streams. This channel can be used for IPC if the
/// spawned process runs a `Server`.
#[allow(clippy::type_complexity)]
pub fn process_stdie<S: Serializer<BufReader<ChildStderr>, BufWriter<ChildStdin>>>(
    command: &mut Command,
    serializer: S
) -> std::io::Result<(
    Child,
    UniChannel<BufReader<ChildStderr>, BufWriter<ChildStdin>, S>
)> {
    let mut child = command
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let Some(stdin) = child.stdin.take() else {
        return Err(std::io::Error::other("spawned process stdin pipe is missing"));
    };

    let Some(stderr) = child.stderr.take() else {
        return Err(std::io::Error::other("spawned process stderr pipe is missing"));
    };

    let channel = UniChannel::new(
        BufReader::new(stderr),
        BufWriter::new(stdin),
        serializer
    );

    Ok((child, channel))
}
