use std::{
    fs::File,
    io::{Read, Write},
};

use args::Args;
use clap::Parser;
use lab2::{TransferError, TransferRequest};
use serde_binary::binary_stream::Endian;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let mut file = File::open(&args.file).unwrap();
    let meta = file.metadata().unwrap();
    let len = meta.len();

    let mut socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
    socket.connect(&SockAddr::from(args.dest))?;

    let mut buffer = [0u8; 8192];

    let request = TransferRequest {
        len,
        name: args.file,
    };

    let request = serde_binary::to_vec(&request, Endian::Big).unwrap();

    socket.write(&request).unwrap();
    let n = socket.read(&mut buffer).unwrap();

    let response: Result<(), TransferError> =
        serde_binary::from_slice(&buffer[..n], Endian::Big).unwrap();

    match response {
        Ok(()) => {
            println!("ok!");
        }

        Err(_) => {
            println!("error!");
            return Ok(());
        }
    }

    let mut bytes_sent = 0;

    while bytes_sent < len {
        let n = file.read(&mut buffer).unwrap();
        socket.write(&buffer[0..n]).unwrap();
        bytes_sent += n as u64;
    }

    println!("finised");

    Ok(())
}

mod args {
    use std::{
        io,
        net::{SocketAddr, ToSocketAddrs},
    };

    #[derive(clap::Parser)]
    pub struct Args {
        #[arg(long, short)]
        pub file: String,

        #[arg(long, short, value_parser = parse_socket_addr)]
        pub dest: SocketAddr,
    }

    fn parse_socket_addr(addr: &str) -> io::Result<SocketAddr> {
        Ok(addr.to_socket_addrs()?.next().unwrap())
    }
}
