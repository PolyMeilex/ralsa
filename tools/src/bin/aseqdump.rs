use alsa::{
    seq::{Addr, ClientIter, EvNote, EventType, PortCap, PortIter, PortType},
    Direction, PollDescriptors,
};
use clap::Parser;
use nix::poll::PollFlags;
use ralsa::{Ignore, Seq};
use std::{ffi::CStr, ops::Deref, str::FromStr};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const CLIENT_NAME: &CStr = unsafe { std::mem::transmute("aseqdump\0") };

/// Show the events received at an ALSA sequencer port
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Prints a list of possible input ports.
    #[clap(short, long)]
    list: bool,

    /// Sets the sequencer ports from which events are received.
    #[clap(short, long, multiple_occurrences = true, multiple_values = true)]
    ports: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let seq = Seq::open(None, None, true)?;
    seq.set_client_name(CLIENT_NAME)?;

    if args.list {
        list_ports(&seq)?;
    } else if !args.ports.is_empty() {
        let simple_port = seq.create_simple_port(
            CLIENT_NAME,
            PortCap::WRITE | PortCap::SUBS_WRITE,
            PortType::MIDI_GENERIC | PortType::APPLICATION,
        )?;

        for port in args.ports {
            let src = Addr::from_str(&port)?;

            simple_port.connect_src(&src);
        }

        let poll_desc_info = (seq.deref(), Some(Direction::Capture));
        let mut poll_fds: Vec<_> = poll_desc_info
            .get()
            .unwrap()
            .into_iter()
            .map(|poolfd| {
                nix::poll::PollFd::new(poolfd.fd, PollFlags::from_bits(poolfd.events).unwrap())
            })
            .collect();

        let ignore_flags = Ignore::TIME | Ignore::ACTIVE_SENSE;

        println!("Source  Event                  Ch  Data");
        loop {
            nix::poll::poll(&mut poll_fds, -1)?;

            let mut seq_input = seq.input();

            while seq_input.event_input_pending(true)? > 0 {
                let event = seq_input.event_input()?;

                let ignore = match event.get_type() {
                    EventType::Qframe | EventType::Tick | EventType::Clock => {
                        ignore_flags.contains(Ignore::TIME)
                    }
                    EventType::Sensing => ignore_flags.contains(Ignore::ACTIVE_SENSE),
                    EventType::Sysex => {
                        let data = event.get_ext().unwrap();
                        dbg!(data);

                        ignore_flags.contains(Ignore::SYSEX)
                    }
                    _ => false,
                };

                if !ignore {
                    let src = event.get_source();
                    let ty = format!("{:?}", event.get_type());

                    print!("{:3}:{:<3} {:<22} ", src.client, src.port, ty,);

                    if let Some(note) = event.get_data::<EvNote>() {
                        print!(
                            "{:<3} note {} velocity {}",
                            note.channel, note.note, note.velocity
                        );
                    }

                    println!();
                }
            }
        }
    }

    Ok(())
}

fn list_ports(seq: &Seq) -> Result<()> {
    println!(" Port    Client name                      Port name");

    for client in ClientIter::new(seq) {
        let client_name = client.get_name()?;

        for port in PortIter::new(seq, client.get_client()).filter(|p| {
            p.get_capability()
                .contains(PortCap::READ | PortCap::SUBS_READ)
        }) {
            let client = port.get_client();
            let port_id = port.get_port();
            let port_name = port.get_name()?;

            println!(
                "{:3}:{:<3}  {:32} {}",
                client, port_id, client_name, port_name
            );
        }
    }

    Ok(())
}
