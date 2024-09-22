use std::env;

use lab5::Server;

fn main() {
    env_logger::init();

    let Some(port) = env::args().nth(1) else {
        panic!("Please specify port");
    };

    let Ok(port) = port.parse::<u16>() else {
        panic!("{port} is not a valid port");
    };

    let mut server = Server::new(port).unwrap_or_else(|err| {
        panic!("error creating server: {err}");
    });

    server.run().unwrap_or_else(|err| {
        panic!("error while running server: {err}");
    });
}
