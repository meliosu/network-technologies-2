use std::{
    io::{self, Read, Write},
    mem,
    net::{Ipv4Addr, SocketAddrV4},
    os::fd::{AsFd, AsRawFd},
};

use nix::poll::{PollFd, PollFlags, PollTimeout};
use rustdns::{Class, Message, Resource, Type};
use socket2::{Domain, Protocol, Socket};

use socket_ext::SocketExt;

use crate::types::{
    Address, ConnectionRequest, ConnectionResponse, GreetingRequest, GreetingResponse,
};

mod socket_ext;

pub struct Server {
    incoming: Socket,
    dns: Socket,
    clients: Vec<Client>,
}

pub struct Client {
    socket: Socket,
    state: State,
}

impl Client {
    pub fn new(socket: Socket) -> Self {
        Self {
            socket,
            state: State::AwaitingGreetingRequest,
        }
    }
}

pub enum State {
    AwaitingGreetingRequest,
    AwaitingGreetingResponse {
        response: GreetingResponse,
    },
    AwaitingConnectionRequest,
    AwaitingDnsAnswer {
        request: ConnectionRequest,
        domain: String,
    },
    AwaitingDestinationConnection {
        addr: SocketAddrV4,
        dest: Socket,
    },
    AwaitingConnectionResponse {
        response: ConnectionResponse,
        dest: Socket,
    },
    Connected {
        dest: Socket,
    },
}

impl Server {
    pub fn new(port: u16) -> io::Result<Self> {
        let incoming = Socket::new(Domain::IPV4, socket2::Type::STREAM, Some(Protocol::TCP))?;
        incoming.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port).into())?;
        incoming.listen(10)?;
        incoming.set_nonblocking(true)?;

        let dns = Socket::new(Domain::IPV4, socket2::Type::DGRAM, Some(Protocol::UDP))?;
        dns.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0).into())?;
        dns.connect(&SocketAddrV4::new(Ipv4Addr::new(8, 8, 8, 8), 53).into())?;
        dns.set_nonblocking(true)?;

        Ok(Self {
            incoming,
            dns,
            clients: Vec::new(),
        })
    }

    pub fn send_dns(&self, domain: &str) -> io::Result<()> {
        let mut message = Message::default();
        message.add_question(domain, Type::A, Class::Internet);
        let bytes = message.to_vec()?;

        if bytes.len() != self.dns.send(&bytes)? {
            Err(io::Error::other("error sending dns message"))
        } else {
            Ok(())
        }
    }

    pub fn recv_dns(&self) -> io::Result<(String, Ipv4Addr)> {
        let mut buffer = [0u8; 4096];
        let n = self.dns.recv(unsafe { mem::transmute(&mut buffer[..]) })?;
        let message = Message::from_slice(&buffer[..n])?;

        for answer in message.answers {
            match answer.resource {
                Resource::A(ipv4) if answer.name == message.questions[0].name => {
                    return Ok((String::from(answer.name.strip_suffix('.').unwrap()), ipv4))
                }

                _ => {}
            }
        }

        Err(io::Error::other(
            "error getting A dns answer for requested domain",
        ))
    }

    pub fn conn(addr: SocketAddrV4) -> io::Result<Socket> {
        let socket = Socket::new(Domain::IPV4, socket2::Type::STREAM, Some(Protocol::TCP))?;
        socket.connect(&addr.into())?;
        socket.set_nonblocking(true)?;
        Ok(socket)
    }

    pub fn pollfds(&self) -> Vec<PollFd> {
        let mut pollfds = Vec::new();

        pollfds.push(PollFd::new(self.incoming.as_fd(), PollFlags::POLLIN));
        pollfds.push(PollFd::new(self.dns.as_fd(), PollFlags::POLLIN));

        for client in &self.clients {
            let events = match client.state {
                State::AwaitingGreetingRequest => PollFlags::POLLIN,
                State::AwaitingGreetingResponse { .. } => PollFlags::POLLOUT,
                State::AwaitingConnectionRequest => PollFlags::POLLIN,
                State::AwaitingDnsAnswer { .. } => PollFlags::empty(),
                State::AwaitingDestinationConnection { .. } => PollFlags::empty(),
                State::AwaitingConnectionResponse { .. } => PollFlags::POLLOUT,
                State::Connected { .. } => PollFlags::POLLIN,
            };

            pollfds.push(PollFd::new(client.socket.as_fd(), events));

            match client.state {
                State::AwaitingDestinationConnection { ref dest, .. } => {
                    pollfds.push(PollFd::new(dest.as_fd(), PollFlags::POLLOUT));
                }

                State::Connected { ref dest } => {
                    pollfds.push(PollFd::new(dest.as_fd(), PollFlags::POLLIN));
                }

                _ => {}
            }
        }

        pollfds
    }

    pub fn transform(&self, client: Client, events: PollFlags) -> io::Result<Client> {
        let state = match client.state {
            State::AwaitingGreetingRequest => {
                if !events.contains(PollFlags::POLLIN) {
                    panic!("events do not contain flag");
                }

                let request: GreetingRequest = client.socket.recv_packet()?;

                let response = if !request.auths.contains(&0x0) {
                    GreetingResponse::new(0x0)
                } else {
                    GreetingResponse::new(0xFF)
                };

                Some(State::AwaitingGreetingResponse { response })
            }

            State::AwaitingGreetingResponse { response } => {
                if !events.contains(PollFlags::POLLOUT) {
                    panic!("events do not contain flag");
                }

                client.socket.send_packet(&response)?;

                if response.auth == 0xFF {
                    Some(State::AwaitingConnectionRequest)
                } else {
                    None
                }
            }

            State::AwaitingConnectionRequest => {
                if !events.contains(PollFlags::POLLIN) {
                    panic!("events do not contain flag");
                }

                let request: ConnectionRequest = client.socket.recv_packet()?;

                match request.address {
                    Address::IPv4(ipv4) => {
                        let addr = SocketAddrV4::new(ipv4, request.port);
                        let dest = Self::conn(addr)?;
                        Some(State::AwaitingDestinationConnection { dest, addr })
                    }

                    Address::IPv6(_) => unimplemented!(),

                    Address::Domain(ref domain) => {
                        self.send_dns(&domain)?;
                        Some(State::AwaitingDnsAnswer {
                            domain: domain.clone(),
                            request,
                        })
                    }
                }
            }

            State::AwaitingDnsAnswer { .. } => {
                panic!("should not be here!");
            }

            State::AwaitingDestinationConnection { dest, addr } => {
                if !events.contains(PollFlags::POLLOUT) {
                    panic!("events do not contain flag");
                }

                let response = ConnectionResponse::new(addr);
                Some(State::AwaitingConnectionResponse { response, dest })
            }

            State::AwaitingConnectionResponse { response, dest } => {
                if !events.contains(PollFlags::POLLOUT) {
                    panic!("events do not contain flag");
                }

                client.socket.send_packet(&response)?;

                if response.status != 0x0 {
                    None
                } else {
                    Some(State::Connected { dest })
                }
            }

            State::Connected { dest } => {
                if !events.contains(PollFlags::POLLIN) {
                    panic!("events do not contain flag");
                }

                let mut buffer = [0u8; 16384];

                let n = client
                    .socket
                    .recv(unsafe { mem::transmute(&mut buffer[..]) })?;

                dest.send(&buffer[..n])?;

                Some(State::Connected { dest })
            }
        };

        if let Some(state) = state {
            Ok(Client {
                socket: client.socket,
                state,
            })
        } else {
            Err(io::Error::other("error in client"))
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        loop {
            let mut pollfds = self.pollfds();

            nix::poll::poll(&mut pollfds, PollTimeout::NONE)?;

            let pollfds = pollfds
                .into_iter()
                .map(|p| (p.as_fd().as_raw_fd(), p.revents()))
                .collect::<Vec<_>>();

            self.clients = mem::take(&mut self.clients)
                .into_iter()
                .filter_map(|client| {
                    let find_events = |fd| {
                        pollfds
                            .iter()
                            .find_map(|&(d, e)| if d == fd { e } else { None })
                    };

                    let client_events = find_events(client.socket.as_raw_fd());

                    match client.state {
                        State::AwaitingGreetingRequest
                        | State::AwaitingConnectionResponse { .. }
                        | State::AwaitingConnectionRequest
                        | State::AwaitingGreetingResponse { .. }
                        | State::AwaitingDnsAnswer { .. } => {
                            if let Some(events) = client_events {
                                self.transform(client, events).ok()
                            } else {
                                None
                            }
                        }

                        State::AwaitingDestinationConnection { addr, dest } => {
                            let server_events = find_events(dest.as_raw_fd());

                            if server_events.is_some_and(|e| e.contains(PollFlags::POLLOUT)) {
                                Some(Client {
                                    socket: client.socket,
                                    state: State::AwaitingConnectionResponse {
                                        response: ConnectionResponse::new(addr),
                                        dest,
                                    },
                                })
                            } else {
                                Some(Client {
                                    socket: client.socket,
                                    state: State::AwaitingDestinationConnection { addr, dest },
                                })
                            }
                        }

                        State::Connected { dest } => {
                            let server_events = find_events(dest.as_raw_fd());

                            if server_events.is_some_and(|e| e.contains(PollFlags::POLLIN)) {
                                let mut buffer = [0u8; 16384];

                                let n = dest
                                    .recv(unsafe { mem::transmute(&mut buffer[..]) })
                                    .unwrap();

                                client.socket.send(&buffer[..n]).unwrap();
                            }

                            Some(Client {
                                socket: client.socket,
                                state: State::Connected { dest },
                            })
                        }
                    }
                })
                .collect();

            // Handle connections to the proxy server
            if pollfds[0].1.is_some_and(|e| e.contains(PollFlags::POLLIN)) {
                let (conn, _) = self.incoming.accept()?;
                self.clients.push(Client::new(conn));
            }

            // Handle DNS
            if pollfds[0].1.is_some_and(|e| e.contains(PollFlags::POLLIN)) {
                let (domain, ip) = self.recv_dns()?;

                for client in &mut self.clients {
                    match &client.state {
                        State::AwaitingDnsAnswer {
                            domain: name,
                            request,
                        } if domain == *name => {
                            let addr = SocketAddrV4::new(ip, request.port);
                            let dest = Self::conn(addr)?;
                            client.state = State::AwaitingDestinationConnection { dest, addr };
                        }

                        _ => {}
                    }
                }
            }
        }
    }
}
