use std::{io, os::unix::prelude::AsRawFd};

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
        let size = nix::unistd::write(self.seq.as_raw_fd(), bytes)?;

        if bytes.len() != size {
            unimplemented!("Message does not fit in the buffer");
        }

        Ok(())
    }
}
