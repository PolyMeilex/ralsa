use crate::{bitfield_unit::BitfieldUnit, string::AsciiString};
use bitflags::bitflags;
use std::os::raw::{c_char, c_int, c_uchar, c_uint, c_ushort, c_void};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SndTimerId {
    pub dev_class: c_int,
    pub dev_sclass: c_int,
    pub card: c_int,
    pub device: c_int,
    pub subdevice: c_int,
}

//
// definition of sequencer event types
//

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EventType(pub c_uchar);

impl EventType {
    /* system messages
     * event data type = #snd_seq_result
     */
    pub const SYSTEM: Self = Self(0);
    pub const RESULT: Self = Self(1);

    /* note messages (channel specific)
     * event data type = #snd_seq_ev_note
     */
    pub const NOTE: Self = Self(5);
    pub const NOTEON: Self = Self(6);
    pub const NOTEOFF: Self = Self(7);
    pub const KEYPRESS: Self = Self(8);

    /* control messages (channel specific)
     * event data type = #snd_seq_ev_ctrl
     */
    pub const CONTROLLER: Self = Self(10);
    pub const PGMCHANGE: Self = Self(11);
    pub const CHANPRESS: Self = Self(12);
    /// from -8192 to 8191 */
    pub const PITCHBEND: Self = Self(13);
    /// 14 bit controller value
    pub const CONTROL14: Self = Self(14);
    /// 14 bit NRPN address + 14 bit unsigned value
    pub const NONREGPARAM: Self = Self(15);
    /// 14 bit RPN address + 14 bit unsigned value
    pub const REGPARAM: Self = Self(16);

    /* synchronisation messages
     * event data type = #snd_seq_ev_ctrl
     */

    /// Song Position Pointer with LSB and MSB values
    pub const SONGPOS: Self = Self(20);
    /// Song Select with song ID number
    pub const SONGSEL: Self = Self(21);
    /// midi time code quarter frame
    pub const QFRAME: Self = Self(22);
    /// SMF Time Signature event
    pub const TIMESIGN: Self = Self(23);
    /// SMF Key Signature event
    pub const KEYSIGN: Self = Self(24);

    /* timer messages
     * event data type = snd_seq_ev_queue_control
     */
    pub const START: Self = Self(30);
    pub const CONTINUE: Self = Self(31);
    pub const STOP: Self = Self(32);
    pub const SETPOS_TICK: Self = Self(33);
    pub const SETPOS_TIME: Self = Self(34);
    pub const TEMPO: Self = Self(35);
    pub const CLOCK: Self = Self(36);
    pub const TICK: Self = Self(37);
    pub const QUEUE_SKEW: Self = Self(38);

    /* others
     * event data type = none
     */
    pub const TUNE_REQUEST: Self = Self(40);
    pub const RESET: Self = Self(41);
    pub const SENSING: Self = Self(42);

    /* echo back, kernel private messages
     * event data type = any type
     */
    pub const ECHO: Self = Self(50);
    pub const OSS: Self = Self(51);

    /* system status messages (broadcast for subscribers)
     * event data type = snd_seq_addr
     */
    pub const CLIENT_START: Self = Self(60);
    pub const CLIENT_EXIT: Self = Self(61);
    pub const CLIENT_CHANGE: Self = Self(62);
    pub const PORT_START: Self = Self(63);
    pub const PORT_EXIT: Self = Self(64);
    pub const PORT_CHANGE: Self = Self(65);

    /* port connection changes
     * event data type = snd_seq_connect
     */
    pub const PORT_SUBSCRIBED: Self = Self(66);
    pub const PORT_UNSUBSCRIBED: Self = Self(67);

    /* 70-89:  synthesizer events - obsoleted */

    /* user-defined events with fixed length
     * event data type = any
     */
    pub const USR0: Self = Self(90);
    pub const USR1: Self = Self(91);
    pub const USR2: Self = Self(92);
    pub const USR3: Self = Self(93);
    pub const USR4: Self = Self(94);
    pub const USR5: Self = Self(95);
    pub const USR6: Self = Self(96);
    pub const USR7: Self = Self(97);
    pub const USR8: Self = Self(98);
    pub const USR9: Self = Self(99);

    /* 100-118: instrument layer - obsoleted */
    /* 119-129: reserved */

    /* 130-139: variable length events
     * event data type = snd_seq_ev_ext
     * (SNDRV_SEQ_EVENT_LENGTH_VARIABLE must be set)
     */
    pub const SYSEX: Self = Self(130);
    pub const BOUNCE: Self = Self(131);
    /* 132-134: reserved */
    pub const USR_VAR0: Self = Self(135);
    pub const USR_VAR1: Self = Self(136);
    pub const USR_VAR2: Self = Self(137);
    pub const USR_VAR3: Self = Self(138);
    pub const USR_VAR4: Self = Self(139);

    /* 150-151: kernel events with quote - DO NOT use in user clients */
    pub const KERNEL_ERROR: Self = Self(150);
    /// obsolete
    pub const KERNEL_QUOTE: Self = Self(151);

    /* 152-191: reserved */

    /* 192-254: hardware specific events */

    /* 255: special event */
    pub const NONE: Self = Self(255);
}

/// Event address
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Addr {
    /// Client number: 0..255, 255 = broadcast to all clients
    pub client: c_uchar,
    /// Port within client: 0..255, 255 = broadcast to all ports
    pub port: c_uchar,
}

/// Port connection
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Connect {
    pub sender: Addr,
    pub dest: Addr,
}

pub mod address {
    use super::*;

    /// unknown source
    pub const UNKNOWN: c_uchar = 253;
    /// send event to all subscribed ports
    pub const SUBSCRIBERS: c_uchar = 254;
    /// send event to all queues/clients/ports/channels
    pub const BROADCAST: c_uchar = 255;
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct QueueId(pub c_uchar);

impl QueueId {
    /// direct dispatch
    pub const DIRECT: Self = Self(253);

    pub fn is_direct(&self) -> bool {
        self.0 == Self::DIRECT.0
    }
}

/// event mode flag - NOTE: only 8 bits available!
/// Used in `Event::flags`
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EventFlags(pub c_uchar);

impl EventFlags {
    /// timestamp in clock ticks
    pub const TIME_STAMP_TICK: u8 = 0;
    /// timestamp in real time
    pub const TIME_STAMP_REAL: u8 = 1 << 0;
    pub const TIME_STAMP_MASK: u8 = 1 << 0;

    /// absolute timestamp
    pub const TIME_MODE_ABS: u8 = 0 << 1;
    /// relative to current time
    pub const TIME_MODE_REL: u8 = 1 << 1;
    pub const TIME_MODE_MASK: u8 = 1 << 1;

    /// fixed event size
    pub const EVENT_LENGTH_FIXED: u8 = 0 << 2;
    /// variable event size
    pub const EVENT_LENGTH_VARIABLE: u8 = 1 << 2;
    /// variable event size - user memory space
    pub const EVENT_LENGTH_VARUSR: u8 = 2 << 2;
    pub const EVENT_LENGTH_MASK: u8 = 3 << 2;

    /// normal priority
    pub const PRIORITY_NORMAL: u8 = 0 << 4;
    /// event should be processed before others
    pub const PRIORITY_HIGH: u8 = 1 << 4;
    pub const PRIORITY_MASK: u8 = 1 << 4;

    // prior events
    pub const fn priority_type(&self) -> u8 {
        self.0 & Self::PRIORITY_MASK
    }

    /// should event be processed before others
    pub const fn is_priority_high(&self) -> bool {
        self.priority_type() == Self::PRIORITY_HIGH
    }

    // event length type
    pub const fn length_type(&self) -> u8 {
        self.0 & Self::EVENT_LENGTH_MASK
    }

    /// fixed event size
    pub const fn is_lenght_fixed(&self) -> bool {
        self.length_type() == Self::EVENT_LENGTH_FIXED
    }

    /// variable event size
    pub const fn is_lenght_variable(&self) -> bool {
        self.length_type() == Self::EVENT_LENGTH_VARIABLE
    }

    /// variable event size - user memory space
    pub const fn is_lenght_varusr(&self) -> bool {
        self.length_type() == Self::EVENT_LENGTH_VARUSR
    }

    // time-stamp type
    pub const fn timestamp_type(&self) -> u8 {
        self.0 & Self::TIME_STAMP_MASK
    }

    /// timestamp in clock ticks
    pub const fn is_time_tick(&self) -> bool {
        self.timestamp_type() == Self::TIME_STAMP_TICK
    }

    /// timestamp in real time
    pub const fn is_time_real(&self) -> bool {
        self.timestamp_type() == Self::TIME_STAMP_REAL
    }

    // time-mode type
    pub const fn timemode_type(&self) -> u8 {
        self.0 & Self::TIME_MODE_MASK
    }

    /// absolute timestamp
    pub const fn is_time_absolute(&self) -> bool {
        self.timemode_type() == Self::TIME_MODE_ABS
    }

    /// relative to current time
    pub const fn is_time_relative(&self) -> bool {
        self.timemode_type() == Self::TIME_MODE_REL
    }
}

/// Note event
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EvNote {
    pub channel: c_uchar,
    pub note: c_uchar,
    pub velocity: c_uchar,
    /// only for SNDRV_SEQ_EVENT_NOTE
    pub off_velocity: c_uchar,
    /// only for SNDRV_SEQ_EVENT_NOTE
    pub duration: c_uint,
}

/// Controller event
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EvCtrl {
    pub channel: c_uchar,
    unused1: c_uchar,
    unused2: c_uchar,
    unused3: c_uchar,
    pub param: c_uint,
    pub value: c_int,
}

/// Generic set of bytes (12x8 bit)
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EvRaw8 {
    /// 8 bit value
    pub d: [c_uchar; 12usize],
}

/// Generic set of integers (3x32 bit)
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EvRaw32 {
    /// 32 bit value
    pub d: [c_uint; 3usize],
}

/// External stored data
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct EvExt {
    /// length of data
    pub len: c_uint,
    /// pointer to data (note: maybe 64-bit)
    pub ptr: *mut c_void,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EvResult {
    /// processed event type
    pub event: c_int,
    pub result: c_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RealTime {
    /// seconds
    pub tv_sec: c_uint,
    /// nanoseconds
    pub tv_nsec: c_uint,
}

/// midi ticks
pub type TickTimeT = c_uint;

#[repr(C)]
#[derive(Copy, Clone)]
pub union Timestamp {
    pub tick: TickTimeT,
    pub time: RealTime,
}

/// Queue skew values
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct QueueSkew {
    pub value: c_uint,
    pub base: c_uint,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct EvQueueControl {
    /// affected queue
    pub queue: QueueId,
    pad: [c_uchar; 3usize],
    pub param: EvQueueControlParam,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union EvQueueControlParam {
    /// Affected value (e.g. tempo)
    pub value: c_int,
    pub time: Timestamp,
    /// sync position
    pub position: c_uint,
    /// Queue skew
    pub skew: QueueSkew,
    pub d32: [c_uint; 2usize],
    pub d8: [c_uchar; 8usize],
}

/// quoted event - inside the kernel only
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct EvQuote {
    /// original sender
    pub origin: Addr,
    /// optional data
    pub value: c_ushort,
    /// quoted event
    pub event: *mut Event,
}

/// sequencer event
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Event {
    /// event type
    pub type_: EventType,
    /// event flags
    pub flags: EventFlags,
    pub tag: c_char,
    /// schedule queue
    pub queue: QueueId,
    /// schedule time
    pub time: Timestamp,
    /// source address
    pub source: Addr,
    /// destination address
    pub dest: Addr,
    /// event data
    pub data: EventData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union EventData {
    pub note: EvNote,
    pub control: EvCtrl,
    pub raw8: EvRaw8,
    pub raw32: EvRaw32,
    pub ext: EvExt,
    pub queue: EvQueueControl,
    pub time: Timestamp,
    pub addr: Addr,
    pub connect: Connect,
    pub result: EvResult,
    pub quote: EvQuote,
}

/// bounce event - stored as variable size data
#[repr(C)]
#[derive(Copy, Clone)]
pub struct EventBounce {
    pub err: c_int,
    pub event: Event,
    /* external data follows here. */
}

/// system information
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SystemInfo {
    /// maximum queues count
    pub queues: c_int,
    /// maximum clients count
    pub clients: c_int,
    /// maximum ports per client
    pub ports: c_int,
    /// maximum channels per port
    pub channels: c_int,
    /// current clients
    pub cur_clients: c_int,
    /// current queues
    pub cur_queues: c_int,
    reserved: [c_char; 24usize],
}

/// system running information
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RunningInfo {
    /// client id
    pub client: c_uchar,
    /// 1 = big-endian
    pub big_endian: c_uchar,
    /// 4 = 32bit, 8 = 64bit
    pub cpu_mode: c_uchar,
    pad: c_uchar,
    reserved: [c_uchar; 12usize],
}

/// known client numbers
pub mod client_id {
    use super::*;

    pub const SYSTEM: c_uchar = 0;
    /* internal client numbers */
    /// midi through
    pub const DUMMY: c_uchar = 14;
    /// oss sequencer emulator
    pub const OSS: c_uchar = 15;
}

/// client types
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ClientType(pub c_int);

impl ClientType {
    pub const NO_CLIENT: Self = Self(0);
    pub const USER_CLIENT: Self = Self(1);
    pub const KERNEL_CLIENT: Self = Self(2);

    pub fn is_no_client(&self) -> bool {
        self == &Self::NO_CLIENT
    }

    pub fn is_user_client(&self) -> bool {
        self == &Self::USER_CLIENT
    }

    pub fn is_kernel_client(&self) -> bool {
        self == &Self::KERNEL_CLIENT
    }
}

bitflags! {
    /// event filter flags
    /// Used in `ClientInfo::filter`
    #[repr(transparent)]
    pub struct Filter: c_uint {
        /// accept broadcast messages
        const BROADCAST = 1<<0;
        /// accept multicast messages
        const MULTICAST = 1<<1;
        /// accept bounce event in error
        const BOUNCE = 1<<2;
        /// use event filter
        const USE_EVENT = 1<<31;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ClientInfo {
    /// client number to inquire
    pub client: c_int,
    /// client type
    pub type_: ClientType,
    /// client name
    pub name: AsciiString<64>,
    /// filter flags
    pub filter: Filter, // c_uint
    /// multicast filter bitmap
    pub multicast_filter: [c_uchar; 8usize],
    /// event filter bitmap
    pub event_filter: [c_uchar; 32usize],
    /// RO: number of ports
    pub num_ports: c_int,
    /// number of lost events
    pub event_lost: c_int,
    /// RO: card number[kernel]
    pub card: c_int,
    /// RO: pid[user]
    pub pid: c_int,
    reserved: [c_char; 56usize],
}

/// client pool size
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ClientPool {
    /// client number to inquire
    pub client: c_int,
    /// outgoing (write) pool size
    pub output_pool: c_int,
    /// incoming (read) pool size
    pub input_pool: c_int,
    /// minimum free pool size for select/blocking mode
    pub output_room: c_int,
    /// unused size
    pub output_free: c_int,
    /// unused size
    pub input_free: c_int,
    reserved: [c_char; 64usize],
}

bitflags! {
    /// Remove events by specified criteria
    /// Used in `RemoveEvents::remove_mode`
    #[repr(transparent)]
    pub struct RemoveMode: c_uint {
        /// Flush input queues
        const INPUT = 1<<0;
        /// Flush output queues
        const OUTPUT = 1<<1;
        /// Restrict by destination q:client:port
        const DEST = 1<<2;
        /// Restrict by channel
        const DEST_CHANNEL = 1<<3;
        /// Restrict to before time
        const TIME_BEFORE = 1<<4;
        /// Restrict to time or after
        const TIME_AFTER = 1<<5;
        /// Time is in ticks
        const TIME_TICK = 1<<6;
        /// Restrict to event type
        const EVENT_TYPE = 1<<7;
        /// Do not flush off events
        const IGNORE_OFF = 1<<8;
        /// Restrict to events with given tag
        const TAG_MATCH = 1<<9;
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct RemoveEvents {
    /// Flags that determine what gets removed
    pub remove_mode: RemoveMode,
    pub time: Timestamp,
    //// Queue for REMOVE_DEST
    pub queue: QueueId,
    /// Address for REMOVE_DEST
    pub dest: Addr,
    /// Channel for REMOVE_DEST
    pub channel: c_uchar,
    /// For REMOVE_EVENT_TYPE
    pub type_: c_int,
    /// Tag for REMOVE_TAG
    pub tag: c_char,
    reserved: [c_int; 10usize],
}

/// known port numbers
pub mod port {
    pub const SYSTEM_TIMER: u8 = 0;
    pub const SYSTEM_ANNOUNCE: u8 = 1;
}

bitflags! {
    /// port capabilities (32 bits)
    /// Used in `PortInfo::capability`
    #[repr(transparent)]
    pub struct PortCapability: c_uint {
        /// readable from this port
        const READ = 1<<0;
        /// writable to this port
        const WRITE = 1<<1;

        const SYNC_READ = 1<<2;
        const SYNC_WRITE = 1<<3;

        const DUPLEX = 1<<4;

        /// allow read subscription
        const SUBS_READ = 1<<5;
        /// allow write subscription
        const SUBS_WRITE = 1<<6;
        /// routing not allowed
        const NO_EXPORT = 1<<7;
    }
}

bitflags! {
    /// port type
    /// Used in `PortInfo::type`
    #[repr(transparent)]
    pub struct PortType: c_uint {
        /// hardware specific
        const SPECIFIC = 1<<0;
        /// generic MIDI device
        const MIDI_GENERIC = 1<<1;
        /// General MIDI compatible device
        const MIDI_GM = 1<<2;
        /// GS compatible device
        const MIDI_GS = 1<<3;
        /// XG compatible device
        const MIDI_XG = 1<<4;
        /// MT-32 compatible device
        const MIDI_MT32 = 1<<5;
        /// General MIDI 2 compatible device
        const MIDI_GM2 = 1<<6;

        /* other standards...*/

        /// Synth device (no MIDI compatible - direct wavetable)
        const SYNTH = 1<<10;
        /// Sampling device (support sample download)
        const DIRECT_SAMPLE = 1<<11;
        /// Sampling device (sample can be downloaded at any time)
        const SAMPLE = 1<<12;

        /*...*/

        /// driver for a hardware device
        const HARDWARE = 1<<16;
        /// implemented in software
        const SOFTWARE = 1<<17;
        /// generates sound
        const SYNTHESIZER = 1<<18;
        /// connects to other device(s)
        const PORT = 1<<19;
        /// application (sequencer/editor)
        const APPLICATION = 1<<20;
    }
}

bitflags! {
    /// misc. conditioning flags
    /// Used in `PortInfo::flags`
    #[repr(transparent)]
    pub struct PortFlags: c_uint {
        const GIVEN_PORT = 1<<0;
        const TIMESTAMP = 1<<1;
        const TIME_REAL = 1<<2;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PortInfo {
    /// client/port numbers
    pub addr: Addr,
    /// port name
    pub name: AsciiString<64>,
    /// port capability bits
    pub capability: PortCapability, // c_uint
    /// port type bits
    pub type_: PortType, // c_uint
    /// channels per MIDI port
    pub midi_channels: c_int,
    /// voices per MIDI port
    pub midi_voices: c_int,
    /// voices per SYNTH port
    pub synth_voices: c_int,
    /// R/O: subscribers for output (from this port)
    pub read_use: c_int,
    /// R/O: subscribers for input (to this port)
    pub write_use: c_int,
    /// reserved for kernel use (must be NULL)
    pub kernel: *mut c_void,
    /// misc. conditioning
    pub flags: PortFlags, // c_uint
    /// queue # for timestamping
    pub time_queue: QueueId,
    reserved: [c_char; 59usize],
}

bitflags! {
    /// queue flags
    /// Used in `QueueInfo::flags` (I'm not sure)
    #[repr(transparent)]
    pub struct QueueFlag: c_uint {
        /// sync enabled
        const SYNC = 1<<0;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct QueueInfo {
    /// queue id
    pub queue: QueueId,

    /*
     *  security settings, only owner of this queue can start/stop timer
     *  etc. if the queue is locked for other clients
     */
    /// client id for owner of the queue
    pub owner: c_int,
    _bitfield_align_1: [u8; 0],
    _bitfield_1: BitfieldUnit<[u8; 1usize]>,
    /// name of this queue
    pub name: AsciiString<64>,
    /// flags
    pub flags: QueueFlag, // c_uint
    reserved: [c_char; 60usize],
}

#[allow(clippy::useless_transmute)]
impl QueueInfo {
    /// timing queue locked for other queues
    #[inline]
    pub fn locked(&self) -> c_uint {
        unsafe { ::std::mem::transmute(self._bitfield_1.get(0usize, 1u8) as u32) }
    }

    /// timing queue locked for other queues
    #[inline]
    pub fn set_locked(&mut self, val: c_uint) {
        unsafe {
            let val: u32 = ::std::mem::transmute(val);
            self._bitfield_1.set(0usize, 1u8, val as u64)
        }
    }

    /// timing queue locked for other queues
    #[inline]
    pub fn new_bitfield_1(locked: c_uint) -> BitfieldUnit<[u8; 1usize]> {
        let mut __bindgen_bitfield_unit: BitfieldUnit<[u8; 1usize]> = Default::default();
        __bindgen_bitfield_unit.set(0usize, 1u8, {
            let locked: u32 = unsafe { ::std::mem::transmute(locked) };
            locked as u64
        });
        __bindgen_bitfield_unit
    }
}

/// queue info/status
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct QueueStatus {
    /// queue id
    pub queue: QueueId,
    /// read-only - queue size
    pub events: c_int,
    /// current tick
    pub tick: TickTimeT,
    /// current time
    pub time: RealTime,
    /// running state of queue
    pub running: c_int,
    /// various flags
    pub flags: c_int,
    reserved: [c_char; 64usize],
}

/// queue tempo
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct QueueTempo {
    /// sequencer queue
    pub queue: QueueId,
    /// current tempo, us/tick
    pub tempo: c_uint,
    /// time resolution, ticks/quarter
    pub ppq: c_int,
    /// queue skew
    pub skew_value: c_uint,
    /// queue skew base
    pub skew_base: c_uint,
    reserved: [c_char; 24usize],
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct QueueTimerType(pub c_int);

/// sequencer timer sources
impl QueueTimerType {
    /// ALSA timer
    pub const ALSA: Self = Self(0);
    /// Midi Clock (CLOCK event)
    pub const MIDI_CLOCK: Self = Self(1);
    /// Midi Timer Tick (TICK event)
    pub const MIDI_TICK: Self = Self(2);

    pub fn is_alsa(&self) -> bool {
        self == &Self::ALSA
    }

    pub fn is_midi_clock(&self) -> bool {
        self == &Self::MIDI_CLOCK
    }

    pub fn is_midi_tick(&self) -> bool {
        self == &Self::MIDI_TICK
    }
}

/// queue timer info
#[repr(C)]
#[derive(Copy, Clone)]
pub struct QueueTimer {
    /// sequencer queue
    pub queue: QueueId,
    /// source timer type
    pub type_: QueueTimerType,
    pub timer: QueueTimerUnion,
    reserved: [c_char; 64usize],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union QueueTimerUnion {
    pub alsa: QueueTimerAlsa,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct QueueTimerAlsa {
    /// ALSA's timer ID
    pub id: SndTimerId,
    /// resolution in Hz
    pub resolution: c_uint,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct QueueClient {
    /// sequencer queue
    pub queue: QueueId,
    /// sequencer client
    pub client: c_int,
    /// queue is used with this client
    /// (must be set for accepting events)
    pub used: c_int,
    reserved: [c_char; 64usize],
}

bitflags! {
    /// Used in `PortSubscribe::flags`
    /// Used in `QuerySubs::flags`
    #[repr(transparent)]
    pub struct SubscribeFlags: c_uint {
        /// exclusive connection
        const EXCLUSIVE = 1<<0;
        const TIMESTAMP = 1<<1;
        const TIME_REAL = 1<<2;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PortSubscribe {
    /// sender address
    pub sender: Addr,
    /// destination address
    pub dest: Addr,
    /// number of voices to be allocated (0 = don't care)
    pub voices: c_uint,
    /// modes
    pub flags: SubscribeFlags, // c_uint
    /// input time-stamp queue (optional)
    pub queue: QueueId,
    pad: [c_uchar; 3usize],
    reserved: [c_char; 64usize],
}

bitflags! {
    /// type of query subscription
    /// Used in `QuerySubs::type`
    #[repr(transparent)]
    pub struct QuerySubscribeType: c_int {
        const READ = 1<<0;
        const WRITE = 1<<1;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct QuerySubscribe {
    /// client/port id to be searched
    pub root: Addr,
    /// READ or WRITE
    pub type_: QuerySubscribeType, // c_int
    /// 0..N-1
    pub index: c_int,
    /// R/O: number of subscriptions on this port
    pub num_subs: c_int,
    /// R/O: result
    pub addr: Addr,
    /// R/O: result
    pub queue: QueueId,
    /// R/O: result
    pub flags: SubscribeFlags, // c_uint
    reserved: [c_char; 64usize],
}
