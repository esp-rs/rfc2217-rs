use crate::codes;

pub const SIZE: usize = 2;

// Telnet commands without the ones related to negotiation and subnegotiation,
// defined here: https://www.rfc-editor.org/rfc/rfc854.txt
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Command {
    NoOp,
    DataMark,
    Break,
    InterruptProcess,
    AbortOutput,
    AreYouThere,
    EraseCharacter,
    EraseLine,
    GoAhead,
    Unsupported(u8),
}

impl Command {
    pub fn serialize(&self, buf: &mut [u8]) {
        buf[0] = codes::IAC;
        buf[1] = match *self {
            Self::NoOp => 241,
            Self::DataMark => 242,
            Self::Break => 243,
            Self::InterruptProcess => 244,
            Self::AbortOutput => 245,
            Self::AreYouThere => 246,
            Self::EraseCharacter => 247,
            Self::EraseLine => 248,
            Self::GoAhead => 249,
            Self::Unsupported(byte) => byte,
        }
    }

    pub const fn deserialize(buf: &[u8]) -> Self {
        assert!(buf[0] == codes::IAC);
        match buf[1] {
            241 => Self::NoOp,
            242 => Self::DataMark,
            243 => Self::Break,
            244 => Self::InterruptProcess,
            245 => Self::AbortOutput,
            246 => Self::AreYouThere,
            247 => Self::EraseCharacter,
            248 => Self::EraseLine,
            249 => Self::GoAhead,
            _ => Self::Unsupported(buf[1]),
        }
    }
}
