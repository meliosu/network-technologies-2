use std::{
    fs::File,
    io::{self, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    thread,
};

use args::Args;
use clap::Parser;
use lab2::{TransferError, TransferRequest};
use serde_binary::binary_stream::Endian;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;

    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), args.port);

    socket.bind(&SockAddr::from(bind_addr))?;

    socket.listen(16)?;

    loop {
        let (mut conn, addr) = socket.accept()?;

        println!("accepted connection from {addr:?}");

        thread::spawn(move || -> io::Result<()> {
            let mut buffer = [0u8; 8192];

            let n = conn.read(&mut buffer).unwrap_or_else(|err| {
                panic!("error receiving request message: {err}");
            });

            let request: TransferRequest = serde_binary::from_slice(&buffer[..n], Endian::Big)
                .unwrap_or_else(|err| {
                    panic!("error parsing request message: {err}");
                });

            let mut out = match File::options()
                .write(true)
                .create(true)
                .open(PathBuf::from("uploads").join(&request.name))
            {
                Ok(file) => {
                    let ok =
                        serde_binary::to_vec::<Result<(), TransferError>>(&Ok(()), Endian::Big)
                            .unwrap();

                    conn.write(&ok)?;
                    file
                }

                Err(_) => {
                    let err = TransferError;

                    let err =
                        serde_binary::to_vec::<Result<(), TransferError>>(&Err(err), Endian::Big)
                            .unwrap();

                    conn.write(&err)?;
                    return Ok(());
                }
            };

            let mut bytes_received = 0;

            while bytes_received < request.len {
                let n = conn.read(&mut buffer).unwrap();
                out.write(&buffer[0..n]).unwrap();
                bytes_received += n as u64;
            }

            println!("finised");

            Ok(())
        });
    }
}

mod args {
    #[derive(clap::Parser)]
    pub struct Args {
        #[arg(long, short, default_value_t = 7123)]
        pub port: u16,
    }
}
