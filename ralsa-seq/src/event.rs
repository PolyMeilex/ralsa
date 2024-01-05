use std::{borrow::Cow, time::Duration};

use alsa_ioctl::seq_ioctl::QueueSkew;
pub use alsa_ioctl::seq_ioctl::{
    self, Addr, Connect, EvCtrl, EvExt, EvNote, EvQueueControl, EvResult, QueueId,
};

#[derive(Clone)]
pub struct Event<'a> {
    kind: EventKind,
    raw: Cow<'a, seq_ioctl::Event>,
    raw_extra: Cow<'a, [u8]>,
}

impl<'a> Event<'a> {
    pub fn new(kind: EventKind) -> Self {
        let mut raw: seq_ioctl::Event = unsafe { std::mem::zeroed() };
        raw.type_ = kind.into();
        raw.data = seq_ioctl::EventData {
            note: EvNote {
                channel: 0,
                note: 50,
                velocity: 255,
                off_velocity: 0,
                duration: 100,
            },
        };
        raw.source = Addr {
            client: 128,
            port: 0,
        };
        raw.dest = Addr {
            // client: 28,
            // port: 0,
            client: seq_ioctl::address::SUBSCRIBERS,
            port: seq_ioctl::address::UNKNOWN,
        };
        raw.queue = QueueId::DIRECT;

        Self {
            kind,
            raw: Cow::Owned(raw),
            raw_extra: Cow::Owned(Vec::new()),
        }
    }

    pub fn event_bytes(&self) -> &[u8] {
        let r = self.raw.as_ref();
        let ptr = r as *const _ as *const u8;
        unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<seq_ioctl::Event>()) }
    }

    #[allow(unused_unsafe)]
    pub unsafe fn read(buff: &'a [u8]) -> Event<'a> {
        assert!(std::mem::size_of::<seq_ioctl::Event>() <= buff.len());
        let raw: &seq_ioctl::Event = unsafe { &*(buff.as_ptr() as *const _) };

        let kind = EventKind::from(raw.type_);

        let ext_len = match kind {
            EventKind::Sysex
            | EventKind::Bounce
            | EventKind::UsrVar0
            | EventKind::UsrVar1
            | EventKind::UsrVar2
            | EventKind::UsrVar3
            | EventKind::UsrVar4 => {
                debug_assert!(raw.flags.is_lenght_fixed());

                let ext = unsafe { &raw.data.ext };
                ext.len as usize
            }
            _ => 0,
        };

        let offset = std::mem::size_of::<seq_ioctl::Event>();
        let raw_extra = &buff[offset..];
        let raw_extra = &raw_extra[..ext_len];

        assert_eq!(ext_len, raw_extra.len());

        Self {
            kind,
            raw: Cow::Borrowed(raw),
            raw_extra: Cow::Borrowed(raw_extra),
        }
    }

    pub fn kind(&self) -> &EventKind {
        &self.kind
    }

    pub fn tag(&self) -> i8 {
        self.raw.tag
    }

    pub fn queue(&self) -> QueueId {
        self.raw.queue
    }

    pub fn time(&self) -> EventTime {
        if self.raw.flags.is_time_tick() {
            let tick = unsafe { self.raw.time.tick };
            EventTime::Tick(tick)
        } else {
            let time = unsafe { &self.raw.time.time };
            EventTime::Time(Duration::new(time.tv_sec as u64, time.tv_nsec))
        }
    }

    pub fn source(&self) -> &Addr {
        &self.raw.source
    }

    pub fn destination(&self) -> &Addr {
        &self.raw.dest
    }

    pub fn is_priority_high(&self) -> bool {
        self.raw.flags.is_priority_high()
    }

    pub fn data(&self) -> EventData<'_> {
        match self.kind {
            // system messages
            // event data type = snd_seq_result
            EventKind::System | EventKind::Result => {
                let result = unsafe { &self.raw.data.result };
                EventData::Result(result)
            }

            // note messages (channel specific)
            // event data type = snd_seq_ev_note
            EventKind::Note | EventKind::Noteon | EventKind::Noteoff | EventKind::Keypress => {
                let note = unsafe { &self.raw.data.note };
                EventData::Note(note)
            }

            // control messages (channel specific)
            // event data type = snd_seq_ev_ctrl
            // synchronisation messages
            // event data type = snd_seq_ev_ctrl
            EventKind::Controller
            | EventKind::Pgmchange
            | EventKind::Chanpress
            | EventKind::Pitchbend
            | EventKind::Control14
            | EventKind::Nonregparam
            | EventKind::Regparam
            | EventKind::Songpos
            | EventKind::Songsel
            | EventKind::Qframe
            | EventKind::Timesign
            | EventKind::Keysign => {
                let control = unsafe { &self.raw.data.control };
                EventData::Control(control)
            }

            // timer messages
            // event data type = snd_seq_ev_queue_control
            EventKind::SyncPos | EventKind::Tick | EventKind::SetposTick => {
                let queue = unsafe { &self.raw.data.queue };
                let position = unsafe { queue.param.position };
                EventData::QueueControl {
                    queue: &queue.queue,
                    data: QueueControlEventData::Position(position),
                }
            }

            EventKind::SetposTime => {
                let queue = unsafe { &self.raw.data.queue };

                let value = unsafe {
                    Duration::new(
                        queue.param.time.time.tv_sec as u64,
                        queue.param.time.time.tv_nsec,
                    )
                };

                EventData::QueueControl {
                    queue: &queue.queue,
                    data: QueueControlEventData::Time(value),
                }
            }

            EventKind::Tempo => {
                let queue = unsafe { &self.raw.data.queue };
                let value = unsafe { queue.param.value };

                EventData::QueueControl {
                    queue: &queue.queue,
                    data: QueueControlEventData::Value(value),
                }
            }

            EventKind::QueueSkew => {
                let queue = unsafe { &self.raw.data.queue };
                let skew = unsafe { queue.param.skew };

                EventData::QueueControl {
                    queue: &queue.queue,
                    data: QueueControlEventData::Skew(skew),
                }
            }

            EventKind::Start | EventKind::Continue | EventKind::Stop | EventKind::Clock => {
                let queue = unsafe { &self.raw.data.queue };
                EventData::QueueControl {
                    queue: &queue.queue,
                    data: QueueControlEventData::None,
                }
            }

            // others
            // event data type = none
            EventKind::TuneRequest | EventKind::Reset | EventKind::Sensing | EventKind::None => {
                EventData::None
            }

            // system status messages (broadcast for subscribers)
            // event data type = snd_seq_addr
            EventKind::ClientStart
            | EventKind::ClientExit
            | EventKind::ClientChange
            | EventKind::PortStart
            | EventKind::PortExit
            | EventKind::PortChange => {
                let addr = unsafe { &self.raw.data.addr };
                EventData::Addr(addr)
            }

            // port connection changes
            // event data type = snd_seq_connect
            EventKind::PortSubscribed | EventKind::PortUnsubscribed => {
                let connect = unsafe { &self.raw.data.connect };
                EventData::Connect(connect)
            }

            // echo back, kernel private messages
            // event data type = any
            // user-defined events with fixed length
            // event data type = any
            EventKind::Echo
            | EventKind::Oss
            | EventKind::Usr0
            | EventKind::Usr1
            | EventKind::Usr2
            | EventKind::Usr3
            | EventKind::Usr4
            | EventKind::Usr5
            | EventKind::Usr6
            | EventKind::Usr7
            | EventKind::Usr8
            | EventKind::Usr9 => {
                let data = unsafe { &self.raw.data.raw8.d };
                EventData::Raw8(data)
            }

            // variable length events
            // event data type = snd_seq_ev_ext
            // (SNDRV_SEQ_EVENT_LENGTH_VARIABLE must be set)
            EventKind::Sysex
            | EventKind::Bounce
            | EventKind::UsrVar0
            | EventKind::UsrVar1
            | EventKind::UsrVar2
            | EventKind::UsrVar3
            | EventKind::UsrVar4 => EventData::Ext(&self.raw_extra),
        }
    }

    pub fn event_with_data(&self) -> EventWithData {
        match self.kind {
            // system messages
            // event data type = snd_seq_result
            EventKind::System => EventWithData::System(unsafe { self.raw.data.result }),
            EventKind::Result => EventWithData::Result(unsafe { self.raw.data.result }),

            // note messages (channel specific)
            // event data type = snd_seq_ev_note
            EventKind::Note => EventWithData::Note(unsafe { self.raw.data.note }),
            EventKind::Noteon => EventWithData::NoteOn(unsafe { self.raw.data.note }),
            EventKind::Noteoff => EventWithData::NoteOff(unsafe { self.raw.data.note }),
            EventKind::Keypress => EventWithData::KeyPress(unsafe { self.raw.data.note }),

            // control messages (channel specific)
            // event data type = snd_seq_ev_ctrl
            // synchronisation messages
            // event data type = snd_seq_ev_ctrl
            EventKind::Controller => EventWithData::Controller(unsafe { self.raw.data.control }),
            EventKind::Pgmchange => EventWithData::Pgmchange(unsafe { self.raw.data.control }),
            EventKind::Chanpress => EventWithData::Chanpress(unsafe { self.raw.data.control }),
            EventKind::Pitchbend => EventWithData::Pitchbend(unsafe { self.raw.data.control }),
            EventKind::Control14 => EventWithData::Control14(unsafe { self.raw.data.control }),
            EventKind::Nonregparam => EventWithData::Nonregparam(unsafe { self.raw.data.control }),
            EventKind::Regparam => EventWithData::Regparam(unsafe { self.raw.data.control }),
            EventKind::Songpos => EventWithData::Songpos(unsafe { self.raw.data.control }),
            EventKind::Songsel => EventWithData::Songsel(unsafe { self.raw.data.control }),
            EventKind::Qframe => EventWithData::Qframe(unsafe { self.raw.data.control }),
            EventKind::Timesign => EventWithData::Timesign(unsafe { self.raw.data.control }),
            EventKind::Keysign => EventWithData::Keysign(unsafe { self.raw.data.control }),

            // timer messages
            // event data type = snd_seq_ev_queue_control
            EventKind::SyncPos => EventWithData::SyncPos {
                queue: unsafe { self.raw.data.queue.queue },
                position: unsafe { self.raw.data.queue.param.position },
            },
            EventKind::Tick => EventWithData::Tick {
                queue: unsafe { self.raw.data.queue.queue },
                position: unsafe { self.raw.data.queue.param.position },
            },
            EventKind::SetposTick => EventWithData::SetposTick {
                queue: unsafe { self.raw.data.queue.queue },
                position: unsafe { self.raw.data.queue.param.position },
            },

            EventKind::SetposTime => {
                let queue = unsafe { &self.raw.data.queue };
                let value = unsafe {
                    Duration::new(
                        queue.param.time.time.tv_sec as u64,
                        queue.param.time.time.tv_nsec,
                    )
                };

                EventWithData::SetposTime {
                    queue: queue.queue,
                    position: value,
                }
            }

            EventKind::Tempo => {
                let queue = unsafe { &self.raw.data.queue };
                let value = unsafe { queue.param.value };

                EventWithData::Tempo {
                    queue: queue.queue,
                    value,
                }
            }

            EventKind::QueueSkew => {
                let queue = unsafe { &self.raw.data.queue };
                let skew = unsafe { queue.param.skew };

                EventWithData::QueueSkew {
                    queue: queue.queue,
                    skew,
                }
            }

            EventKind::Start => EventWithData::Start(unsafe { self.raw.data.queue.queue }),
            EventKind::Continue => EventWithData::Continue(unsafe { self.raw.data.queue.queue }),
            EventKind::Stop => EventWithData::Stop(unsafe { self.raw.data.queue.queue }),
            EventKind::Clock => EventWithData::Clock(unsafe { self.raw.data.queue.queue }),

            // others
            // event data type = none
            EventKind::TuneRequest => EventWithData::TuneRequest,
            EventKind::Reset => EventWithData::Reset,
            EventKind::Sensing => EventWithData::Sensing,
            EventKind::None => EventWithData::None,

            // system status messages (broadcast for subscribers)
            // event data type = snd_seq_addr
            EventKind::ClientStart => EventWithData::ClientStart(unsafe { self.raw.data.addr }),
            EventKind::ClientExit => EventWithData::ClientExit(unsafe { self.raw.data.addr }),
            EventKind::ClientChange => EventWithData::ClientChange(unsafe { self.raw.data.addr }),
            EventKind::PortStart => EventWithData::PortStart(unsafe { self.raw.data.addr }),
            EventKind::PortExit => EventWithData::PortExit(unsafe { self.raw.data.addr }),
            EventKind::PortChange => EventWithData::PortChange(unsafe { self.raw.data.addr }),

            // port connection changes
            // event data type = snd_seq_connect
            EventKind::PortSubscribed => {
                EventWithData::PortSubscribed(unsafe { self.raw.data.connect })
            }
            EventKind::PortUnsubscribed => {
                EventWithData::PortUnsubscribed(unsafe { self.raw.data.connect })
            }

            // echo back, kernel private messages
            // event data type = any
            // user-defined events with fixed length
            // event data type = any
            EventKind::Echo => EventWithData::Echo(unsafe { self.raw.data.raw8.d }),
            EventKind::Oss => EventWithData::Oss(unsafe { self.raw.data.raw8.d }),
            EventKind::Usr0 => EventWithData::Usr0(unsafe { self.raw.data.raw8.d }),
            EventKind::Usr1 => EventWithData::Usr1(unsafe { self.raw.data.raw8.d }),
            EventKind::Usr2 => EventWithData::Usr2(unsafe { self.raw.data.raw8.d }),
            EventKind::Usr3 => EventWithData::Usr3(unsafe { self.raw.data.raw8.d }),
            EventKind::Usr4 => EventWithData::Usr4(unsafe { self.raw.data.raw8.d }),
            EventKind::Usr5 => EventWithData::Usr5(unsafe { self.raw.data.raw8.d }),
            EventKind::Usr6 => EventWithData::Usr6(unsafe { self.raw.data.raw8.d }),
            EventKind::Usr7 => EventWithData::Usr7(unsafe { self.raw.data.raw8.d }),
            EventKind::Usr8 => EventWithData::Usr8(unsafe { self.raw.data.raw8.d }),
            EventKind::Usr9 => EventWithData::Usr9(unsafe { self.raw.data.raw8.d }),

            // variable length events
            // event data type = snd_seq_ev_ext
            // (SNDRV_SEQ_EVENT_LENGTH_VARIABLE must be set)
            EventKind::Sysex => EventWithData::Sysex(&self.raw_extra),
            EventKind::Bounce => EventWithData::Bounce(&self.raw_extra),
            EventKind::UsrVar0 => EventWithData::UsrVar0(&self.raw_extra),
            EventKind::UsrVar1 => EventWithData::UsrVar1(&self.raw_extra),
            EventKind::UsrVar2 => EventWithData::UsrVar2(&self.raw_extra),
            EventKind::UsrVar3 => EventWithData::UsrVar3(&self.raw_extra),
            EventKind::UsrVar4 => EventWithData::UsrVar4(&self.raw_extra),
        }
    }
}

impl<'a> std::fmt::Debug for Event<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Event")
            .field("flags", &self.raw.flags)
            .field("tag", &self.tag())
            .field("queue", &self.queue())
            .field("time", &self.time())
            .field("source", &self.source())
            .field("destination", &self.destination())
            .field("kind", &self.event_with_data())
            .finish()
    }
}

#[derive(Debug, Clone)]
pub enum QueueControlEventData {
    /// Affected value (e.g. tempo)
    Value(i32),
    /// Sync position
    Position(u32),
    Time(Duration),
    /// Queue skew values
    Skew(seq_ioctl::QueueSkew),
    None,
}

#[derive(Debug, Clone)]
pub enum EventTime {
    Time(Duration),
    Tick(u32),
}

#[derive(Debug, Clone)]
pub enum EventData<'a> {
    Result(&'a EvResult),
    Note(&'a EvNote),
    Control(&'a EvCtrl),
    QueueControl {
        queue: &'a QueueId,
        data: QueueControlEventData,
    },
    Addr(&'a Addr),
    Connect(&'a Connect),
    Raw8(&'a [u8; 12]),
    Ext(&'a [u8]),
    None,
}

/// sequencer event type
#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, num_derive::FromPrimitive)]
pub enum EventKind {
    /// system status; event data type = #snd_seq_result_t
    System = seq_ioctl::EventType::SYSTEM.0,
    /// returned result status; event data type = #snd_seq_result_t
    Result,

    /// note on and off with duration; event data type = #snd_seq_ev_note_t
    Note = seq_ioctl::EventType::NOTE.0,
    /// note on; event data type = #snd_seq_ev_note_t
    Noteon,
    /// note off; event data type = #snd_seq_ev_note_t
    Noteoff,
    /// key pressure change (aftertouch); event data type = #snd_seq_ev_note_t
    Keypress,

    /// controller; event data type = #snd_seq_ev_ctrl_t
    Controller = seq_ioctl::EventType::CONTROLLER.0,
    /// program change; event data type = #snd_seq_ev_ctrl_t
    Pgmchange,
    /// channel pressure; event data type = #snd_seq_ev_ctrl_t
    Chanpress,
    /// pitchwheel; event data type = #snd_seq_ev_ctrl_t; data is from -8192 to 8191)
    Pitchbend,
    /// 14 bit controller value; event data type = #snd_seq_ev_ctrl_t
    Control14,
    /// 14 bit NRPN;  event data type = #snd_seq_ev_ctrl_t
    Nonregparam,
    /// 14 bit RPN; event data type = #snd_seq_ev_ctrl_t
    Regparam,

    /// SPP with LSB and MSB values; event data type = #snd_seq_ev_ctrl_t
    Songpos = seq_ioctl::EventType::SONGPOS.0,
    /// Song Select with song ID number; event data type = #snd_seq_ev_ctrl_t
    Songsel,
    /// midi time code quarter frame; event data type = #snd_seq_ev_ctrl_t
    Qframe,
    /// SMF Time Signature event; event data type = #snd_seq_ev_ctrl_t
    Timesign,
    /// SMF Key Signature event; event data type = #snd_seq_ev_ctrl_t
    Keysign,

    /// MIDI Real Time Start message; event data type = #snd_seq_ev_queue_control_t
    Start = seq_ioctl::EventType::START.0,
    /// MIDI Real Time Continue message; event data type = #snd_seq_ev_queue_control_t
    Continue,
    /// MIDI Real Time Stop message; event data type = #snd_seq_ev_queue_control_t
    Stop,
    /// Set tick queue position; event data type = #snd_seq_ev_queue_control_t
    SetposTick,
    /// Set real-time queue position; event data type = #snd_seq_ev_queue_control_t
    SetposTime,
    /// (SMF) Tempo event; event data type = #snd_seq_ev_queue_control_t
    Tempo,
    /// MIDI Real Time Clock message; event data type = #snd_seq_ev_queue_control_t
    Clock,
    /// MIDI Real Time Tick message; event data type = #snd_seq_ev_queue_control_t
    Tick,
    /// Queue timer skew; event data type = #snd_seq_ev_queue_control_t
    QueueSkew,
    /// Sync position changed; event data type = #snd_seq_ev_queue_control_t
    SyncPos,

    /// Tune request; event data type = none
    TuneRequest = seq_ioctl::EventType::TUNE_REQUEST.0,
    /// Reset to power-on state; event data type = none
    Reset,
    /// Active sensing event; event data type = none
    Sensing,

    /// Echo-back event; event data type = any type
    Echo = seq_ioctl::EventType::ECHO.0,
    /// OSS emulation raw event; event data type = any type
    Oss,

    /// New client has connected; event data type = #snd_seq_addr_t
    ClientStart = seq_ioctl::EventType::CLIENT_START.0,
    /// Client has left the system; event data type = #snd_seq_addr_t
    ClientExit,
    /// Client status/info has changed; event data type = #snd_seq_addr_t
    ClientChange,
    /// New port was created; event data type = #snd_seq_addr_t
    PortStart,
    /// Port was deleted from system; event data type = #snd_seq_addr_t
    PortExit,
    /// Port status/info has changed; event data type = #snd_seq_addr_t
    PortChange,

    /// Ports connected; event data type = #snd_seq_connect_t
    PortSubscribed,
    /// Ports disconnected; event data type = #snd_seq_connect_t
    PortUnsubscribed,

    /*
        70-89:  synthesizer events - obsoleted
    */
    /// user-defined event; event data type = any (fixed size)
    Usr0 = seq_ioctl::EventType::USR0.0,
    /// user-defined event; event data type = any (fixed size)
    Usr1,
    /// user-defined event; event data type = any (fixed size)
    Usr2,
    /// user-defined event; event data type = any (fixed size)
    Usr3,
    /// user-defined event; event data type = any (fixed size)
    Usr4,
    /// user-defined event; event data type = any (fixed size)
    Usr5,
    /// user-defined event; event data type = any (fixed size)
    Usr6,
    /// user-defined event; event data type = any (fixed size)
    Usr7,
    /// user-defined event; event data type = any (fixed size)
    Usr8,
    /// user-defined event; event data type = any (fixed size)
    Usr9,

    /*
        100-118: instrument layer - obsoleted
        119-129: reserved
    */
    /// system exclusive data (variable length);  event data type = #snd_seq_ev_ext_t
    Sysex = seq_ioctl::EventType::SYSEX.0,
    /// error event;  event data type = #snd_seq_ev_ext_t
    Bounce,
    /* 132-134: reserved */
    /// reserved for user apps; event data type = #snd_seq_ev_ext_t
    UsrVar0 = seq_ioctl::EventType::USR_VAR0.0,
    /// reserved for user apps; event data type = #snd_seq_ev_ext_t
    UsrVar1,
    /// reserved for user apps; event data type = #snd_seq_ev_ext_t
    UsrVar2,
    /// reserved for user apps; event data type = #snd_seq_ev_ext_t
    UsrVar3,
    /// reserved for user apps; event data type = #snd_seq_ev_ext_t
    UsrVar4,

    /* 150-151: kernel events with quote - DO NOT use in user clients */
    /// NOP; ignored in any case"]
    None = seq_ioctl::EventType::NONE.0,
}

impl From<u8> for EventKind {
    fn from(val: u8) -> Self {
        num_traits::FromPrimitive::from_u8(val).unwrap_or(Self::None)
    }
}

impl From<seq_ioctl::EventType> for EventKind {
    fn from(val: seq_ioctl::EventType) -> Self {
        Self::from(val.0)
    }
}

impl From<EventKind> for seq_ioctl::EventType {
    fn from(v: EventKind) -> Self {
        Self(v as u8)
    }
}

/// sequencer event type
#[derive(Debug, Clone)]
pub enum EventWithData<'a> {
    /// system status; event data type = #snd_seq_result_t
    System(EvResult),
    /// returned result status; event data type = #snd_seq_result_t
    Result(EvResult),

    /// note on and off with duration; event data type = #snd_seq_ev_note_t
    Note(EvNote),
    /// note on; event data type = #snd_seq_ev_note_t
    NoteOn(EvNote),
    /// note off; event data type = #snd_seq_ev_note_t
    NoteOff(EvNote),
    /// key pressure change (aftertouch); event data type = #snd_seq_ev_note_t
    KeyPress(EvNote),

    /// controller; event data type = #snd_seq_ev_ctrl_t
    Controller(EvCtrl),
    /// program change; event data type = #snd_seq_ev_ctrl_t
    Pgmchange(EvCtrl),
    /// channel pressure; event data type = #snd_seq_ev_ctrl_t
    Chanpress(EvCtrl),
    /// pitchwheel; event data type = #snd_seq_ev_ctrl_t; data is from -8192 to 8191)
    Pitchbend(EvCtrl),
    /// 14 bit controller value; event data type = #snd_seq_ev_ctrl_t
    Control14(EvCtrl),
    /// 14 bit NRPN;  event data type = #snd_seq_ev_ctrl_t
    Nonregparam(EvCtrl),
    /// 14 bit RPN; event data type = #snd_seq_ev_ctrl_t
    Regparam(EvCtrl),

    /// SPP with LSB and MSB values; event data type = #snd_seq_ev_ctrl_t
    Songpos(EvCtrl),
    /// Song Select with song ID number; event data type = #snd_seq_ev_ctrl_t
    Songsel(EvCtrl),
    /// midi time code quarter frame; event data type = #snd_seq_ev_ctrl_t
    Qframe(EvCtrl),
    /// SMF Time Signature event; event data type = #snd_seq_ev_ctrl_t
    Timesign(EvCtrl),
    /// SMF Key Signature event; event data type = #snd_seq_ev_ctrl_t
    Keysign(EvCtrl),

    /// MIDI Real Time Start message; event data type = #snd_seq_ev_queue_control_t
    Start(QueueId),
    /// MIDI Real Time Continue message; event data type = #snd_seq_ev_queue_control_t
    Continue(QueueId),
    /// MIDI Real Time Stop message; event data type = #snd_seq_ev_queue_control_t
    Stop(QueueId),
    /// Set tick queue position; event data type = #snd_seq_ev_queue_control_t
    SetposTick { queue: QueueId, position: u32 },
    /// Set real-time queue position; event data type = #snd_seq_ev_queue_control_t
    SetposTime { queue: QueueId, position: Duration },
    /// (SMF) Tempo event; event data type = #snd_seq_ev_queue_control_t
    Tempo { queue: QueueId, value: i32 },
    /// MIDI Real Time Clock message; event data type = #snd_seq_ev_queue_control_t
    Clock(QueueId),
    /// MIDI Real Time Tick message; event data type = #snd_seq_ev_queue_control_t
    Tick { queue: QueueId, position: u32 },
    /// Queue timer skew; event data type = #snd_seq_ev_queue_control_t
    QueueSkew { queue: QueueId, skew: QueueSkew },
    /// Sync position changed; event data type = #snd_seq_ev_queue_control_t
    SyncPos { queue: QueueId, position: u32 },

    /// Tune request; event data type = none
    TuneRequest,
    /// Reset to power-on state; event data type = none
    Reset,
    /// Active sensing event; event data type = none
    Sensing,

    /// Echo-back event; event data type = any type
    Echo([u8; 12]),
    /// OSS emulation raw event; event data type = any type
    Oss([u8; 12]),

    /// New client has connected; event data type = #snd_seq_addr_t
    ClientStart(Addr),
    /// Client has left the system; event data type = #snd_seq_addr_t
    ClientExit(Addr),
    /// Client status/info has changed; event data type = #snd_seq_addr_t
    ClientChange(Addr),
    /// New port was created; event data type = #snd_seq_addr_t
    PortStart(Addr),
    /// Port was deleted from system; event data type = #snd_seq_addr_t
    PortExit(Addr),
    /// Port status/info has changed; event data type = #snd_seq_addr_t
    PortChange(Addr),

    /// Ports connected; event data type = #snd_seq_connect_t
    PortSubscribed(Connect),
    /// Ports disconnected; event data type = #snd_seq_connect_t
    PortUnsubscribed(Connect),

    /*
        70-89:  synthesizer events - obsoleted
    */
    /// user-defined event; event data type = any (fixed size)
    Usr0([u8; 12]),
    /// user-defined event; event data type = any (fixed size)
    Usr1([u8; 12]),
    /// user-defined event; event data type = any (fixed size)
    Usr2([u8; 12]),
    /// user-defined event; event data type = any (fixed size)
    Usr3([u8; 12]),
    /// user-defined event; event data type = any (fixed size)
    Usr4([u8; 12]),
    /// user-defined event; event data type = any (fixed size)
    Usr5([u8; 12]),
    /// user-defined event; event data type = any (fixed size)
    Usr6([u8; 12]),
    /// user-defined event; event data type = any (fixed size)
    Usr7([u8; 12]),
    /// user-defined event; event data type = any (fixed size)
    Usr8([u8; 12]),
    /// user-defined event; event data type = any (fixed size)
    Usr9([u8; 12]),

    /*
        100-118: instrument layer - obsoleted
        119-129: reserved
    */
    /// system exclusive data (variable length);  event data type = #snd_seq_ev_ext_t
    Sysex(&'a [u8]),
    /// error event;  event data type = #snd_seq_ev_ext_t
    Bounce(&'a [u8]),
    /* 132-134: reserved */
    /// reserved for user apps; event data type = #snd_seq_ev_ext_t
    UsrVar0(&'a [u8]),
    /// reserved for user apps; event data type = #snd_seq_ev_ext_t
    UsrVar1(&'a [u8]),
    /// reserved for user apps; event data type = #snd_seq_ev_ext_t
    UsrVar2(&'a [u8]),
    /// reserved for user apps; event data type = #snd_seq_ev_ext_t
    UsrVar3(&'a [u8]),
    /// reserved for user apps; event data type = #snd_seq_ev_ext_t
    UsrVar4(&'a [u8]),

    /* 150-151: kernel events with quote - DO NOT use in user clients */
    /// NOP; ignored in any case"]
    None,
}
