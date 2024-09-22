#![allow(dead_code)]

use std::{
    collections::HashMap,
    io::{self, Read, Write},
    net::{Ipv4Addr, SocketAddrV4},
    os::fd::{AsFd, AsRawFd},
    rc::Rc,
};

use nix::poll::{PollFd, PollFlags, PollTimeout};
use socket2::{Protocol, Socket, Type};

pub mod types;

pub struct Server {
    this: Socket,
    dns: Socket,
    queries: Vec<String>,
    answers: Vec<(String, Ipv4Addr)>,
    clients: Vec<Rc<Client>>,
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
            clients: Vec::new(),
        })
    }

    pub fn recv_dns(&mut self) -> io::Result<(String, Ipv4Addr)> {
        let mut buffer = [0u8; 4096];

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

    pub fn send_dns(&mut self, domain: &str) -> io::Result<()> {
        let mut message = rustdns::Message::default();

        message.add_question(domain, rustdns::Type::A, rustdns::Class::Internet);

        self.dns.write(&message.to_vec()?)?;

        Ok(())
    }

    pub fn run(&mut self) -> io::Result<()> {
        loop {
            let mut pollfds = Vec::new();
            let mut map: HashMap<i32, Rc<Client>> = HashMap::new();

            pollfds.push(PollFd::new(self.this.as_fd(), PollFlags::POLLIN));
            pollfds.push(PollFd::new(
                self.dns.as_fd(),
                PollFlags::POLLIN | PollFlags::POLLOUT,
            ));

            for client in &self.clients {
                let events = match client.state {
                    ClientState::AwaitingGreeting => PollFlags::POLLIN,
                    ClientState::AwaitingGreetingResponse => PollFlags::POLLOUT,
                    ClientState::AwaitingConnection => PollFlags::POLLIN,
                    ClientState::AwaitingDNS { .. } => PollFlags::empty(),
                    ClientState::AwaitingConnectionResponse => PollFlags::POLLOUT,
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

            nix::poll::poll(&mut pollfds, PollTimeout::NONE)?;

            let pollfds: Vec<_> = pollfds
                .into_iter()
                .map(|pollfd| (pollfd.as_fd().as_raw_fd(), pollfd.revents()))
                .collect();

            for (i, (fd, revents)) in pollfds.into_iter().enumerate() {
                if i == 0 {
                    if let Some(revents) = revents {
                        if revents.contains(PollFlags::POLLIN) {
                            let (conn, _) = self.this.accept()?;

                            let client = Rc::new(Client {
                                socket: conn,
                                state: ClientState::AwaitingGreeting,
                            });

                            self.clients.push(client);
                        }
                    }
                }

                if i == 1 {
                    if let Some(revents) = revents {
                        if revents.contains(PollFlags::POLLIN) {
                            let answer = self.recv_dns()?;
                            self.answers.push(answer);
                        }

                        if revents.contains(PollFlags::POLLOUT) {
                            if let Some(query) = self.queries.pop() {
                                self.send_dns(&query)?;
                            }
                        }
                    }
                }
            }
        }
    }
}

pub enum ClientState {
    AwaitingGreeting,
    AwaitingGreetingResponse,
    AwaitingConnection,
    AwaitingDNS { domain: String },
    AwaitingConnectionResponse,
    Connected { destination: Socket },
}

pub struct Client {
    socket: Socket,
    state: ClientState,
}
