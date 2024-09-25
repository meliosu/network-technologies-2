use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4};

#[derive(Debug)]
pub struct GreetingRequest {
    pub version: u8,
    pub auths: Vec<u8>,
}

#[derive(Debug)]
pub struct GreetingResponse {
    pub version: u8,
    pub auth: u8,
}

#[derive(Debug)]
pub enum Address {
    IPv4(Ipv4Addr),
    IPv6(Ipv6Addr),
    Domain(String),
}

#[derive(Debug)]
pub struct ConnectionRequest {
    pub version: u8,
    pub command: u8,
    pub address: Address,
    pub port: u16,
}

#[derive(Debug)]
pub struct ConnectionResponse {
    pub version: u8,
    pub status: u8,
    pub address: Address,
    pub port: u16,
}

impl GreetingResponse {
    pub fn new(auth: u8) -> Self {
        Self { version: 0x5, auth }
    }
}

impl ConnectionResponse {
    pub fn new(addr: SocketAddrV4) -> Self {
        Self {
            version: 0x5,
            status: 0x0,
            address: Address::IPv4(*addr.ip()),
            port: addr.port(),
        }
    }
}

pub trait Encode {
    fn to_bytes(&self) -> Vec<u8>;
}

pub trait Decode {
    fn from_bytes(bytes: &[u8]) -> Option<Self>
    where
        Self: Sized;
}

impl Encode for Address {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Address::IPv4(ipv4) => [&[0x1], &ipv4.to_bits().to_be_bytes()[..]].concat(),
            Address::IPv6(ipv6) => [&[0x4], &ipv6.to_bits().to_be_bytes()[..]].concat(),
            Address::Domain(domain) => [&[0x3], &[domain.len() as u8], domain.as_bytes()].concat(),
        }
    }
}

impl Decode for Address {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            [0x1, addr @ ..] => Some(Address::IPv4(Ipv4Addr::from_bits(u32::from_be_bytes(
                addr.try_into().ok()?,
            )))),

            [0x4, addr @ ..] => Some(Address::IPv6(Ipv6Addr::from_bits(u128::from_be_bytes(
                addr.try_into().ok()?,
            )))),

            [0x3, _, addr @ ..] => Some(Address::Domain(String::from_utf8(addr.to_vec()).ok()?)),

            _ => None,
        }
    }
}

impl Decode for GreetingRequest {
    fn from_bytes(bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        let [version, nauth, auths @ ..] = bytes else {
            return None;
        };

        if *nauth as usize != auths.len() {
            return None;
        }

        Some(Self {
            version: *version,
            auths: auths.to_vec(),
        })
    }
}

impl Encode for GreetingResponse {
    fn to_bytes(&self) -> Vec<u8> {
        [self.version, self.auth].to_vec()
    }
}

impl Decode for ConnectionRequest {
    fn from_bytes(bytes: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        let [version, command, 0x0, addr @ .., p1, p2] = bytes else {
            return None;
        };

        Some(Self {
            version: *version,
            command: *command,
            address: Address::from_bytes(addr)?,
            port: u16::from_be_bytes([*p1, *p2]),
        })
    }
}

impl Encode for ConnectionResponse {
    fn to_bytes(&self) -> Vec<u8> {
        [
            &[self.version, self.status, 0x0],
            &self.address.to_bytes()[..],
            &self.port.to_be_bytes()[..],
        ]
        .concat()
    }
}
