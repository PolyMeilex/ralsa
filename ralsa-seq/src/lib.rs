use std::{
    ffi::CStr,
    io,
    os::{
        fd::AsFd,
        unix::prelude::{AsRawFd, OwnedFd, RawFd},
    },
    path::{Path, PathBuf},
    sync::Arc,
};

use alsa_ioctl::seq_ioctl::{self, Addr, PortCapability, PortType};
use rustix::fs::{open, Mode, OFlags};

// const SND_SEQ_OBUF_SIZE: usize = 16 * 1024; /* default size */
const SEQ_INPUT_BUF_SIZE: usize = 500; /* in event_size aligned */

const MSG_SIZE: usize = std::mem::size_of::<seq_ioctl::Event>();
const CELL_SIZE: usize = MSG_SIZE;

pub mod event;

mod input;
pub use input::SeqInput;

mod output;
pub use output::SeqOutput;

fn query_seq_path() -> io::Result<PathBuf> {
    let mut enumerator = udev::Enumerator::new()?;
    enumerator.match_subsystem("sound")?;
    enumerator.match_sysname("seq")?;

    enumerator
        .scan_devices()?
        .next()
        .and_then(|dev| dev.devnode().map(ToOwned::to_owned))
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
}

#[derive(Debug)]
struct SeqInner {
    fd: OwnedFd,
    client_id: u32,
}

#[derive(Debug, Clone)]
pub struct Seq {
    inner: Arc<SeqInner>,
}

impl Seq {
    pub fn open() -> io::Result<(Seq, SeqInput, SeqOutput)> {
        Self::open_path(query_seq_path()?)
    }

    pub fn open_path<P>(path: P) -> io::Result<(Seq, SeqInput, SeqOutput)>
    where
        P: AsRef<Path>,
    {
        let fd = open(
            path.as_ref(),
            OFlags::RDWR | OFlags::NONBLOCK,
            Mode::empty(),
        )?;

        let _version = seq_ioctl::pversion(&fd)?;
        let client_id = seq_ioctl::client_id(&fd)?;

        let seq = Seq {
            inner: Arc::new(SeqInner {
                fd,
                client_id: client_id.0 as u32,
            }),
        };

        for client in seq.clients_iter() {
            println!("client: {:?}", client.name);
            for port in seq.ports_iter(client.client as u8) {
                println!("port: {:?}", port.name);
            }
        }

        Ok((seq.clone(), SeqInput::new(seq.clone()), SeqOutput::new(seq)))
    }

    pub fn create_port(&self, mut info: seq_ioctl::PortInfo) -> io::Result<Port> {
        info.addr.client = self.inner.client_id as u8;

        seq_ioctl::create_port(self, &mut info)?;

        Ok(Port { addr: info.addr })
    }

    pub fn create_simple_port(
        &self,
        name: &CStr,
        capability: PortCapability,
        kind: PortType,
    ) -> io::Result<Port> {
        let mut info: seq_ioctl::PortInfo = unsafe { std::mem::zeroed() };

        for (src, dest) in name.to_bytes_with_nul().iter().zip(info.name.0.iter_mut()) {
            *dest = *src;
        }

        info.capability = capability;
        info.type_ = kind;
        info.midi_channels = 16;
        info.midi_voices = 64;
        info.synth_voices = 0;

        self.create_port(info)
    }

    pub fn delete_port(&self, port: &Port) -> io::Result<()> {
        let mut port_info: seq_ioctl::PortInfo = unsafe { std::mem::zeroed() };
        port_info.addr = port.addr;

        seq_ioctl::delete_port(self, port_info)?;

        Ok(())
    }

    pub fn clients_iter(&self) -> ClientIter {
        ClientIter::new(self.clone())
    }

    pub fn ports_iter(&self, client: u8) -> PortIter {
        PortIter::new(self.clone(), client)
    }
}

impl AsRawFd for Seq {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.fd.as_raw_fd()
    }
}

impl AsFd for Seq {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        self.inner.fd.as_fd()
    }
}

#[derive(Clone, Debug)]
pub struct Port {
    addr: Addr,
}

impl Port {
    pub fn connect_src(&self, seq: &Seq, src: Addr) -> io::Result<()> {
        let mut data: seq_ioctl::PortSubscribe = unsafe { std::mem::zeroed() };

        data.sender = src;
        data.dest = self.addr;

        seq_ioctl::subscribe_port(seq, data)?;

        Ok(())
    }

    pub fn connect_dest(&self, seq: &Seq, dest: Addr) -> io::Result<()> {
        let mut data: seq_ioctl::PortSubscribe = unsafe { std::mem::zeroed() };

        data.sender = self.addr;
        data.dest = dest;

        seq_ioctl::subscribe_port(seq, data)?;

        Ok(())
    }
}

pub struct ClientIter {
    seq: Seq,
    client_info: seq_ioctl::ClientInfo,
}

impl ClientIter {
    fn new(seq: Seq) -> Self {
        let mut client_info: seq_ioctl::ClientInfo = unsafe { std::mem::zeroed() };
        client_info.client = -1;
        Self { seq, client_info }
    }

    /// Copy free version of `Iterator::next`
    pub fn next_client(&mut self) -> Option<&seq_ioctl::ClientInfo> {
        seq_ioctl::query_next_client(&self.seq, &mut self.client_info).ok()?;
        Some(&self.client_info)
    }
}

impl Iterator for ClientIter {
    type Item = seq_ioctl::ClientInfo;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_client().copied()
    }
}

pub struct PortIter {
    seq: Seq,
    port_info: seq_ioctl::PortInfo,
}

impl PortIter {
    fn new(seq: Seq, client: u8) -> Self {
        let mut port_info: seq_ioctl::PortInfo = unsafe { std::mem::zeroed() };
        port_info.addr.client = client;
        port_info.addr.port = u8::MAX;
        Self { seq, port_info }
    }

    /// Copy free version of `Iterator::next`
    pub fn next_port(&mut self) -> Option<&seq_ioctl::PortInfo> {
        seq_ioctl::query_next_port(&self.seq, &mut self.port_info).ok()?;
        Some(&self.port_info)
    }
}

impl Iterator for PortIter {
    type Item = seq_ioctl::PortInfo;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_port().copied()
    }
}
