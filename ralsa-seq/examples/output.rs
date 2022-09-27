use std::ffi::CString;

use alsa_ioctl::seq_ioctl::{PortCapability, PortType};
use ralsa_seq::event::{Event, EventKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (seq, _seq_input, mut seq_output) = ralsa_seq::Seq::open()?;

    let name = CString::new("output example")?;
    let capability = PortCapability::WRITE
        | PortCapability::SUBS_WRITE
        | PortCapability::READ
        | PortCapability::SUBS_READ;
    let kind = PortType::MIDI_GENERIC | PortType::APPLICATION;

    let _port = seq.create_simple_port(&name, capability, kind)?;

    seq_output.send(&Event::new(EventKind::Noteon))?;

    Ok(())
}
