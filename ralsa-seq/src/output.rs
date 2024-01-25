use std::{io, os::unix::prelude::AsRawFd};

use rustix::fd::{AsFd, RawFd};

use super::{event, Seq};

#[derive(Debug)]
pub struct SeqOutput {
    seq: Seq,
}

impl SeqOutput {
    pub(crate) fn new(seq: Seq) -> Self {
        Self { seq }
    }

    pub fn seq(&self) -> &Seq {
        &self.seq
    }

    /// Output an event directly to the sequencer NOT through output buffer
    ///
    /// This function sends an event to the sequencer directly not through the
    /// output buffer.  
    pub fn send(&mut self, event: &event::Event) -> io::Result<()> {
        let bytes = event.event_bytes();
        let size = rustix::io::write(&self.seq, bytes)?;

        if bytes.len() != size {
            unimplemented!("Message does not fit in the buffer");
        }

        Ok(())
    }
}

impl AsRawFd for SeqOutput {
    fn as_raw_fd(&self) -> RawFd {
        self.seq.as_raw_fd()
    }
}

impl AsFd for SeqOutput {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        self.seq.as_fd()
    }
}
