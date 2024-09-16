use std::{
    collections::HashMap,
    env,
    mem::{self, MaybeUninit},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

use anyhow::{anyhow, ensure};
use colored::Colorize;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use uuid::Uuid;

const PORT: u16 = 7123;
const HEADER: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
const DELAY: Duration = Duration::from_secs(1);
const PACKET_SIZE: usize = HEADER.len() + mem::size_of::<Uuid>();

struct PeerInfo {
    last_packet: Instant,
    addr: SockAddr,
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

    let clients: Mutex<HashMap<Uuid, PeerInfo>> = Mutex::new(HashMap::new());

    let my_uuid = Uuid::new_v4();

    println!("+++ Starting client with uuid {my_uuid} +++");

    thread::scope(|s| {
        // Reaper Thread
        s.spawn(|| loop {
            thread::sleep(DELAY);

            let mut clients = clients.lock().unwrap();

            clients.retain(|uuid, info| {
                if info.last_packet.elapsed() > DELAY {
                    let info = format!("- {uuid} [{}]", format_sockaddr(&info.addr));
                    println!("{}", info.red());

                    false
                } else {
                    true
                }
            });
        });

        // Multicast Thread
        s.spawn(|| loop {
            thread::sleep(DELAY / 4);

            let packet = [&HEADER[..], my_uuid.as_bytes()].concat();
            let group_address: SockAddr = SocketAddr::new(addr, PORT).into();

            socket
                .send_to(&packet, &group_address)
                .unwrap_or_else(|err| {
                    panic!("error sending message on a socket: {err}");
                });
        });

        // Receiving Thread
        s.spawn(|| {
            let mut buffer: [MaybeUninit<u8>; PACKET_SIZE] = [MaybeUninit::uninit(); PACKET_SIZE];

            loop {
                let (_, from) = socket.recv_from(&mut buffer).unwrap_or_else(|err| {
                    panic!("error receiving message on a socket: {err}");
                });

                let buffer: &[u8; PACKET_SIZE] = unsafe { std::mem::transmute(&buffer) };

                let Some(peer_uuid) = buffer
                    .strip_prefix(&HEADER)
                    .and_then(|bytes| bytes.try_into().ok())
                    .map(|bytes| Uuid::from_bytes(bytes))
                    .filter(|peer_uuid| *peer_uuid != my_uuid)
                else {
                    continue;
                };

                let mut clients = clients.lock().unwrap();

                let address = format_sockaddr(&from);

                let peer = PeerInfo {
                    last_packet: Instant::now(),
                    addr: from,
                };

                if clients.insert(peer_uuid, peer).is_none() {
                    let info = format!("+ {peer_uuid} [{}]", address);
                    println!("{}", info.green());
                }
            }
        });
    });
}

fn format_sockaddr(addr: &SockAddr) -> String {
    if let Some(ipv4) = addr.as_socket_ipv4() {
        format!("{ipv4}")
    } else if let Some(ipv6) = addr.as_socket_ipv6() {
        format!("{ipv6}")
    } else {
        format!("UNKNOWN")
    }
}

fn setup_socket(addr: IpAddr) -> anyhow::Result<Socket> {
    ensure!(addr.is_multicast(), "{addr} is not a multicast address");

    let domain = match addr {
        IpAddr::V4(_) => Domain::IPV4,
        IpAddr::V6(_) => Domain::IPV6,
    };

    let socket = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))
        .map_err(|err| anyhow!("creating socket: {err}"))?;

    socket
        .set_reuse_address(true)
        .map_err(|err| anyhow!("enabling address reuse for socket: {err}"))?;

    let bind_addr: SockAddr = if cfg!(target_os = "windows") {
        let addr = match addr {
            IpAddr::V4(_) => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            IpAddr::V6(_) => IpAddr::V6(Ipv6Addr::UNSPECIFIED),
        };

        SocketAddr::new(addr, PORT).into()
    } else {
        SocketAddr::new(addr, PORT).into()
    };

    socket
        .bind(&bind_addr)
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

    addr.parse()
        .map_err(|err| anyhow!("{addr} is not an ip address: {err}"))
}
