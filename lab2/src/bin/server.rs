use std::thread;

use args::Args;
use clap::Parser;

use lab2::server::Server;

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let server = Server::new().unwrap();
    server.listen(args.port).unwrap();

    loop {
        let mut conn = server.accept().unwrap();

        thread::spawn(move || {
            conn.transfer().unwrap();
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
