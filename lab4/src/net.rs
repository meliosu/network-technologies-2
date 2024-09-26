use std::{
    io,
    net::{Ipv4Addr, SocketAddr, UdpSocket},
};

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

    pub fn send_multicast<M: prost::Message>(&self, msg: M) -> io::Result<()> {
        self.ucast.send_to(&msg.encode_to_vec()[..], self.m_addr)?;
        Ok(())
    }

    pub fn send_unicast<M: prost::Message>(&self, msg: M, addr: SocketAddr) -> io::Result<()> {
        self.ucast.send_to(&msg.encode_to_vec()[..], addr)?;
        Ok(())
    }

    pub fn recv_multicast<M: prost::Message + Default>(&self) -> io::Result<(M, SocketAddr)> {
        let mut buffer = [0u8; 4096];
        let (n, addr) = self.mcast.recv_from(&mut buffer)?;
        Ok((M::decode(&buffer[..n])?, addr))
    }

    pub fn recv_unicast<M: prost::Message + Default>(&self) -> io::Result<(M, SocketAddr)> {
        let mut buffer = [0u8; 4096];
        let (n, addr) = self.ucast.recv_from(&mut buffer)?;
        Ok((M::decode(&buffer[..n])?, addr))
    }
}
