use nix::libc::c_int;
use nix::{ioctl_read, ioctl_readwrite, ioctl_write_ptr};

use super::types;

// #define SNDRV_SEQ_IOCTL_PVERSION	_IOR ('S', 0x00, int)
ioctl_read! {
    pversion, b'S', 0x00, c_int
}

// #define SNDRV_SEQ_IOCTL_CLIENT_ID	_IOR ('S', 0x01, int)
ioctl_read! {
    client_id, b'S', 0x01, c_int
}

// #define SNDRV_SEQ_IOCTL_SYSTEM_INFO	_IOWR('S', 0x02, struct snd_seq_system_info)
ioctl_readwrite! {
    system_info, b'S', 0x02, types::SystemInfo
}

// #define SNDRV_SEQ_IOCTL_RUNNING_MODE	_IOWR('S', 0x03, struct snd_seq_running_info)
ioctl_readwrite! {
    running_mode, b'S', 0x03, types::RunningInfo
}

//
// Client
//

// #define SNDRV_SEQ_IOCTL_GET_CLIENT_INFO	_IOWR('S', 0x10, struct snd_seq_client_info)
ioctl_readwrite! {
    get_client_info, b'S', 0x10, types::ClientInfo
}

// #define SNDRV_SEQ_IOCTL_SET_CLIENT_INFO	_IOW ('S', 0x11, struct snd_seq_client_info)
ioctl_write_ptr! {
    set_client_info, b'S', 0x11, types::ClientInfo
}

//
// Port
//

// #define SNDRV_SEQ_IOCTL_CREATE_PORT	_IOWR('S', 0x20, struct snd_seq_port_info)
ioctl_readwrite! {
    create_port, b'S', 0x20, types::PortInfo
}

// #define SNDRV_SEQ_IOCTL_DELETE_PORT	_IOW ('S', 0x21, struct snd_seq_port_info)
ioctl_write_ptr! {
    delete_port, b'S', 0x21, types::PortInfo
}

// #define SNDRV_SEQ_IOCTL_GET_PORT_INFO	_IOWR('S', 0x22, struct snd_seq_port_info)
ioctl_readwrite! {
    get_port_info, b'S', 0x22, types::PortInfo
}

// #define SNDRV_SEQ_IOCTL_SET_PORT_INFO	_IOW ('S', 0x23, struct snd_seq_port_info)
ioctl_write_ptr! {
    set_port_info, b'S', 0x23, types::PortInfo
}

//
// Subscribe
//

// #define SNDRV_SEQ_IOCTL_SUBSCRIBE_PORT	_IOW ('S', 0x30, struct snd_seq_port_subscribe)
ioctl_write_ptr! {
    subscribe_port, b'S', 0x30, types::PortSubscribe
}

// #define SNDRV_SEQ_IOCTL_UNSUBSCRIBE_PORT _IOW ('S', 0x31, struct snd_seq_port_subscribe)
ioctl_write_ptr! {
    unsubscribe_port, b'S', 0x31, types::PortSubscribe
}

//
// Misc
//

// #define SNDRV_SEQ_IOCTL_CREATE_QUEUE	_IOWR('S', 0x32, struct snd_seq_queue_info)
ioctl_readwrite! {
    create_queue, b'S', 0x32, types::QueueInfo
}

// #define SNDRV_SEQ_IOCTL_DELETE_QUEUE	_IOW ('S', 0x33, struct snd_seq_queue_info)
ioctl_write_ptr! {
    delete_queue, b'S', 0x33, types::QueueInfo
}

// #define SNDRV_SEQ_IOCTL_GET_QUEUE_INFO	_IOWR('S', 0x34, struct snd_seq_queue_info)
ioctl_readwrite! {
    get_queue_info, b'S', 0x34, types::QueueInfo
}

// #define SNDRV_SEQ_IOCTL_SET_QUEUE_INFO	_IOWR('S', 0x35, struct snd_seq_queue_info)
ioctl_readwrite! {
    set_queue_info, b'S', 0x35, types::QueueInfo
}

// #define SNDRV_SEQ_IOCTL_GET_NAMED_QUEUE	_IOWR('S', 0x36, struct snd_seq_queue_info)
ioctl_readwrite! {
    get_named_queue, b'S', 0x36, types::QueueInfo
}

// #define SNDRV_SEQ_IOCTL_GET_QUEUE_STATUS _IOWR('S', 0x40, struct snd_seq_queue_status)
ioctl_readwrite! {
    get_queue_status, b'S', 0x40, types::QueueStatus
}

// #define SNDRV_SEQ_IOCTL_GET_QUEUE_TEMPO	_IOWR('S', 0x41, struct snd_seq_queue_tempo)
ioctl_readwrite! {
    get_queue_tempo, b'S', 0x41, types::QueueTempo
}

// #define SNDRV_SEQ_IOCTL_SET_QUEUE_TEMPO	_IOW ('S', 0x42, struct snd_seq_queue_tempo)
ioctl_write_ptr! {
    set_queue_tempo, b'S', 0x42, types::QueueTempo
}

// #define SNDRV_SEQ_IOCTL_GET_QUEUE_TIMER	_IOWR('S', 0x45, struct snd_seq_queue_timer)
ioctl_readwrite! {
    get_queue_timer, b'S', 0x45, types::QueueTimer
}

// #define SNDRV_SEQ_IOCTL_SET_QUEUE_TIMER	_IOW ('S', 0x46, struct snd_seq_queue_timer)
ioctl_write_ptr! {
    set_queue_timer, b'S', 0x46, types::QueueTimer
}

// #define SNDRV_SEQ_IOCTL_GET_QUEUE_CLIENT	_IOWR('S', 0x49, struct snd_seq_queue_client)
ioctl_readwrite! {
    get_queue_client, b'S', 0x49, types::QueueClient
}

// #define SNDRV_SEQ_IOCTL_SET_QUEUE_CLIENT	_IOW ('S', 0x4a, struct snd_seq_queue_client)
ioctl_write_ptr! {
    set_queue_client, b'S', 0x4a, types::QueueClient
}

// #define SNDRV_SEQ_IOCTL_GET_CLIENT_POOL	_IOWR('S', 0x4b, struct snd_seq_client_pool)
ioctl_readwrite! {
    get_client_pool, b'S', 0x4b, types::ClientPool
}

// #define SNDRV_SEQ_IOCTL_SET_CLIENT_POOL	_IOW ('S', 0x4c, struct snd_seq_client_pool)
ioctl_write_ptr! {
    set_client_pool, b'S', 0x4c, types::ClientPool
}

// #define SNDRV_SEQ_IOCTL_REMOVE_EVENTS	_IOW ('S', 0x4e, struct snd_seq_remove_events)
ioctl_write_ptr! {
    remove_events, b'S', 0x4e, types::RemoveEvents
}

// #define SNDRV_SEQ_IOCTL_QUERY_SUBS	_IOWR('S', 0x4f, struct snd_seq_query_subs)
ioctl_readwrite! {
    query_subs, b'S', 0x4f, types::QuerySubscribe
}

// #define SNDRV_SEQ_IOCTL_GET_SUBSCRIPTION	_IOWR('S', 0x50, struct snd_seq_port_subscribe)
ioctl_readwrite! {
    get_subscription, b'S', 0x50, types::PortSubscribe
}

// #define SNDRV_SEQ_IOCTL_QUERY_NEXT_CLIENT	_IOWR('S', 0x51, struct snd_seq_client_info)
ioctl_readwrite! {
    query_next_client, b'S', 0x51, types::ClientInfo
}

// #define SNDRV_SEQ_IOCTL_QUERY_NEXT_PORT	_IOWR('S', 0x52, struct snd_seq_port_info)
ioctl_readwrite! {
    query_next_port, b'S', 0x52, types::PortInfo
}
