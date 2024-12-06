use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
    sync::Arc,
};

use crate::proto::GameMessage;

const UNICAST_ADDR: &'static str = "192.168.237.84:0";

#[derive(Clone)]
pub struct Communicator {
    inner: Arc<comm::Communicator>,
}

impl Communicator {
    pub fn new<A>(multiaddr: A) -> io::Result<Self>
    where
        A: ToSocketAddrs,
    {
        Ok(Self {
            inner: Arc::new(comm::Communicator::new(multiaddr)?),
        })
    }

    pub fn send_mcast(&self, msg: &GameMessage) -> io::Result<()> {
        self.inner.send_mcast(msg)
    }

    pub fn send_ucast(&self, addr: SocketAddr, msg: &GameMessage) -> io::Result<()> {
        self.inner.send_ucast(addr, msg)
    }

    pub fn recv_mcast(&self) -> io::Result<(GameMessage, SocketAddr)> {
        self.inner.recv_mcast()
    }

    pub fn recv_ucast(&self) -> io::Result<(GameMessage, SocketAddr)> {
        self.inner.recv_ucast()
    }

    pub fn ucast_addr(&self) -> SocketAddr {
        self.inner.ucast_addr()
    }
}

mod comm {
    use std::{
        io,
        net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket},
        os::fd::{AsRawFd, FromRawFd},
    };

    use prost::Message;
    use socket2::{Domain, Protocol, SockAddr, Socket, Type};

    use crate::proto::GameMessage;

    use super::UNICAST_ADDR;

    pub struct Communicator {
        mcast: UdpSocket,
        ucast: UdpSocket,
        mcast_addr: SocketAddr,
    }

    impl Communicator {
        pub fn new<A>(multiaddr: A) -> io::Result<Self>
        where
            A: ToSocketAddrs,
        {
            let multiaddr = multiaddr.to_socket_addrs()?.next().unwrap();

            let IpAddr::V4(ipv4) = multiaddr.ip() else {
                panic!("not ipv4");
            };

            let mcast_addr = SockAddr::from(multiaddr);

            let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
            socket.set_reuse_address(true)?;
            socket.bind(&mcast_addr)?;
            socket.join_multicast_v4(&ipv4, &Ipv4Addr::UNSPECIFIED)?;

            let fd = socket.as_raw_fd();
            std::mem::forget(socket);

            let mcast = unsafe { UdpSocket::from_raw_fd(fd) };
            let ucast = UdpSocket::bind(UNICAST_ADDR)?;

            Ok(Self {
                ucast,
                mcast,
                mcast_addr: multiaddr,
            })
        }

        pub fn ucast_addr(&self) -> SocketAddr {
            self.ucast.local_addr().unwrap()
            //self.ucast.peer_addr().unwrap()
        }

        pub fn send_mcast(&self, msg: &GameMessage) -> io::Result<()> {
            self.ucast
                .send_to(&msg.encode_to_vec(), self.mcast_addr)
                .map(|_| ())
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
}
