use alsa_ioctl::seq_ioctl::{PortCapability, PortType};
use nix::poll::PollFlags;
use std::{ffi::CString, os::unix::prelude::AsRawFd};

use ralsa_seq::event::EventWithData;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (seq, mut seq_input, _) = ralsa_seq::Seq::open()?;

    let name = CString::new("input example")?;
    let capability = PortCapability::WRITE
        | PortCapability::SUBS_WRITE
        | PortCapability::READ
        | PortCapability::SUBS_READ;
    let kind = PortType::MIDI_GENERIC | PortType::APPLICATION;
    let _port = seq.create_simple_port(&name, capability, kind)?;

    let fd = seq_input.as_raw_fd();

    let pool_fd = nix::poll::PollFd::new(fd, PollFlags::POLLIN);

    loop {
        nix::poll::poll(&mut [pool_fd], -1)?;

        while let Some(event) = seq_input.input_event(true) {
            // dbg!(&event);

            match event.event_with_data() {
                EventWithData::Qframe(_) | EventWithData::Tick { .. } | EventWithData::Clock(_) => {
                    // continue;
                }
                EventWithData::Sensing => {
                    // continue;
                }
                EventWithData::NoteOn(_note) => {}
                EventWithData::NoteOff(_note) => {}
                _ => {
                    dbg!(event);
                }
            }
        }
    }
}
