use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_binary::binary_stream::Endian;
use socket2::Domain;
use socket2::Protocol;
use socket2::SockAddr;
use socket2::Socket;
use socket2::Type;

use crate::TransferComplete;

use super::TransferRequest;
use super::TransferResponse;

pub struct Client {
    pub(crate) socket: Socket,
}

impl Client {
    pub fn new() -> io::Result<Self> {
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
        Ok(Self { socket })
    }

    pub fn connect(&self, addr: SocketAddr) -> io::Result<()> {
        self.socket.connect(&SockAddr::from(addr))
    }

    fn send<T: Serialize>(&mut self, value: &T) -> io::Result<usize> {
        self.socket
            .write(&serde_binary::to_vec(value, Endian::Big).unwrap())
    }

    fn recv<T: DeserializeOwned>(&mut self) -> io::Result<T> {
        let mut buffer = [0u8; 8192];
        let read = self.socket.read(&mut buffer)?;
        Ok(serde_binary::from_slice(&buffer[..read], Endian::Big).unwrap())
    }

    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.socket.write(buffer)
    }

    pub fn transfer<P: AsRef<Path>>(&mut self, file: P) -> io::Result<()> {
        let mut out = File::open(file.as_ref())?;
        let len = out.metadata()?.len();

        let mut buffer = [0u8; 8192];

        self.send(&TransferRequest::new(
            file.as_ref().to_string_lossy().to_string(),
            len,
        ))?;

        let response: TransferResponse = self.recv()?;

        match response {
            TransferResponse::Success => {}
            TransferResponse::Failure => {
                eprintln!("received error from server");
                return Ok(());
            }
        }

        let mut bytes_sent = 0;

        while bytes_sent < len {
            let read = out.read(&mut buffer)?;
            let sent = self.write(&buffer[..read])?;

            bytes_sent += sent as u64;
        }

        let complete: TransferComplete = self.recv()?;

        println!("complete. bytes transfered: {}", complete.len);

        Ok(())
    }
}
