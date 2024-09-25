use std::{
    io,
    net::{IpAddr, SocketAddr, ToSocketAddrs},
};

use lab4::net::Communicator;

fn main() -> io::Result<()> {
    let mcast_addr = "239.192.0.4:9192".to_socket_addrs()?[0];
    let comm = Communicator::new(mcast_addr)?;

    loop {
        todo!()
    }

    Ok(())
}
