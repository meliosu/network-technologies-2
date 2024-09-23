use std::{
    io,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6, UdpSocket},
};

use tokio::net::UdpSocket;

pub struct Communicator {
    mcast: UdpSocket,
    ucast: UdpSocket,
    m_addr: SocketAddr,
}

impl Communicator {
    pub fn new(m_addr: SocketAddr) -> io::Result<Self> {
        let mcast = UdpSocket::bind(m_addr)?;

        match m_addr {
            SocketAddr::V4(ipv4) => {
                mcast.join_multicast_v4(ipv4.ip(), &Ipv4Addr::UNSPECIFIED)?;
            }

            SocketAddr::V6(ipv6) => {
                mcast.join_multicast_v6(ipv6.ip(), 0)?;
            }
        }

        let ucast = UdpSocket::bind("0.0.0.0:0")?;

        Ok(Self {
            mcast,
            ucast,
            m_addr,
        })
    }

    pub fn multicast<M: prost::Message>(&self, msg: M) -> io::Result<()> {
        self.ucast.send_to(&msg.encode_to_vec()[..], self.m_addr)?;
        Ok(())
    }

    pub fn unicast<M: prost::Message>(&self, msg: M, addr: SocketAddr) -> io::Result<()> {
        self.ucast.send_to(&msg.encode_to_vec()[..], addr)?;
        Ok(())
    }

    pub fn receive<M: prost::Message>(&self) -> io::Result<(M, SocketAddr)> {
        let mut buffer = [0u8; 4096];
        let (n, addr) = self.mcast.recv_from(&mut buffer)?;
        Ok((M::decode(&buffer[..n]), addr))
    }
}
