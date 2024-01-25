use std::io;

use async_io::Async;
use ralsa_seq::SeqInput;

pub struct MidiInputStream {
    seq: Async<SeqInput>,
}

impl MidiInputStream {
    pub fn new(seq: SeqInput) -> Self {
        Self {
            seq: Async::new(seq).unwrap(),
        }
    }

    pub async fn read(&mut self) -> io::Result<Option<ralsa_seq::event::Event<'_>>> {
        self.seq.readable().await?;
        let seq = unsafe { self.seq.get_mut() };
        let event = seq.input_event(true);
        Ok(event)
    }
}
