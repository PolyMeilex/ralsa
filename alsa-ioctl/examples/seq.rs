use nix::{
    fcntl::OFlag,
    sys::stat::Mode,
    unistd::{read, sysconf, SysconfVar},
};

use alsa_ioctl::seq_ioctl::{self, Event, PortCapability, PortType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fd = nix::fcntl::open("/dev/snd/seq", OFlag::O_RDWR, Mode::empty())?;

    let mut version = 0;
    unsafe {
        seq_ioctl::pversion(fd, &mut version)?;
    }

    dbg!(version);

    let mut client_id = 0;
    unsafe {
        seq_ioctl::client_id(fd, &mut client_id)?;
    }

    dbg!(client_id);

    let mut port: seq_ioctl::PortInfo = unsafe { std::mem::zeroed() };
    port.addr.client = client_id as u8;
    port.capability = PortCapability::WRITE | PortCapability::SUBS_WRITE;
    port.type_ = PortType::MIDI_GENERIC | PortType::APPLICATION;

    unsafe {
        seq_ioctl::create_port(fd, &mut port)?;
    }

    let page_size = sysconf(SysconfVar::PAGE_SIZE)?.unwrap() as usize;

    loop {
        let mut buff = vec![0u8; page_size];

        let len = read(fd, &mut buff)?;

        dbg!(len);

        let item = buff.as_ptr() as *const Event;

        let item = unsafe { &*item };

        dbg!(item.type_);
    }
}
