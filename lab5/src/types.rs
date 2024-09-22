use std::net::{Ipv4Addr, Ipv6Addr};

pub struct GreetingRequest {
    pub version: u8,
    pub auths: Vec<u8>,
}

impl GreetingRequest {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let [version, nauths, auths @ ..] = bytes else {
            return None;
        };

        if *nauths as usize != auths.len() {
            return None;
        }

        Some(Self {
            version: *version,
            auths: auths.to_vec(),
        })
    }
}

pub struct GreetingResponse {
    pub version: u8,
    pub auth: u8,
}

impl GreetingResponse {
    pub fn to_bytes(&self) -> Vec<u8> {
        [self.version, self.auth].to_vec()
    }
}

#[derive(Clone)]
pub enum Address {
    IPv4(Ipv4Addr),
    IPv6(Ipv6Addr),
    Domain(String),
}

impl Address {
    // TODO: make this sane
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            [0x1, addr @ ..] => Some(Self::IPv4(Ipv4Addr::from_bits(u32::from_be_bytes(
                addr.try_into().unwrap(),
            )))),

            [0x3, addr @ ..] => Some(Self::Domain(String::from_utf8(addr.to_vec()).unwrap())),

            [0x4, addr @ ..] => Some(Self::IPv6(Ipv6Addr::from_bits(u128::from_be_bytes(
                addr.try_into().unwrap(),
            )))),

            _ => None,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::IPv4(addr) => [&[0x1], &addr.to_bits().to_be_bytes()[..]].concat(),
            Self::Domain(addr) => [&[0x3], addr.as_bytes()].concat(),
            Self::IPv6(addr) => [&[0x4], &addr.to_bits().to_be_bytes()[..]].concat(),
        }
    }
}

#[derive(Clone)]
pub struct ConnectionRequest {
    pub version: u8,
    pub command: u8,
    pub destination: Address,
    pub port: u16,
}

impl ConnectionRequest {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let [version, command, 0x0, dest @ .., port1, port2] = bytes else {
            return None;
        };

        let destination = Address::from_bytes(dest)?;

        Some(Self {
            version: *version,
            command: *command,
            destination,
            port: u16::from_be_bytes([*port1, *port2]),
        })
    }
}

pub struct ConnectionResponse {
    pub version: u8,
    pub status: u8,
    pub bind_address: Address,
    pub bind_port: u16,
}

impl ConnectionResponse {
    pub fn to_bytes(&self) -> Vec<u8> {
        [
            &[self.version, self.status, 0x0],
            &self.bind_address.to_bytes()[..],
            &self.bind_port.to_be_bytes()[..],
        ]
        .concat()
    }
}
