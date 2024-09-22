#![allow(dead_code)]

use std::{
    collections::HashMap,
    io::{self, Read, Write},
    net::{Ipv4Addr, SocketAddrV4},
    num::NonZeroUsize,
    os::fd::{AsFd, AsRawFd, FromRawFd},
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
    servers: HashMap<i32, i32>,
}

impl Server {
    pub fn new(port: u16) -> io::Result<Self> {
        let this = Socket::new(socket2::Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
        this.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port).into())?;
        this.listen(16)?;
        this.set_nonblocking(true)?;

        let dns = Socket::new(socket2::Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        dns.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0).into())?;
        dns.connect(&SocketAddrV4::new(Ipv4Addr::new(8, 8, 8, 8), 53).into())?;
        dns.set_nonblocking(true)?;

        Ok(Self {
            this,
            dns,
            queries: Vec::new(),
            answers: Vec::new(),
            clients: HashMap::new(),
            servers: HashMap::new(),
        })
    }

    fn recv_dns(&mut self) -> io::Result<()> {
        let mut buffer = [0u8; 16384];

        let received = self.dns.read(&mut buffer)?;

        let message = rustdns::Message::from_slice(&buffer[..received])?;

        log::trace!("dns answer: {message:#?}");

        let rustdns::Message {
            questions, answers, ..
        } = message;

        //let domain = questions[0].name.clone();

        self.answers
            .extend(answers.into_iter().filter_map(|a| match a.resource {
                rustdns::Resource::A(ip) => {
                    Some((a.name.strip_suffix(".").unwrap().to_string(), ip))
                }
                _ => None,
            }));

        Ok(())

        //let Some(ip) = answers.into_iter().find_map(|a| match a.resource {
        //    rustdns::Resource::A(ip) => Some(ip),
        //    _ => None,
        //}) else {
        //    // TODO: handle properly
        //    panic!("DNS Query Returned No IPv4 (A) answers");
        //};
        //
        //Ok((domain, ip))
    }

    fn send_dns(&mut self, domain: &str) -> io::Result<()> {
        log::trace!("sending dns for domain {domain}");

        let mut message = rustdns::Message::default();

        message.add_question(domain, rustdns::Type::A, rustdns::Class::Internet);

        self.dns.write(&message.to_vec()?).map(|_| ())
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
                ClientState::Connected { destination } => pollfds.push(PollFd::new(
                    destination.as_fd(),
                    PollFlags::POLLIN | PollFlags::POLLOUT,
                )),

                ClientState::AwaitingConnectionResponse { destination } => pollfds.push(
                    PollFd::new(destination.as_fd(), PollFlags::POLLIN | PollFlags::POLLOUT),
                ),

                _ => {}
            }
        }

        pollfds
    }

    pub fn connect_to(ip: &Ipv4Addr) -> io::Result<Socket> {
        let destination = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;

        log::trace!("connecting to {ip}");
        destination.connect(&SocketAddrV4::new(*ip, 443).into())?;
        destination.set_nonblocking(true)?;
        log::trace!("connected to {ip}");

        Ok(destination)
    }

    pub fn run(&mut self) -> io::Result<()> {
        let mut buffer = [0u8; 16384];

        loop {
            let mut pollfds = self.setup_pollfds();

            nix::poll::poll(&mut pollfds, PollTimeout::NONE)?;

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
                //let answer = self.recv_dns()?;
                //self.answers.push(answer);
            }

            if dns_revents.contains(PollFlags::POLLOUT) {
                if let Some(query) = self.queries.pop() {
                    self.send_dns(&query)?;
                }
            }

            for (fd, revents) in others {
                if revents.intersects(PollFlags::POLLHUP | PollFlags::POLLERR) {
                    if let Some(client) = self.clients.remove(fd) {
                        match client.state {
                            ClientState::Connected { destination } => {
                                self.servers.remove(&destination.as_raw_fd());
                            }

                            ClientState::AwaitingConnectionResponse { destination } => {
                                self.servers.remove(&destination.as_raw_fd());
                            }

                            _ => {}
                        }
                    }

                    continue;
                }

                // TODO: make this fucking monstrosity sane
                if let Some(client_fd) = self.servers.get(fd) {
                    if revents.contains(PollFlags::POLLIN) {
                        let server_socket = unsafe { Socket::from_raw_fd(*fd) };
                        let client_socket = unsafe { Socket::from_raw_fd(*client_fd) };

                        let n = server_socket.read(&mut buffer)?;
                        client_socket.write(&buffer[..n])?;

                        //server_socket.sendfile(
                        //    &client_socket.as_raw_fd(),
                        //    0,
                        //    Some(NonZeroUsize::new(16384).unwrap()),
                        //)?;

                        std::mem::forget(server_socket);
                        std::mem::forget(client_socket);
                    }

                    continue;
                }

                if let Some(ref mut client) = self.clients.get_mut(fd) {
                    match std::mem::take(&mut client.state) {
                        ClientState::AwaitingGreeting => {
                            if revents.contains(PollFlags::POLLIN) {
                                let n = client.socket.read(&mut buffer)?;

                                if n == 0 {
                                    eprintln!("read 0 bytes while in greeting");
                                    continue;
                                }

                                let Some(greeting_request) =
                                    GreetingRequest::from_bytes(&buffer[..n])
                                else {
                                    panic!("invalid greeting request");
                                };

                                log::trace!("got greeting request: {greeting_request:#?}");

                                client.state = ClientState::AwaitingGreetingResponse {
                                    request: greeting_request,
                                };
                            } else {
                                eprintln!("NO POLLIN in AwaitingGreeting");
                            }
                        }

                        ClientState::AwaitingGreetingResponse { request } => {
                            if revents.contains(PollFlags::POLLOUT) {
                                if request.auths.contains(&0x0) {
                                    let greeting_response = GreetingResponse {
                                        version: 0x5,
                                        auth: 0x0,
                                    };

                                    client.socket.write(&greeting_response.to_bytes())?;

                                    log::trace!("sent greeting response: {greeting_response:#?}");

                                    client.state = ClientState::AwaitingConnection;
                                } else {
                                    let greeting_response = GreetingResponse {
                                        version: 0x5,
                                        auth: 0xFF,
                                    };

                                    client.socket.write(&greeting_response.to_bytes())?;
                                    log::trace!("sent greeting response: {greeting_response:#?}");

                                    // TODO: close client connection

                                    client.state = ClientState::AwaitingConnection;
                                }
                            } else {
                                eprintln!("NO POLLOUT in AwaitingGreetingResponse");
                            }
                        }

                        ClientState::AwaitingConnection => {
                            if revents.contains(PollFlags::POLLIN) {
                                let n = client.socket.read(&mut buffer)?;

                                let Some(connection_request) =
                                    ConnectionRequest::from_bytes(&buffer[..n])
                                else {
                                    panic!("invalid connection request");
                                };

                                match &connection_request.destination {
                                    Address::IPv4(ipv4) => {
                                        let destination = Self::connect_to(&ipv4)?;

                                        self.servers.insert(
                                            destination.as_raw_fd(),
                                            client.socket.as_raw_fd(),
                                        );

                                        log::trace!("connected to dest");

                                        client.state = ClientState::Connected { destination };
                                    }

                                    Address::IPv6(_) => {
                                        panic!("ipv6 unsupported");
                                    }

                                    Address::Domain(domain) => {
                                        self.queries.push(domain.to_string());

                                        client.state = ClientState::AwaitingDNS {
                                            domain: domain.to_string(),
                                        };

                                        log::trace!("awaiting dns");
                                    }
                                }
                            } else {
                                eprintln!("NO POLLIN in AwaitingConnection");
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

                                self.servers
                                    .insert(destination.as_raw_fd(), client.socket.as_raw_fd());

                                client.state =
                                    ClientState::AwaitingConnectionResponse { destination };

                                log::trace!("connected to destination after dns");
                            } else {
                                eprintln!("didn't find domain {domain}");
                                client.state = ClientState::AwaitingDNS { domain };
                            }
                        }

                        ClientState::AwaitingConnectionResponse { destination } => {
                            if revents.contains(PollFlags::POLLOUT) {
                                let addr = destination.peer_addr()?;

                                let Some(ipv4) = addr.as_socket_ipv4() else {
                                    panic!("non-ipv4 not implemented");
                                };

                                let address = Address::IPv4(ipv4.ip().clone());

                                let connection_response = ConnectionResponse {
                                    version: 0x5,
                                    status: 0x00,
                                    bind_address: address,
                                    bind_port: ipv4.port(),
                                };

                                client.socket.write(&connection_response.to_bytes())?;
                                client.state = ClientState::Connected { destination };

                                log::trace!("fully connected");
                            } else {
                                eprintln!("NO POLLOUT in AwaitingConnectionResponse")
                            }
                        }

                        ClientState::Connected { destination } => {
                            if revents.contains(PollFlags::POLLIN) {
                                // TODO: ensure server has POLLOUT

                                let n = client.socket.read(&mut buffer)?;
                                destination.write(&buffer[..n])?;

                                //client.socket.sendfile(
                                //    &destination,
                                //    0,
                                //    Some(NonZeroUsize::new(16384).unwrap()),
                                //)?;

                                client.state = ClientState::Connected { destination };
                            } else {
                                eprintln!("NO POLLIN in Connected");
                                client.state = ClientState::Connected { destination };
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Default)]
pub enum ClientState {
    #[default]
    AwaitingGreeting,
    AwaitingGreetingResponse {
        request: GreetingRequest,
    },

    AwaitingConnection,
    AwaitingDNS {
        domain: String,
    },

    AwaitingConnectionResponse {
        destination: Socket,
    },

    Connected {
        destination: Socket,
    },
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
