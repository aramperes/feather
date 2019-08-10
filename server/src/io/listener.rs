use crate::io::ListenerToServerMessage;
use crossbeam::channel as mpmc;
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// Represents a listener.
pub struct Listener {
    tcp: TcpListener,
    sender: mpmc::Sender<ListenerToServerMessage>,
}

impl Listener {
    /// Creates a new listener, binding to the given
    /// address.
    pub fn new(
        addr: &SocketAddr,
        sender: mpmc::Sender<ListenerToServerMessage>,
    ) -> Result<Self, io::Error> {
        let tcp = TcpListener::bind(addr)?;
        Ok(Self { tcp, sender })
    }

    /// Runs the listener on the current task.
    pub async fn run(&mut self) -> Result<(), io::Error> {
        loop {
            let (stream, addr) = self.tcp.accept().await?;
            info!("Accepting connection from {}", addr);

            let sender = self.sender.clone();

            tokio::spawn(async move || {
                super::worker::handle_connection(stream, addr, sender).await;
            });
        }
    }
}
