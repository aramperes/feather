use crate::config::Config;
use crate::PlayerCount;
use feather_core::network::packet::Packet;
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;
use uuid::Uuid;

mod initialhandler;
mod listener;
mod worker;

use crossbeam::channel as crossbeam_ch;
use tokio::sync::mpsc as tokio_ch;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Client(usize);

/// A message sent between the server and
/// a connection's handle task.
#[derive(Debug)]
pub enum ServerToHandleMessage {
    SendPacket(Box<dyn Packet>),
    NotifyPacketReceived(Box<dyn Packet>),
    NotifyDisconnect,
    Disconnect(String),
}

/// A message sent between the server
/// and the listener task.
pub enum ListenerToServerMessage {
    NewClient(NewClientInfo),
}

/// Struct representing metadata for a newly
/// connected client. This is sent to the server
/// after the initial handler has completed.
pub struct NewClientInfo {
    pub ip: SocketAddr,
    pub username: String,
    pub profile: Vec<mojang_api::ServerAuthProperty>,
    pub uuid: Uuid,

    /// The sender to communicate with the connection
    /// handler.
    pub sender: tokio_ch::Sender<ServerToHandleMessage>,
    /// The receiver to communicate with the connection
    /// handler.
    pub receiver: crossbeam_ch::Receiver<ServerToHandleMessage>,
}

/// A handle to the TCP listener for the server.
///
/// This handle includes channels to communicate
/// with the listener, allowing for graceful shutdown
/// or being notified of new clients.
///
/// The use of two different channel types (`crossbeam` and
/// `tokio::sync::mpsc` is due to the fact that `tokio` receivers
/// can only be polled inside the Tokio runtime. As a result,
/// the server's receiving channel needs to be blocking, while
/// the server's sending channel can use `tokio`.
#[derive(Debug)]
pub struct ListenerHandle {
    /// The sending channel used to communicate with the listener.
    pub sender: tokio_ch::Sender<ListenerToServerMessage>,
    /// The receiver, used to be notified of new clients.
    pub receiver: crossbeam_ch::Receiver<ServerToHandleMessage>,
}

impl ListenerHandle {}

/// Initializes certain static variables.
pub fn init() {
    lazy_static::initialize(&initialhandler::RSA_KEY);
}
