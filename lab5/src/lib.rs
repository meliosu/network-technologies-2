use std::{
    collections::HashMap,
    io::{self},
    net::{Ipv4Addr, SocketAddrV4},
    os::fd::{AsFd, AsRawFd},
};

use nix::poll::{PollFd, PollFlags, PollTimeout};
use rustdns::Resource;
use socket2::{Domain, Protocol, Socket, Type};

use socket_ext::SocketExt;
use types::{Address, ConnectionRequest, ConnectionResponse, GreetingRequest, GreetingResponse};

pub mod socket_ext;
pub mod types;

pub struct Server {
    this: Socket,
    dns: Socket,
    clients: HashMap<i32, Client>,
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
            clients: HashMap::new(),
        })
    }

    fn recv_dns(&mut self) -> io::Result<(String, Option<Ipv4Addr>)> {
        let mut buffer = [0u8; 4096];
        let received = self.dns.read(&mut buffer)?;
        let message = rustdns::Message::from_slice(&buffer[..received])?;
        let name = message.questions[0].name.clone();
        let ip = message.answers.into_iter().find_map(|r| match r.resource {
            Resource::A(ip) if r.name == name => Some(ip),
            _ => None,
        });

        Ok((name.strip_suffix('.').unwrap().to_string(), ip))
    }

    fn send_dns(&self, domain: &str) -> io::Result<()> {
        let mut message = rustdns::Message::default();
        message.add_question(domain, rustdns::Type::A, rustdns::Class::Internet);
        self.dns.send(&message.to_vec()?).map(|_| ())
    }

    fn transition_client(
        &mut self,
        client: Client,
        revents: PollFlags,
    ) -> io::Result<Option<Client>> {
        let state = match client.state {
            State::AwaitingGreetingRequest => {
                if !revents.contains(PollFlags::POLLIN) {
                    eprintln!("NO POLLIN in AwaitingGreetingRequest");
                }

                let request: GreetingRequest = client.socket.recv_packet().unwrap();
                Some(State::AwaitingGreetingResponse { request })
            }

            State::AwaitingGreetingResponse { request } => {
                if !revents.contains(PollFlags::POLLOUT) {
                    eprintln!("NO POLLOUT in AwaitingGreetingResponse");
                }

                if request.auths.contains(&0x0) {
                    let response = GreetingResponse::new(0x0);
                    client.socket.send_packet(&response)?;
                    Some(State::AwaitingConnectionRequest)
                } else {
                    let response = GreetingResponse::new(0xFF);
                    client.socket.send_packet(&response)?;
                    None
                }
            }

            State::AwaitingConnectionRequest => {
                if !revents.contains(PollFlags::POLLIN) {
                    eprintln!("NO POLLIN in AwaitingConnectionRequest");
                }

                let request: ConnectionRequest = client.socket.recv_packet()?;

                match request.address {
                    Address::IPv4(ipv4) => {
                        let destination = Self::connect(SocketAddrV4::new(ipv4, request.port))?;
                        Some(State::Connected { destination })
                    }

                    Address::Domain(ref domain) => {
                        self.send_dns(&domain)?;
                        Some(State::AwaitingDNS {
                            domain: domain.to_string(),
                            request,
                        })
                    }

                    Address::IPv6(_) => None,
                }
            }

            State::AwaitingDNS { domain, request } => Some(State::AwaitingDNS { request, domain }),

            State::AwaitingConnectionResponse { destination, addr } => {
                if !revents.contains(PollFlags::POLLOUT) {
                    eprintln!("NO POLLOUT in AwaitingConnectionResponse");
                }

                let response = ConnectionResponse::new(addr);
                client.socket.send_packet(&response)?;
                Some(State::Connected { destination })
            }

            State::Connected { destination } => {
                let mut buffer = [0u8; 4096];

                if revents.contains(PollFlags::POLLIN) {
                    let n = client.socket.read(&mut buffer)?;
                    if n == 0 {
                        return Ok(None);
                    }

                    destination.write(&buffer[..n])?;
                }

                Some(State::Connected { destination })
            }
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

    fn pollfds(&mut self) -> Vec<PollFd> {
        let mut pollfds = Vec::new();

        pollfds.push(PollFd::new(self.this.as_fd(), PollFlags::POLLIN));
        pollfds.push(PollFd::new(self.dns.as_fd(), PollFlags::POLLIN));

        for (_, client) in &self.clients {
            let events = match client.state {
                State::AwaitingGreetingRequest => PollFlags::POLLIN,
                State::AwaitingGreetingResponse { .. } => PollFlags::POLLOUT,
                State::AwaitingConnectionRequest => PollFlags::POLLIN,
                State::AwaitingDNS { .. } => PollFlags::empty(),
                State::AwaitingConnectionResponse { .. } => PollFlags::POLLOUT,
                State::Connected { .. } => PollFlags::POLLIN,
            };

            pollfds.push(PollFd::new(client.socket.as_fd(), events));

            match &client.state {
                State::Connected { destination } => {
                    pollfds.push(PollFd::new(destination.as_fd(), PollFlags::POLLIN))
                }

                _ => {}
            }
        }

        pollfds
    }

    pub fn connect(addr: SocketAddrV4) -> io::Result<Socket> {
        let destination = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
        destination.connect(&addr.into())?;
        destination.set_nonblocking(true)?;
        Ok(destination)
    }

    pub fn run(&mut self) -> io::Result<()> {
        loop {
            let mut pollfds = self.pollfds();

            nix::poll::poll(&mut pollfds, PollTimeout::NONE)?;

            let pollfds: Vec<_> = pollfds
                .into_iter()
                .map(|pfd| (pfd.as_fd().as_raw_fd(), pfd.revents()))
                .collect();

            for (fd, revents) in pollfds.iter().skip(2).filter_map(|(d, e)| {
                if let Some(events) = e {
                    Some((d, events))
                } else {
                    None
                }
            }) {
                if revents.intersects(PollFlags::POLLHUP | PollFlags::POLLERR | PollFlags::POLLNVAL)
                {
                    self.clients.remove(fd);
                    continue;
                }

                if let Some(client) = self.clients.remove(fd) {
                    if let Some(next) = self.transition_client(client, *revents)? {
                        self.clients.insert(*fd, next);
                    }
                } else if let Some((server, client)) =
                    self.clients.values().find_map(|client| match client.state {
                        State::Connected { ref destination } if destination.as_raw_fd() == *fd => {
                            Some((destination, &client.socket))
                        }
                        _ => None,
                    })
                {
                    if revents.contains(PollFlags::POLLIN) {
                        let mut buffer = [0u8; 16384];
                        let n = server.read(&mut buffer)?;
                        if n == 0 {
                            self.clients.remove(&client.as_raw_fd());
                            continue;
                        }

                        client.write(&buffer[..n])?;
                    }
                }
            }

            if let Some(server_revents) = pollfds[0].1 {
                if server_revents.contains(PollFlags::POLLIN) {
                    let (conn, _) = self.this.accept()?;

                    let client = Client {
                        socket: conn,
                        state: State::AwaitingGreetingRequest,
                    };

                    self.clients.insert(client.socket.as_raw_fd(), client);
                }
            }

            if let Some(dns_revents) = pollfds[1].1 {
                if dns_revents.contains(PollFlags::POLLIN) {
                    let (domain, addr) = self.recv_dns()?;

                    self.clients.retain(|_, client| match &client.state {
                        State::AwaitingDNS {
                            domain: name,
                            request,
                        } if *name == domain => {
                            if let Some(ip) = addr {
                                let Ok(conn) = Self::connect(SocketAddrV4::new(ip, request.port))
                                else {
                                    return false;
                                };

                                client.state = State::AwaitingConnectionResponse {
                                    destination: conn,
                                    addr: SocketAddrV4::new(ip, request.port),
                                };

                                true
                            } else {
                                false
                            }
                        }

                        _ => true,
                    });
                }
            }
        }
    }
}

pub enum State {
    AwaitingGreetingRequest,
    AwaitingGreetingResponse {
        request: GreetingRequest,
    },
    AwaitingConnectionRequest,
    AwaitingDNS {
        request: ConnectionRequest,
        domain: String,
    },
    AwaitingConnectionResponse {
        destination: Socket,
        addr: SocketAddrV4,
    },
    Connected {
        destination: Socket,
    },
}

pub struct Client {
    socket: Socket,
    state: State,
}
