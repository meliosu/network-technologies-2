use std::env;

use lab5::Server;

fn main() {
    let Some(port) = env::args().nth(1) else {
        panic!("Please specify port");
    };

    let Ok(port) = port.parse::<u16>() else {
        panic!("{port} is not a valid port");
    };

    let _server = Server::new(port).unwrap_or_else(|err| {
        panic!("error creating server: {err}");
    });
}
