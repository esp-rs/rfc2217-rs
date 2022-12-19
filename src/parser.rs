use crate::{codes, command, negotiation, subnegotiation, Command, Negotiation, Subnegotiation};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Event {
    Data(u8),
    Command(Command),
    Negotiation(Negotiation),
    Subnegotiation(Subnegotiation),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    SubnegotiationParsing,
    BufferOverflow,
}

enum State {
    Data,
    Command,
    Negotiation,
    SubnegotiationOption,
    SubnegotiationSubOption,
    SubnegotiationData,
    SubnegotiationEnd,
}

pub struct Parser {
    state: State,
    buf: [u8; subnegotiation::MAX_SIZE],
    buf_cnt: usize,
}

impl Parser {
    pub const fn new() -> Self {
        Self {
            state: State::Data,
            buf: [0; subnegotiation::MAX_SIZE],
            buf_cnt: 0,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn process_byte(&mut self, byte: u8) -> Result<Option<Event>, Error> {
        match self.state {
            State::Data => {
                if byte == codes::IAC {
                    self.state = State::Command;
                    return Ok(None);
                }
                Ok(Some(Event::Data(byte)))
            }

            State::Command => {
                if byte == codes::IAC {
                    self.state = State::Data;
                    return Ok(Some(Event::Data(byte)));
                }
                self.buf_cnt = 0;
                self.write_to_buf(codes::IAC)?;
                self.write_to_buf(byte)?;
                Ok(self.process_command_byte(byte))
            }

            State::Negotiation => {
                self.write_to_buf(byte)?;
                self.state = State::Data;
                Ok(Some(Event::Negotiation(Negotiation::deserialize(
                    &self.buf[..negotiation::SIZE],
                ))))
            }

            State::SubnegotiationOption => {
                self.write_to_buf(byte)?;
                self.state = State::SubnegotiationSubOption;
                Ok(None)
            }

            State::SubnegotiationSubOption => {
                self.write_to_buf(byte)?;
                self.state = State::SubnegotiationData;
                Ok(None)
            }

            State::SubnegotiationData => {
                self.write_to_buf(byte)?;
                if byte == codes::IAC {
                    self.state = State::SubnegotiationEnd;
                }
                Ok(None)
            }

            State::SubnegotiationEnd => {
                match byte {
                    // If the IAC byte repeats it's data
                    codes::IAC => {
                        self.write_to_buf(byte)?;
                        self.state = State::SubnegotiationData;
                        Ok(None)
                    }
                    codes::SE => {
                        self.write_to_buf(byte)?;
                        self.state = State::Data;
                        Ok(Some(Event::Subnegotiation(Subnegotiation::deserialize(
                            &self.buf[..self.buf_cnt],
                        ))))
                    }
                    _ => Err(Error::SubnegotiationParsing),
                }
            }
        }
    }

    fn process_command_byte(&mut self, command_code: u8) -> Option<Event> {
        match command_code {
            codes::WILL | codes::WONT | codes::DO | codes::DONT => {
                self.state = State::Negotiation;
                None
            }
            codes::SB => {
                self.state = State::SubnegotiationOption;
                None
            }
            _ => {
                self.state = State::Data;
                Some(Event::Command(Command::deserialize(
                    &self.buf[..command::SIZE],
                )))
            }
        }
    }

    fn write_to_buf(&mut self, byte: u8) -> Result<(), Error> {
        if self.buf_cnt == self.buf.len() {
            return Err(Error::BufferOverflow);
        }
        self.buf[self.buf_cnt] = byte;
        self.buf_cnt += 1;
        Ok(())
    }
}
