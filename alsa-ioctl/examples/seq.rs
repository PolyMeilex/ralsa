use rustix::{
    fs::{Mode, OFlags},
    io::read,
    param::page_size,
};

use alsa_ioctl::seq_ioctl::{self, Event, PortCapability, PortType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fd = rustix::fs::open("/dev/snd/seq", OFlags::RDWR, Mode::empty())?;

    let version = seq_ioctl::pversion(&fd)?;
    dbg!(version);

    let client_id = seq_ioctl::client_id(&fd)?;
    dbg!(client_id);

    let mut port: seq_ioctl::PortInfo = unsafe { std::mem::zeroed() };
    port.addr.client = client_id.0 as u8;
    port.capability = PortCapability::WRITE | PortCapability::SUBS_WRITE;
    port.type_ = PortType::MIDI_GENERIC | PortType::APPLICATION;

    seq_ioctl::create_port(&fd, &mut port)?;

    let page_size = page_size();

    loop {
        let mut buff = vec![0u8; page_size];

        let len = read(&fd, &mut buff)?;

        dbg!(len);

        let item = buff.as_ptr() as *const Event;

        let item = unsafe { &*item };

        dbg!(item.type_);
    }
}
