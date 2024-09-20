use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Instant;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_binary::binary_stream::Endian;
use socket2::Domain;
use socket2::Protocol;
use socket2::SockAddr;
use socket2::Socket;
use socket2::Type;

use super::TransferRequest;
use super::TransferResponse;

pub struct Server {
    pub(crate) socket: Socket,
}

pub struct Connection {
    pub socket: Socket,
    pub addr: SockAddr,
}

impl Connection {
    fn new(socket: Socket, addr: SockAddr) -> Self {
        Self { socket, addr }
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

    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.socket.read(buffer)
    }

    pub fn transfer(&mut self) -> io::Result<()> {
        let mut buffer = [0u8; 8192];

        let request: TransferRequest = self.recv()?;

        let mut out = match File::options()
            .write(true)
            .create(true)
            .open(PathBuf::from("uploads").join(&request.name))
        {
            Ok(file) => {
                self.send(&TransferResponse::Success)?;
                file
            }

            Err(_) => {
                self.send(&TransferResponse::Failure)?;
                return Ok(());
            }
        };

        let mut bytes_rcvd = 0;

        while bytes_rcvd < request.len {
            let rcvd = self.read(&mut buffer)?;
            out.write(&buffer[..rcvd])?;

            bytes_rcvd += rcvd as u64;
        }

        Ok(())
    }
}

impl Server {
    pub fn new() -> io::Result<Self> {
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
        Ok(Self { socket })
    }

    pub fn listen(&self, port: u16) -> io::Result<()> {
        self.socket.bind(&SockAddr::from(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            port,
        )))?;

        self.socket.listen(10)
    }

    pub fn accept(&self) -> io::Result<Connection> {
        let (sock, addr) = self.socket.accept()?;
        Ok(Connection::new(sock, addr))
    }
}
