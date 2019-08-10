//! The IO worker contains the logic for
//! handling client connections.
//!
//! # Initial handling
//! Upon running, an IO worker task
//! begins "initial handling." All received
//! packets are forwarded to the `initialhandler` module,
//! which contains logic for handling the status ping
//! and login sequence.
//!
//! After initial handling has completed, the worker
//! notifies the server of the new client, sending it
//! a handle with which the server can send packets
//! to the client.

use super::ListenerToServerMessage;
use crate::config::Config;
use crate::io::initialhandler::InitialHandler;
use crate::io::ServerToHandleMessage;
use crate::PlayerCount;
use crossbeam::channel::Sender;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Receiver;

/// Represents a client.
struct Client {
    initial_handler: Option<InitialHandler>,

    stream: TcpStream,
    addr: SocketAddr,

    sender: Option<Sender<ServerToHandleMessage>>,
    receiver: Option<Receiver<ServerToHandleMessage>>,
}

/// Handles a connection.
pub async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    sender: Sender<ListenerToServerMessage>,
    player_count: Arc<PlayerCount>,
    config: Arc<Config>,
) {
    let mut client = Client {
        initial_handler: Some(InitialHandler::new(config, player_count)),

        stream,
        addr,

        sender: None,
        receiver: None,
    };
}
