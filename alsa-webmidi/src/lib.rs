use std::{
    collections::HashMap,
    ffi::CString,
    sync::{Arc, Mutex},
};

use alsa_ioctl::seq_ioctl::{Addr, PortCapability, PortInfo, PortType};
use ralsa_seq::{Port, Seq, SeqOutput};
use rustix::event::{PollFd, PollFlags};

type OpenInputs = Arc<Mutex<HashMap<Addr, Box<dyn FnMut(ralsa_seq::event::Event) + Send>>>>;

pub struct MIDIAccess {
    seq: Seq,
    seq_input: std::thread::JoinHandle<()>,
    _seq_output: SeqOutput,

    open_inputs: OpenInputs,

    port: Port,
}

impl Default for MIDIAccess {
    fn default() -> Self {
        Self::new()
    }
}

impl MIDIAccess {
    pub fn new() -> Self {
        let (seq, mut seq_input, seq_output) = ralsa_seq::Seq::open().unwrap();

        let name = CString::new("input example").unwrap();
        let capability = PortCapability::WRITE
            | PortCapability::SUBS_WRITE
            | PortCapability::READ
            | PortCapability::SUBS_READ;
        let kind = PortType::MIDI_GENERIC | PortType::APPLICATION;
        let port = seq.create_simple_port(&name, capability, kind).unwrap();

        let open_inputs: OpenInputs = Default::default();

        let arc = open_inputs.clone();
        let seq_input = std::thread::spawn(move || {
            let fd = seq_input.seq().clone();
            let mut pool_fd = [PollFd::new(&fd, PollFlags::IN)];

            loop {
                rustix::event::poll(&mut pool_fd, -1).unwrap();

                while let Some(event) = seq_input.input_event(true) {
                    if let Some(cb) = arc.lock().unwrap().get_mut(event.source()) {
                        (cb)(event);
                    }
                }
            }
        });

        Self {
            seq,
            seq_input,
            _seq_output: seq_output,
            open_inputs,

            port,
        }
    }

    pub fn inputs(&self) -> Vec<MIDIInput> {
        self.seq
            .clients_iter()
            .flat_map(|client| {
                self.seq
                    .ports_iter(client.client as u8)
                    // port must understand MIDI messages
                    .filter(|port| port.type_.contains(PortType::MIDI_GENERIC))
                    // we need both READ and SUBS_READ
                    .filter(|port| {
                        port.capability
                            .contains(PortCapability::READ | PortCapability::SUBS_READ)
                    })
            })
            .map(|port| {
                MIDIInput::new(
                    self.seq.clone(),
                    self.port.clone(),
                    port.addr,
                    self.open_inputs.clone(),
                )
            })
            .collect()
    }

    pub fn outputs(&self) -> Vec<PortInfo> {
        self.seq
            .clients_iter()
            .flat_map(|client| {
                self.seq
                    .ports_iter(client.client as u8)
                    // port must understand MIDI messages
                    .filter(|port| port.type_.contains(PortType::MIDI_GENERIC))
                    // we need both WRITE and SUBS_WRITE
                    .filter(|port| {
                        port.capability
                            .contains(PortCapability::WRITE | PortCapability::SUBS_WRITE)
                    })
            })
            .collect()
    }

    pub fn run(self) {
        self.seq_input.join().unwrap();
    }
}

pub struct MIDIInput {
    seq: Seq,
    port: Port,
    addr: Addr,

    open_inputs: OpenInputs,
}

impl MIDIInput {
    fn new(seq: Seq, port: Port, addr: Addr, open_inputs: OpenInputs) -> Self {
        Self {
            seq,
            port,
            addr,
            open_inputs,
        }
    }

    pub fn open<F>(&self, cb: F)
    where
        F: FnMut(ralsa_seq::event::Event) + Send + 'static,
    {
        self.port.connect_src(&self.seq, self.addr).unwrap();

        self.open_inputs
            .lock()
            .unwrap()
            .insert(self.addr, Box::new(cb));
    }
}
