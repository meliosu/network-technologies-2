use crate::types::Decode;
use crate::types::Encode;
use socket2::Socket;
use std::io;

pub trait SocketExt {
    fn read(&self, buffer: &mut [u8]) -> io::Result<usize>;
    fn write(&self, buffer: &[u8]) -> io::Result<usize>;
    fn send_packet<E: Encode>(&self, packet: &E) -> io::Result<()>;
    fn recv_packet<D: Decode>(&self) -> io::Result<D>;
}

impl SocketExt for socket2::Socket {
    fn read(&self, buffer: &mut [u8]) -> io::Result<usize> {
        self.recv(unsafe { std::mem::transmute(buffer) })
    }

    fn write(&self, buffer: &[u8]) -> io::Result<usize> {
        self.send(buffer)
    }

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
