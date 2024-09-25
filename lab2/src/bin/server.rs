use std::thread;

use args::Args;
use clap::Parser;

use lab2::{format_sockaddr, server::Server};

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let server = Server::new().unwrap_or_else(|err| {
        panic!("error creating server: {err}");
    });

    server.listen(args.port).unwrap_or_else(|err| {
        panic!("error listening: {err}");
    });

    loop {
        let mut conn = server.accept().unwrap_or_else(|err| {
            panic!("error accepting connections: {err}");
        });

        println!("new connection: {}", format_sockaddr(&conn.addr));

        thread::spawn(move || {
            if let Err(err) = conn.transfer() {
                eprintln!("error while transfering: {err}");
            }
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
