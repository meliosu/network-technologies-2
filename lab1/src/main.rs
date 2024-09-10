use std::{
    collections::HashMap,
    env, io,
    mem::{self, MaybeUninit},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

use anyhow::anyhow;

use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use uuid::Uuid;

const PORT: u16 = 7123;
const HEADER: [u8; 4] = [0xDC, 0xAF, 0xFF, 0x00];
const DELAY: Duration = Duration::from_secs(1);
const PACKET_SIZE: usize = HEADER.len() + mem::size_of::<Uuid>();

fn setup_socket(addr: IpAddr) -> anyhow::Result<Socket> {
    let domain = match addr {
        IpAddr::V4(_) => Domain::IPV4,
        IpAddr::V6(_) => Domain::IPV6,
    };

    let socket = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))
        .map_err(|err| anyhow!("creating socket: {err}"))?;

    socket
        .set_reuse_address(true)
        .map_err(|err| anyhow!("enabling address reuse for socket: {err}"))?;

    let multicast_addr: SockAddr = SocketAddr::new(addr, PORT).into();

    socket
        .bind(&multicast_addr)
        .map_err(|err| anyhow!("binding socket: {err}"))?;

    match addr {
        IpAddr::V4(ipv4) => {
            socket
                .join_multicast_v4(&ipv4, &Ipv4Addr::UNSPECIFIED)
                .map_err(|err| anyhow!("joining multicast group (ipv4): {err}"))?;
        }

        IpAddr::V6(ipv6) => {
            socket
                .join_multicast_v6(&ipv6, 0)
                .map_err(|err| anyhow!("joining multicast group (ipv6): {err}"))?;
        }
    }

    Ok(socket)
}

fn parse_args() -> anyhow::Result<IpAddr> {
    let addr = env::args().nth(1).ok_or(anyhow!("no address provided"))?;
    let addr: IpAddr = addr
        .parse()
        .map_err(|err| anyhow!("{addr} is not a valid ip address: {err}"))?;

    if !addr.is_multicast() {
        return Err(anyhow!("{addr} is not a multicast address"));
    }

    Ok(addr)
}

fn main() {
    let addr = match parse_args() {
        Ok(addr) => addr,
        Err(err) => {
            eprintln!("error parsing args: {err}");
            return;
        }
    };

    let socket = match setup_socket(addr) {
        Ok(socket) => socket,
        Err(err) => {
            eprintln!("error setting up socket: {err}");
            return;
        }
    };

    let uuid = Uuid::new_v4();

    let clients: Mutex<HashMap<Uuid, Instant>> = Mutex::new(HashMap::new());

    thread::scope(|s| {
        s.spawn(|| loop {
            thread::sleep(DELAY);

            let mut clients = clients.lock().unwrap();

            clients.retain(|uuid, time| {
                if time.elapsed() > DELAY {
                    println!("- {uuid}");
                    false
                } else {
                    true
                }
            });
        });

        s.spawn(|| loop {
            thread::sleep(DELAY / 4);

            let group_address: SockAddr = SocketAddr::new(addr, PORT).into();

            socket
                .send_to(
                    &[&HEADER[..], &uuid.as_bytes()[..]].concat(),
                    &group_address,
                )
                .unwrap();
        });

        s.spawn(|| {
            let mut buffer: [MaybeUninit<u8>; PACKET_SIZE] = [MaybeUninit::uninit(); PACKET_SIZE];

            loop {
                socket.recv(&mut buffer).unwrap();

                let buffer: &[u8; PACKET_SIZE] = unsafe { std::mem::transmute(&buffer) };

                let Some(peer_uuid) = buffer
                    .strip_prefix(&HEADER)
                    .and_then(|bytes| bytes.try_into().ok())
                    .map(|bytes| Uuid::from_bytes(bytes))
                    .filter(|peer_uuid| *peer_uuid != uuid)
                else {
                    continue;
                };

                let mut clients = clients.lock().unwrap();

                let now = Instant::now();

                if clients.insert(peer_uuid, now).is_none() {
                    println!("+ {peer_uuid}");
                }
            }
        });
    });
}
