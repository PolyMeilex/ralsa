use std::{
    io,
    os::{
        fd::AsFd,
        unix::prelude::{AsRawFd, RawFd},
    },
};

use rustix::event::PollFlags;

use super::{event, Seq, CELL_SIZE, SEQ_INPUT_BUF_SIZE};

#[derive(Debug)]
pub struct SeqInput {
    seq: Seq,
    input_buffer: Vec<u8>,
    cell_count: usize,
    cell_id: usize,
}

impl SeqInput {
    pub(crate) fn new(seq: Seq) -> Self {
        Self {
            seq,
            input_buffer: vec![0u8; SEQ_INPUT_BUF_SIZE * CELL_SIZE],
            cell_count: 0,
            cell_id: 0,
        }
    }

    pub fn seq(&self) -> &Seq {
        &self.seq
    }

    pub fn read(&mut self) -> io::Result<()> {
        let len = rustix::io::read(&self.seq, &mut self.input_buffer)?;

        self.cell_count = len / CELL_SIZE;
        self.cell_id = 0;

        Ok(())
    }

    pub fn has_input_events(&self) -> bool {
        self.cell_count != 0
    }

    pub fn input_event(&mut self, fetch_sequencer: bool) -> Option<event::Event<'_>> {
        // If there is no events check if fd was read fully
        // Or is there data still left in it
        if !self.has_input_events() && fetch_sequencer {
            let pool_fd = rustix::event::PollFd::new(self, PollFlags::IN);

            let mut fds = [pool_fd];
            rustix::event::poll(&mut fds, 0).ok();

            if fds[0].revents().contains(PollFlags::IN) {
                self.read().ok();
            }
        }

        self.has_input_events().then(|| {
            let offset = self.cell_id * CELL_SIZE;
            let event = unsafe { event::Event::read(&self.input_buffer[offset..]) };

            self.cell_id += 1;
            self.cell_count -= 1;

            if let event::EventData::Ext(ext) = event.data() {
                // TODO: Replace this c'ism with something easier to understand
                // This is basically snapping to nearest cell size
                let extra_cells = (ext.len() + CELL_SIZE - 1) / CELL_SIZE;

                if self.cell_count < extra_cells {
                    // err
                }

                self.cell_id += extra_cells;
                self.cell_count -= extra_cells;
            }

            event
        })
    }
}

impl AsRawFd for SeqInput {
    fn as_raw_fd(&self) -> RawFd {
        self.seq.as_raw_fd()
    }
}

impl AsFd for SeqInput {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        self.seq.as_fd()
    }
}
