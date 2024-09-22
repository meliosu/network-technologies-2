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

pub enum Address {
    IPv4(u32),
    IPv6([u8; 16]),
    Domain(Vec<u8>),
}

impl Address {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            [0x1, addr @ ..] => Some(Self::IPv4(u32::from_be_bytes(addr.try_into().ok()?))),
            [0x3, addr @ ..] => Some(Self::Domain(addr.to_vec())),
            [0x4, addr @ ..] => Some(Self::IPv6(addr.try_into().ok()?)),

            _ => None,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::IPv4(addr) => [&[0x1], &addr.to_be_bytes()[..]].concat(),
            Self::IPv6(addr) => [&[0x3], &addr[..]].concat(),
            Self::Domain(addr) => [&[0x4], &addr[..]].concat(),
        }
    }
}

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
