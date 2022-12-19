use crate::codes;

pub const SIZE: usize = 3;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Negotiation {
    pub intent: Intent,
    pub option: Option,
}

// Telnet options
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Option {
    Binary,
    Echo,
    SuppressGoAhead,
    ComPort,
    Unsupported(u8),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Intent {
    Will,
    Wont,
    Do,
    Dont,
}

impl Negotiation {
    pub fn serialize(&self, buf: &mut [u8]) {
        buf[0] = codes::IAC;
        buf[1] = self.intent.to_u8();
        buf[2] = self.option.to_u8();
    }

    pub const fn deserialize(buf: &[u8]) -> Self {
        assert!(buf[0] == codes::IAC);
        Self {
            intent: Intent::from_u8(buf[1]),
            option: Option::from_u8(buf[2]),
        }
    }
}

impl Option {
    const fn from_u8(byte: u8) -> Option {
        match byte {
            0 => Self::Binary,
            1 => Self::Echo,
            3 => Self::SuppressGoAhead,
            44 => Self::ComPort,
            _ => Self::Unsupported(byte),
        }
    }

    const fn to_u8(self) -> u8 {
        match self {
            Self::Binary => 0,
            Self::Echo => 1,
            Self::SuppressGoAhead => 3,
            Self::ComPort => 44,
            Self::Unsupported(byte) => byte,
        }
    }
}

impl Intent {
    const fn from_u8(byte: u8) -> Intent {
        match byte {
            codes::WILL => Self::Will,
            codes::WONT => Self::Wont,
            codes::DO => Self::Do,
            codes::DONT => Self::Dont,
            _ => panic!("Not a command code for negotiation intent"),
        }
    }

    const fn to_u8(self) -> u8 {
        match self {
            Self::Will => codes::WILL,
            Self::Wont => codes::WONT,
            Self::Do => codes::DO,
            Self::Dont => codes::DONT,
        }
    }
}
