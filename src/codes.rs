// Telnet command codes needed for command, negotiation and
// subnegotiation serializing/deserializing
pub const IAC: u8 = 255;
pub const WILL: u8 = 251;
pub const WONT: u8 = 252;
pub const DO: u8 = 253;
pub const DONT: u8 = 254;
pub const SB: u8 = 250;
pub const SE: u8 = 240;

pub const COM_PORT_OPTION: u8 = 44;
