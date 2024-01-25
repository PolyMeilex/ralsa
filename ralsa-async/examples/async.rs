use std::ffi::CString;

use alsa_ioctl::seq_ioctl::{PortCapability, PortType};

fn main() {
    async_io::block_on(async {
        let (seq, seq_input, seq_output) = ralsa_seq::Seq::open().unwrap();

        let name = CString::new("input example 😀").unwrap();
        let capability = PortCapability::WRITE
            | PortCapability::SUBS_WRITE
            | PortCapability::READ
            | PortCapability::SUBS_READ;
        let kind = PortType::MIDI_GENERIC | PortType::APPLICATION;
        let port = seq.create_simple_port(&name, capability, kind).unwrap();

        let mut input = ralsa_async::MidiInputStream::new(seq_input);

        loop {
            let event = input.read().await.unwrap();
            dbg!(event);
        }
    });
}
