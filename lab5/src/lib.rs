#![allow(dead_code)]

use std::{
    collections::HashMap,
    io::{self},
    net::{Ipv4Addr, SocketAddrV4},
    os::fd::{AsFd, AsRawFd},
};

use nix::poll::{PollFd, PollFlags, PollTimeout};
use socket2::{Protocol, Socket, Type};
use types::{ConnectionRequest, GreetingRequest, GreetingResponse};

pub mod types;

pub struct Server {
    this: Socket,
    dns: Socket,
    queries: Vec<String>,
    answers: Vec<(String, Ipv4Addr)>,
    clients: HashMap<i32, Client>,
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
        })
    }

    fn recv_dns(&mut self) -> io::Result<(String, Ipv4Addr)> {
        let mut buffer = [0u8; 16384];

        let received = self.dns.read(&mut buffer)?;

        let rustdns::Message {
            questions, answers, ..
        } = rustdns::Message::from_slice(&buffer[..received])?;

        let domain = questions[0].name.clone();

        let Some(ip) = answers.into_iter().find_map(|a| match a.resource {
            rustdns::Resource::A(ip) => Some(ip),
            _ => None,
        }) else {
            panic!("DNS Query Returned No IPv4 (A) answers (need to handle)");
        };

        Ok((domain, ip))
    }

    fn send_dns(&mut self, domain: &str) -> io::Result<()> {
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
                ClientState::Connected { .. } => PollFlags::POLLIN | PollFlags::POLLOUT,
            };

            pollfds.push(PollFd::new(client.socket.as_fd(), events));

            match &client.state {
                ClientState::Connected { destination } => pollfds.push(PollFd::new(
                    destination.as_fd(),
                    PollFlags::POLLIN | PollFlags::POLLOUT,
                )),

                _ => {}
            }
        }

        pollfds
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
                let answer = self.recv_dns()?;
                self.answers.push(answer);
            }

            if dns_revents.contains(PollFlags::POLLOUT) {
                if let Some(query) = self.queries.pop() {
                    self.send_dns(&query)?;
                }
            }

            for (fd, revents) in others {
                if let Some(ref mut client) = self.clients.get_mut(fd) {
                    match &client.state {
                        ClientState::AwaitingGreeting => {
                            if revents.contains(PollFlags::POLLIN) {
                                let n = client.socket.read(&mut buffer)?;

                                let Some(greeting_request) =
                                    GreetingRequest::from_bytes(&buffer[..n])
                                else {
                                    panic!("invalid greeting request");
                                };

                                if revents.contains(PollFlags::POLLOUT) {
                                    if greeting_request.auths.contains(&0x0) {
                                        let greeting_response = GreetingResponse {
                                            version: 0x5,
                                            auth: 0x0,
                                        };

                                        client.socket.write(&greeting_response.to_bytes())?;
                                    } else {
                                        let greeting_response = GreetingResponse {
                                            version: 0x5,
                                            auth: 0xFF,
                                        };

                                        client.socket.write(&greeting_response.to_bytes())?;

                                        // TODO: close client connection
                                    }

                                    client.state = ClientState::AwaitingConnection;
                                } else {
                                    client.state = ClientState::AwaitingGreetingResponse {
                                        request: greeting_request,
                                    };
                                }
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
                                } else {
                                    let greeting_response = GreetingResponse {
                                        version: 0x5,
                                        auth: 0xFF,
                                    };

                                    client.socket.write(&greeting_response.to_bytes())?;

                                    // TODO: close client connection
                                }
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

                                if revents.contains(PollFlags::POLLOUT) {
                                } else {
                                    client.state = ClientState::AwaitingConnectionResponse {
                                        request: connection_request,
                                    };
                                }
                            }
                        }

                        ClientState::AwaitingDNS { domain } => {
                            if let Some(idx) =
                                self.answers.iter().position(|(name, _)| name == domain)
                            {
                                let (_, ip) = self.answers.remove(idx);

                                todo!()
                            }
                        }

                        ClientState::AwaitingConnectionResponse { request } => {
                            if revents.contains(PollFlags::POLLOUT) {}
                        }

                        ClientState::Connected { destination } => {}
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
    AwaitingConnectionResponse { request: ConnectionRequest },
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
