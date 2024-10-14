use std::{
    io,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
};

use prost::Message;

use crate::proto::GameMessage;

pub struct Communicator {
    pub mcast: UdpSocket,
    pub ucast: UdpSocket,
}

impl Communicator {
    pub fn new<A>(multiaddr: A) -> io::Result<Self>
    where
        A: ToSocketAddrs,
    {
        let multiaddr = multiaddr.to_socket_addrs()?.next().unwrap();
        let mcast = UdpSocket::bind(multiaddr)?;
        mcast.connect(multiaddr)?;

        let ucast = UdpSocket::bind("0.0.0.0:0")?;

        Ok(Self { ucast, mcast })
    }

    pub fn send_mcast(&self, msg: &GameMessage) -> io::Result<()> {
        self.mcast.send(&msg.encode_to_vec()).map(|_| ())
    }

    pub fn send_ucast(&self, addr: SocketAddr, msg: &GameMessage) -> io::Result<()> {
        self.ucast.send_to(&msg.encode_to_vec(), addr).map(|_| ())
    }

    pub fn recv_mcast(&self) -> io::Result<(GameMessage, SocketAddr)> {
        let mut buffer = [0u8; 4096];
        let (n, addr) = self.mcast.recv_from(&mut buffer)?;
        Ok((GameMessage::decode(&buffer[..n])?, addr))
    }

    pub fn recv_ucast(&self) -> io::Result<(GameMessage, SocketAddr)> {
        let mut buffer = [0u8; 4096];
        let (n, addr) = self.ucast.recv_from(&mut buffer)?;
        Ok((GameMessage::decode(&buffer[..n])?, addr))
    }
}
