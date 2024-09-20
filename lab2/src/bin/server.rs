use std::io::Write;
use std::{fs::File, path::PathBuf, thread};

use args::Args;
use clap::Parser;
use lab2::{read, recv, send, Server, TransferRequest, TransferResponse};

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let server = Server::new().unwrap();
    server.listen(args.port).unwrap();

    loop {
        let (mut conn, _addr) = server.accept().unwrap();

        thread::spawn(move || {
            let mut buffer = [0u8; 8192];

            let request: TransferRequest = recv(&mut conn).unwrap();

            let mut file = match File::options()
                .write(true)
                .create(true)
                .open(PathBuf::from("uploads").join(&request.name))
            {
                Ok(file) => {
                    send(&mut conn, &TransferResponse::Success).unwrap();
                    file
                }

                Err(_) => {
                    send(&mut conn, &TransferResponse::Failure).unwrap();
                    return;
                }
            };

            let mut bytes_rcvd = 0;

            while bytes_rcvd < request.len {
                let rcvd = read(&mut conn, &mut buffer).unwrap();
                file.write(&buffer[..rcvd]).unwrap();

                bytes_rcvd += rcvd as u64;
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
