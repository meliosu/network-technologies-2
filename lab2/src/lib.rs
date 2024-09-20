use std::{
    fs::File,
    io::{self, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::Path,
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_binary::binary_stream::Endian;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

pub struct TransferComplete {
    pub len: u64,
}

#[derive(Serialize, Deserialize)]
pub struct TransferRequest {
    pub len: u64,
    pub name: String,
}

impl TransferRequest {
    pub fn new(name: String, len: u64) -> Self {
        Self { name, len }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_binary::to_vec(&self, Endian::Big).unwrap()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        serde_binary::from_slice(bytes, Endian::Big).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub enum TransferResponse {
    Success,
    Failure,
}

impl TransferResponse {
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_binary::to_vec(&self, Endian::Big).unwrap()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        serde_binary::from_slice(bytes, Endian::Big).unwrap()
    }
}

pub struct Client {
    socket: Socket,
}

impl Client {
    pub fn new() -> io::Result<Self> {
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
        Ok(Self { socket })
    }

    pub fn connect(&self, addr: SocketAddr) -> io::Result<()> {
        self.socket.connect(&SockAddr::from(addr))
    }

    pub fn transfer<P: AsRef<Path>>(&mut self, file: P) -> io::Result<()> {
        let mut out = File::open(file.as_ref())?;
        let len = out.metadata()?.len();

        let mut buffer = [0u8; 8192];

        let request = TransferRequest::new(file.as_ref().to_string_lossy().to_string(), len);
        self.socket.write(&request.to_bytes())?;

        let read = self.socket.read(&mut buffer)?;
        let response = TransferResponse::from_bytes(&buffer[..read]);

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
            let sent = self.socket.write(&buffer[..read])?;

            bytes_sent += sent as u64;
        }

        Ok(())
    }
}

pub struct Server {
    socket: Socket,
}

pub fn send<T: Serialize>(sock: &mut Socket, value: &T) -> io::Result<usize> {
    sock.write(&serde_binary::to_vec(value, Endian::Big).unwrap())
}

pub fn recv<T: DeserializeOwned>(sock: &mut Socket) -> io::Result<T> {
    let mut buffer = [0u8; 8192];
    let read = sock.read(&mut buffer)?;
    Ok(serde_binary::from_slice(&buffer[..read], Endian::Big).unwrap())
}

pub fn read(sock: &mut Socket, buffer: &mut [u8]) -> io::Result<usize> {
    sock.read(buffer)
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

    pub fn accept(&self) -> io::Result<(Socket, SockAddr)> {
        self.socket.accept()
    }
}
