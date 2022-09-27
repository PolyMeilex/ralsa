use std::{cell::RefCell, ffi::CStr, ops::Deref, rc::Rc};

use alsa::{
    seq::{Addr, PortCap, PortInfo, PortSubscribe, PortType},
    Direction,
};
use bitflags::bitflags;

bitflags! {
    /// An enum that is used to specify what kind of MIDI messages should
    /// be ignored when receiving messages.
    pub struct Ignore: u8 {
        const SYSEX        = 0b001;
        const TIME         = 0b010;
        const ACTIVE_SENSE = 0b100;
    }
}

pub fn snd_seq_connect_from(seq: &alsa::Seq, myport: i32, src: &Addr) {
    let subs = PortSubscribe::empty().unwrap();

    subs.set_sender(Addr {
        client: src.client,
        port: src.port,
    });
    subs.set_dest(Addr {
        client: seq.client_id().unwrap(),
        port: myport,
    });

    seq.subscribe_port(&subs).unwrap();
}

#[derive(Clone)]
pub struct Seq {
    inner: Rc<alsa::Seq>,
}

impl Seq {
    pub fn open(name: Option<&CStr>, dir: Option<Direction>, nonblock: bool) -> alsa::Result<Self> {
        let seq = alsa::Seq::open(name, dir, nonblock)?;

        Ok(Self {
            inner: Rc::new(seq),
        })
    }

    pub fn create_simple_port(
        &self,
        name: &CStr,
        caps: PortCap,
        kind: PortType,
    ) -> alsa::Result<Port> {
        let port = self.inner.create_simple_port(name, caps, kind)?;

        dbg!(port);
        Ok(Port {
            port,
            seq: self.clone(),
            subscriptions: Default::default(),
        })
    }

    pub fn create_port(&self, port: &PortInfo) -> alsa::Result<Port> {
        let port = self.inner.create_port(port).map(|_| port.get_port())?;

        Ok(Port {
            port,
            seq: self.clone(),
            subscriptions: Default::default(),
        })
    }
}

impl Deref for Seq {
    type Target = alsa::Seq;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct Port {
    port: i32,
    seq: Seq,
    subscriptions: RefCell<Vec<alsa::seq::PortSubscribe>>,
}

impl Port {
    pub fn id(&self) -> i32 {
        self.port
    }

    pub fn addr(&self) -> Addr {
        Addr {
            client: self.seq.client_id().unwrap(),
            port: self.port,
        }
    }

    pub fn connect_src(&self, src: &Addr) {
        let subs = PortSubscribe::empty().unwrap();

        subs.set_sender(Addr {
            client: src.client,
            port: src.port,
        });
        subs.set_dest(self.addr());

        self.seq.subscribe_port(&subs).unwrap();
        self.subscriptions.borrow_mut().push(subs);
    }

    pub fn connect_dest(&self, dest: &Addr) {
        let subs = PortSubscribe::empty().unwrap();

        subs.set_sender(self.addr());
        subs.set_dest(Addr {
            client: dest.client,
            port: dest.port,
        });

        self.seq.subscribe_port(&subs).unwrap();
        self.subscriptions.borrow_mut().push(subs);
    }

    pub fn subscribe(&self, sub: PortSubscribe) {
        self.seq.subscribe_port(&sub).unwrap();
        self.subscriptions.borrow_mut().push(sub);
    }
}

impl Deref for Port {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.port
    }
}

impl Drop for Port {
    fn drop(&mut self) {
        for sub in self.subscriptions.borrow().iter() {
            self.seq
                .unsubscribe_port(sub.get_sender(), sub.get_dest())
                .ok();
        }

        self.seq.delete_port(self.port).ok();
    }
}
