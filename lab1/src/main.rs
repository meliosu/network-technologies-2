use std::{
    collections::HashMap,
    env,
    mem::MaybeUninit,
    net::{IpAddr, SocketAddr},
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use uuid::Uuid;

const PORT: u16 = 7123;
const HEADER: [u8; 4] = [0xDC, 0xAF, 0xFF, 0x00];

fn main() {
    let addr = match env::args().nth(1) {
        Some(addr) => addr,
        None => {
            eprintln!("please provide group address");
            return;
        }
    };

    let addr: IpAddr = match addr.parse() {
        Ok(addr) => addr,
        Err(err) => {
            eprintln!("{addr} is not a valid ip address: {err}");
            return;
        }
    };

    if !addr.is_multicast() {
        eprintln!("{addr} is not a multicast address");
        return;
    }

    let domain = if addr.is_ipv4() {
        Domain::IPV4
    } else {
        Domain::IPV6
    };

    let socket = match Socket::new(domain, Type::DGRAM, Some(Protocol::UDP)) {
        Ok(socket) => socket,
        Err(err) => {
            eprintln!("error creating socket: {err}");
            return;
        }
    };

    match socket.reuse_address() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("error reusing socket address: {err}");
            return;
        }
    }

    match socket.bind(&SocketAddr::new(addr, PORT).into()) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("error binding socket: {err}");
            return;
        }
    }

    match addr {
        IpAddr::V4(ipv4) => match socket.join_multicast_v4(&ipv4, &ipv4) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("error joining multicast group: {err}");
                return;
            }
        },

        IpAddr::V6(ipv6) => match socket.join_multicast_v6(&ipv6, 0) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("error joining multicast group: {err}");
                return;
            }
        },
    }

    let uuid = Uuid::new_v4();

    let clients: Mutex<HashMap<Uuid, Instant>> = Mutex::new(HashMap::new());

    thread::scope(|s| {
        s.spawn(|| loop {
            thread::sleep(Duration::from_secs(1));

            let mut clients = clients.lock().unwrap();

            clients.retain(|_, time| time.elapsed() < Duration::from_secs(1));
        });

        s.spawn(|| loop {
            thread::sleep(Duration::from_millis(200));

            let sockaddr: SockAddr = SocketAddr::new(addr, PORT).into();

            socket
                .send_to(&[&HEADER[..], &uuid.as_bytes()[..]].concat(), &sockaddr)
                .unwrap();
        });

        s.spawn(|| {
            const PACKET_SIZE: usize = HEADER.len() + std::mem::size_of::<Uuid>();

            let mut buffer: [MaybeUninit<u8>; PACKET_SIZE] = [MaybeUninit::uninit(); PACKET_SIZE];

            loop {
                socket.recv(&mut buffer).unwrap();

                let buffer: &[u8; PACKET_SIZE] = unsafe { std::mem::transmute(&buffer) };

                let Some(peer_uuid) = buffer.strip_prefix(&HEADER) else {
                    eprintln!("wrong packet");
                    continue;
                };

                let Ok(peer_uuid) = peer_uuid.try_into() else {
                    eprintln!("not an uuid");
                    continue;
                };

                let peer_uuid = Uuid::from_bytes(peer_uuid);

                if peer_uuid == uuid {
                    eprintln!("self");
                    continue;
                }

                let now = Instant::now();

                let mut clients = clients.lock().unwrap();

                println!("+ {peer_uuid}");

                clients.insert(peer_uuid, now);
            }
        });
    });
}
