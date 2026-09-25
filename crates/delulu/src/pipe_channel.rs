//! PS-B-03: the sandbox channel over two inherited pipes.
//!
//! A guest that runs under a separate identity cannot be reached the way PS-A reached it. There the
//! guest listened on a named pipe or socket in a channel directory and the host connected. A Windows
//! AppContainer lives in its own object namespace, so a pipe it creates is not one the host can open
//! by name, and a pipe the host creates is not one the container may open unless its security is
//! written for that container. Inherited handles need neither: the host makes the two pipes, the
//! guest is born holding its ends, and nothing is looked up by name on either side.
//!
//! What the named-pipe connection gave the channel and a plain pipe does not is a READ DEADLINE: an
//! anonymous pipe has no timeout. So the reading end is drained by one thread into a queue, and a read
//! waits on the queue with the deadline. The thread ends when the pipe does (end of file or an error),
//! which is also when the process at the other end is gone.

use std::io::{self, Read, Write};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::Duration;

/// How much one read of the pipe takes at most. Frames are length-prefixed, so a read of any size is
/// correct; this only bounds one queue entry.
const CHUNK: usize = 64 * 1024;

/// One side of the channel: reads come from the queue the drain thread fills, writes go straight to
/// the pipe.
pub struct PipeChannel<W: Write> {
    rx: Receiver<io::Result<Vec<u8>>>,
    pending: Vec<u8>,
    pos: usize,
    eof: bool,
    deadline: Option<Duration>,
    writer: W,
}

impl<W: Write> PipeChannel<W> {
    pub fn new<R: Read + Send + 'static>(mut reader: R, writer: W) -> io::Result<Self> {
        let (tx, rx) = mpsc::channel();
        std::thread::Builder::new().name("delulu-channel-read".into()).spawn(move || {
            let mut buf = vec![0u8; CHUNK];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        let _ = tx.send(Ok(Vec::new()));
                        return;
                    }
                    Ok(n) => {
                        if tx.send(Ok(buf[..n].to_vec())).is_err() {
                            return; // the channel was dropped; nobody is reading
                        }
                    }
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => {
                        let _ = tx.send(Err(e));
                        return;
                    }
                }
            }
        })?;
        Ok(PipeChannel { rx, pending: Vec::new(), pos: 0, eof: false, deadline: None, writer })
    }

    /// The same contract as the named-pipe connection's: `None` waits for ever, which no caller
    /// here uses, and a read past the deadline is `TimedOut`.
    pub fn set_read_timeout(&mut self, deadline: Option<Duration>) -> io::Result<()> {
        if deadline == Some(Duration::ZERO) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "a zero read deadline"));
        }
        self.deadline = deadline;
        Ok(())
    }
}

impl<W: Write> Read for PipeChannel<W> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if out.is_empty() {
            return Ok(0);
        }
        while self.pos >= self.pending.len() {
            if self.eof {
                return Ok(0);
            }
            let next = match self.deadline {
                Some(d) => self.rx.recv_timeout(d),
                None => self.rx.recv().map_err(|_| RecvTimeoutError::Disconnected),
            };
            match next {
                Ok(Ok(chunk)) if chunk.is_empty() => self.eof = true,
                Ok(Ok(chunk)) => {
                    self.pending = chunk;
                    self.pos = 0;
                }
                Ok(Err(e)) => {
                    self.eof = true;
                    return Err(e);
                }
                Err(RecvTimeoutError::Timeout) => {
                    return Err(io::Error::new(io::ErrorKind::TimedOut, "no frame from the other side before the channel deadline"))
                }
                Err(RecvTimeoutError::Disconnected) => self.eof = true,
            }
        }
        let n = out.len().min(self.pending.len() - self.pos);
        out[..n].copy_from_slice(&self.pending[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
    }
}

impl<W: Write> Write for PipeChannel<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A reader that yields its bytes one at a time and then blocks, so a test can tell "the data
    /// arrived" from "the deadline fired" without a real process at the other end.
    struct Slow(Vec<u8>, std::sync::mpsc::Receiver<()>);
    impl Read for Slow {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            if self.0.is_empty() {
                let _ = self.1.recv(); // block until the test is over
                return Ok(0);
            }
            out[0] = self.0.remove(0);
            Ok(1)
        }
    }

    #[test]
    fn bytes_arrive_in_order_and_a_silent_peer_hits_the_deadline_instead_of_hanging() {
        let (_hold, gate) = std::sync::mpsc::channel();
        let mut c = PipeChannel::new(Slow(b"frame".to_vec(), gate), Vec::new()).unwrap();
        c.set_read_timeout(Some(Duration::from_millis(200))).unwrap();
        let mut got = [0u8; 5];
        c.read_exact(&mut got).unwrap();
        assert_eq!(&got, b"frame");
        let started = std::time::Instant::now();
        let e = c.read(&mut got).unwrap_err();
        assert_eq!(e.kind(), io::ErrorKind::TimedOut);
        assert!(started.elapsed() < Duration::from_secs(5), "the deadline bounded the wait");
    }

    #[test]
    fn the_end_of_the_pipe_is_the_end_of_the_channel() {
        let mut c = PipeChannel::new(io::Cursor::new(b"ab".to_vec()), Vec::new()).unwrap();
        c.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut s = Vec::new();
        c.read_to_end(&mut s).unwrap();
        assert_eq!(s, b"ab");
        assert_eq!(c.read(&mut [0u8; 4]).unwrap(), 0, "and it stays ended");
        c.write_all(b"xy").unwrap();
        assert_eq!(c.writer, b"xy", "writes go straight through");
        assert!(c.set_read_timeout(Some(Duration::ZERO)).is_err(), "a zero deadline is refused, as a socket refuses it");
    }
}
