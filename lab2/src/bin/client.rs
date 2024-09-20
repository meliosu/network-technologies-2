use args::Args;
use clap::Parser;

use lab2::client::Client;

fn main() {
    let args = Args::parse();

    let mut client = Client::new().unwrap();

    client.connect(args.dest).unwrap();
    println!("Connected to server, transfering {}", args.file);

    client.transfer(&args.file).unwrap();
    println!("Transfer complete!");
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
