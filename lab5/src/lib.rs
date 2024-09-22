#![allow(dead_code)]

use std::{
    collections::HashMap,
    io::{self},
    net::{Ipv4Addr, SocketAddrV4},
    os::fd::{AsFd, AsRawFd},
};

use nix::poll::{PollFd, PollFlags, PollTimeout};
use socket2::{Domain, Protocol, Socket, Type};
use types::{Address, ConnectionRequest, ConnectionResponse, GreetingRequest, GreetingResponse};

pub mod types;

pub struct Server {
    this: Socket,
    dns: Socket,
    queries: Vec<String>,
    answers: Vec<(String, Ipv4Addr)>,
    clients: HashMap<i32, Client>,
    buffer: Vec<u8>,
}

impl Server {
    pub fn new(port: u16) -> io::Result<Self> {
        let this = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
        this.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port).into())?;
        this.listen(16)?;
        this.set_nonblocking(true)?;

        let dns = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        dns.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0).into())?;
        dns.connect(&SocketAddrV4::new(Ipv4Addr::new(8, 8, 8, 8), 53).into())?;
        dns.set_nonblocking(true)?;

        Ok(Self {
            this,
            dns,
            queries: Vec::new(),
            answers: Vec::new(),
            clients: HashMap::new(),
            buffer: vec![0u8; 16384],
        })
    }

    // TODO: refactor
    fn recv_dns(&mut self) -> io::Result<()> {
        let received = self.dns.read(&mut self.buffer)?;
        let message = rustdns::Message::from_slice(&self.buffer[..received])?;
        let rustdns::Message { answers, .. } = message;
        self.answers
            .extend(answers.into_iter().filter_map(|a| match a.resource {
                rustdns::Resource::A(ip) => {
                    Some((a.name.strip_suffix(".").unwrap().to_string(), ip))
                }
                _ => None,
            }));

        Ok(())
    }

    fn send_dns(&mut self, domain: &str) -> io::Result<()> {
        let mut message = rustdns::Message::default();
        message.add_question(domain, rustdns::Type::A, rustdns::Class::Internet);
        self.dns.write(&message.to_vec()?).map(|_| ())
    }

    fn transition_client(
        &mut self,
        client: Client,
        revents: PollFlags,
    ) -> io::Result<Option<Client>> {
        let state = match client.state {
            ClientState::AwaitingGreeting if revents.contains(PollFlags::POLLIN) => {
                let n = client.socket.read(&mut self.buffer)?;

                if n != 0 {
                    let Some(greeting_request) = GreetingRequest::from_bytes(&self.buffer[..n])
                    else {
                        return Ok(None);
                    };

                    Some(ClientState::AwaitingGreetingResponse {
                        request: greeting_request,
                    })
                } else {
                    eprintln!("read 0 bytes while in greeting");
                    None
                }
            }

            ClientState::AwaitingGreetingResponse { request }
                if revents.contains(PollFlags::POLLOUT) =>
            {
                if request.auths.contains(&0x0) {
                    let greeting_response = GreetingResponse {
                        version: 0x5,
                        auth: 0x0,
                    };

                    client.socket.write(&greeting_response.to_bytes())?;

                    Some(ClientState::AwaitingConnection)
                } else {
                    let greeting_response = GreetingResponse {
                        version: 0x5,
                        auth: 0xFF,
                    };

                    client.socket.write(&greeting_response.to_bytes())?;

                    None
                }
            }

            ClientState::AwaitingConnection if revents.contains(PollFlags::POLLIN) => {
                let n = client.socket.read(&mut self.buffer)?;

                let Some(connection_request) = ConnectionRequest::from_bytes(&self.buffer[..n])
                else {
                    return Ok(None);
                };

                match &connection_request.destination {
                    Address::IPv4(ipv4) => {
                        let destination = Self::connect_to(&ipv4)?;

                        Some(ClientState::Connected { destination })
                    }

                    Address::Domain(domain) => {
                        self.queries.push(domain.to_string());

                        Some(ClientState::AwaitingDNS {
                            domain: domain.to_string(),
                        })
                    }

                    Address::IPv6(_) => {
                        eprintln!("IPv6 unsupported");
                        None
                    }
                }
            }

            ClientState::AwaitingDNS { domain } => {
                if let Some(ip) = self
                    .answers
                    .iter()
                    .filter_map(|(name, ip)| (*name == domain).then_some(ip))
                    .next()
                {
                    let destination = Self::connect_to(&ip)?;
                    Some(ClientState::AwaitingConnectionResponse { destination })
                } else {
                    Some(ClientState::AwaitingDNS { domain })
                }
            }

            ClientState::AwaitingConnectionResponse { destination }
                if revents.contains(PollFlags::POLLOUT) =>
            {
                let addr = destination.peer_addr()?;

                let Some(ipv4) = addr.as_socket_ipv4() else {
                    return Ok(None);
                };

                let address = Address::IPv4(ipv4.ip().clone());

                let connection_response = ConnectionResponse {
                    version: 0x5,
                    status: 0x00,
                    bind_address: address,
                    bind_port: ipv4.port(),
                };

                client.socket.write(&connection_response.to_bytes())?;

                Some(ClientState::Connected { destination })
            }

            ClientState::Connected { destination } => {
                if revents.contains(PollFlags::POLLIN) {
                    let n = client.socket.read(&mut self.buffer)?;
                    if n == 0 {
                        return Ok(None);
                    }

                    destination.write(&self.buffer[..n])?;
                }

                Some(ClientState::Connected { destination })
            }

            _ => Some(client.state),
        };

        if let Some(state) = state {
            Ok(Some(Client {
                socket: client.socket,
                state,
            }))
        } else {
            Ok(None)
        }
    }

    fn setup_pollfds(&mut self) -> Vec<PollFd> {
        let mut pollfds = Vec::new();

        pollfds.push(PollFd::new(self.this.as_fd(), PollFlags::POLLIN));

        let dns_events = if self.queries.is_empty() {
            PollFlags::POLLIN
        } else {
            PollFlags::POLLIN | PollFlags::POLLOUT
        };

        pollfds.push(PollFd::new(self.dns.as_fd(), dns_events));

        for (_, client) in &self.clients {
            let events = match client.state {
                ClientState::AwaitingGreeting => PollFlags::POLLIN,
                ClientState::AwaitingGreetingResponse { .. } => PollFlags::POLLOUT,
                ClientState::AwaitingConnection => PollFlags::POLLIN,
                ClientState::AwaitingDNS { .. } => PollFlags::empty(),
                ClientState::AwaitingConnectionResponse { .. } => PollFlags::POLLOUT,
                ClientState::Connected { .. } => PollFlags::POLLIN,
            };

            pollfds.push(PollFd::new(client.socket.as_fd(), events));

            match &client.state {
                ClientState::Connected { destination } => {
                    pollfds.push(PollFd::new(destination.as_fd(), PollFlags::POLLIN))
                }

                _ => {}
            }
        }

        pollfds
    }

    pub fn connect_to(ip: &Ipv4Addr) -> io::Result<Socket> {
        let destination = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
        destination.connect(&SocketAddrV4::new(*ip, 443).into())?;
        destination.set_nonblocking(true)?;
        Ok(destination)
    }

    pub fn run(&mut self) -> io::Result<()> {
        loop {
            let mut pollfds = self.setup_pollfds();

            nix::poll::poll(&mut pollfds, PollTimeout::NONE)?;

            eprintln!(
                "{}",
                pollfds.iter().filter(|fd| fd.events().is_empty()).count()
            );

            let pollfds: Vec<_> = pollfds
                .into_iter()
                .filter_map(|pollfd| {
                    if let Some(revents) = pollfd.revents() {
                        Some((pollfd.as_fd().as_raw_fd(), revents))
                    } else {
                        None
                    }
                })
                .collect();

            let [(_, server_revents), (_, dns_revents), others @ ..] = &pollfds[..] else {
                unreachable!();
            };

            if server_revents.contains(PollFlags::POLLIN) {
                let (conn, _) = self.this.accept()?;

                let client = Client {
                    socket: conn,
                    state: ClientState::AwaitingGreeting,
                };

                self.clients.insert(client.socket.as_raw_fd(), client);
            }

            if dns_revents.contains(PollFlags::POLLIN) {
                self.recv_dns()?;
            }

            if dns_revents.contains(PollFlags::POLLOUT) {
                if let Some(query) = self.queries.pop() {
                    self.send_dns(&query)?;
                }
            }

            for (fd, revents) in others {
                if revents.intersects(PollFlags::POLLHUP | PollFlags::POLLERR) {
                    self.clients.remove(fd);
                    continue;
                }

                if let Some(client) = self.clients.remove(fd) {
                    if let Some(next) = self.transition_client(client, *revents)? {
                        self.clients.insert(*fd, next);
                    }
                } else if let Some((server, client)) =
                    self.clients.values().find_map(|client| match client.state {
                        ClientState::Connected { ref destination }
                            if destination.as_raw_fd() == *fd =>
                        {
                            Some((destination, &client.socket))
                        }
                        _ => None,
                    })
                {
                    if revents.contains(PollFlags::POLLIN) {
                        let n = server.read(&mut self.buffer)?;

                        if n == 0 {
                            self.clients.remove(&client.as_raw_fd());
                            continue;
                        }

                        client.write(&self.buffer[..n])?;
                    }
                }
            }
        }
    }
}

pub enum ClientState {
    AwaitingGreeting,
    AwaitingGreetingResponse { request: GreetingRequest },
    AwaitingConnection,
    AwaitingDNS { domain: String },
    AwaitingConnectionResponse { destination: Socket },
    Connected { destination: Socket },
}

pub struct Client {
    socket: Socket,
    state: ClientState,
}

pub trait SocketExt {
    fn read(&self, buffer: &mut [u8]) -> io::Result<usize>;
    fn write(&self, buffer: &[u8]) -> io::Result<usize>;
}

impl SocketExt for socket2::Socket {
    fn read(&self, buffer: &mut [u8]) -> io::Result<usize> {
        self.recv(unsafe { std::mem::transmute(buffer) })
    }

    fn write(&self, buffer: &[u8]) -> io::Result<usize> {
        self.send(buffer)
    }
}
