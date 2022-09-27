use alsa::seq::{Addr, ClientIter, EvNote, PortCap, PortIter, PortSubscribe, PortType};
use clap::Parser;
use ralsa::Seq;
use std::{borrow::Cow, ffi::CStr, str::FromStr, time::Duration};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const CLIENT_NAME: &CStr = unsafe { std::mem::transmute("aplaymidi\0") };

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
        let vport = seq.create_simple_port(
            CLIENT_NAME,
            PortCap::READ | PortCap::SUBS_READ,
            PortType::MIDI_GENERIC | PortType::APPLICATION,
        )?;

        for port in args.ports {
            let port_addr = Addr::from_str(&port)?;

            // Make subscription
            let sub = PortSubscribe::empty().unwrap();
            sub.set_sender(Addr {
                client: seq.client_id().unwrap(),
                port: vport.id(),
            });
            sub.set_dest(port_addr);
            sub.set_time_update(true);
            sub.set_time_real(true);

            vport.subscribe(sub);
        }

        for kind in [alsa::seq::EventType::Noteon, alsa::seq::EventType::Noteoff] {
            let mut event = alsa::seq::Event::new(
                kind,
                &EvNote {
                    channel: 0,
                    note: 50,
                    velocity: 55,
                    off_velocity: 0,
                    duration: 100,
                },
            );

            event.set_source(vport.id());
            event.set_subs();
            event.set_direct();

            seq.event_output(&mut event)?;

            seq.drain_output()?;

            std::thread::sleep(Duration::from_secs_f32(1.5));
        }

        {
            let data = vec![254; 29];
            let mut event =
                alsa::seq::Event::new_ext(alsa::seq::EventType::Sysex, Cow::Owned(data));

            event.set_source(vport.id());
            event.set_subs();
            event.set_direct();

            seq.event_output(&mut event)?;

            seq.drain_output()?;
        }
    }

    Ok(())
}

fn list_ports(seq: &Seq) -> Result<()> {
    println!(" Port    Client name                      Port name");

    for client in ClientIter::new(seq) {
        let client_name = client.get_name()?;

        for port in PortIter::new(seq, client.get_client())
            // port must understand MIDI messages
            .filter(|p| p.get_type().contains(PortType::MIDI_GENERIC))
            // we need both WRITE and SUBS_WRITE
            .filter(|p| {
                p.get_capability()
                    .contains(PortCap::WRITE | PortCap::SUBS_WRITE)
            })
        {
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
