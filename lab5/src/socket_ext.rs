use std::io;

use socket2::Socket;

use crate::types::{Decode, Encode};

pub trait SocketExt {
    fn send_packet<E: Encode>(&self, packet: &E) -> io::Result<()>;
    fn recv_packet<D: Decode>(&self) -> io::Result<D>;
}

impl SocketExt for Socket {
    fn send_packet<E: Encode>(&self, packet: &E) -> io::Result<()> {
        let bytes = packet.to_bytes();

        if bytes.len() == self.send(&bytes)? {
            Ok(())
        } else {
            Err(io::Error::other("error sending"))
        }
    }

    fn recv_packet<D: Decode>(&self) -> io::Result<D> {
        let mut buffer = [0u8; 1024];
        let n = self.recv(unsafe { std::mem::transmute(&mut buffer[..]) })?;
        D::from_bytes(&buffer[..n]).ok_or(io::Error::other("error recving"))
    }
}
