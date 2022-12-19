use crate::serialport_conversions::*;
use crate::{
    codes, negotiation, parser, subnegotiation, Command, Negotiation, Parser, Subnegotiation,
};
use serialport::{ClearBuffer, FlowControl, SerialPort};
use std::io::{self, BufWriter, Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

#[derive(Debug)]
pub enum Error {
    Parsing(parser::Error),
    SerialInit(serialport::Error),
    Serial(io::Error),
    Tcp(io::Error),
}

pub struct Server {
    port: Box<dyn SerialPort>,
    port_writer: BufWriter<Box<dyn SerialPort>>,
    tcp_conn: TcpStream,
    tcp_writer: BufWriter<TcpStream>,
    tcp_answer_buf: [u8; subnegotiation::MAX_SIZE],
    parser: Parser,
    signature: Vec<u8>,
    suspended_flow_control: FlowControl,
    break_state: bool,
}

impl Server {
    pub fn new<A: ToSocketAddrs>(serial_port_name: &str, tcp_addr: A) -> Result<Self, Error> {
        let port = serialport::new(serial_port_name, 9600)
            .open()
            .map_err(Error::SerialInit)?;
        let port_clone = port.try_clone().map_err(Error::SerialInit)?;
        let listener = TcpListener::bind(tcp_addr).map_err(Error::Tcp)?;
        let (connection, _) = listener.accept().map_err(Error::Tcp)?;
        connection.set_nonblocking(true).map_err(Error::Tcp)?;
        let cloned_connection = connection.try_clone().map_err(Error::Tcp)?;

        Ok(Server {
            port: port,
            parser: Parser::new(),
            port_writer: BufWriter::new(port_clone),
            tcp_conn: connection,
            tcp_writer: BufWriter::new(cloned_connection),
            tcp_answer_buf: [0; subnegotiation::MAX_SIZE],
            signature: Vec::new(),
            suspended_flow_control: FlowControl::None,
            break_state: false,
        })
    }

    pub fn run(&mut self) -> Result<(), Error> {
        // Read and handle the data from the TCP connection
        let mut tcp_data = [0; 256];
        match self.tcp_conn.read(&mut tcp_data) {
            Ok(bytes_read) => {
                self.process_tcp_data(&tcp_data[..bytes_read])?;
            }
            Err(error) => match error.kind() {
                io::ErrorKind::WouldBlock => {}
                _ => return Err(Error::Tcp(error)),
            },
        }

        // Read and handle the data from the serial port
        let mut port_data = [0; 256];
        match self.port.read(&mut port_data) {
            Ok(bytes_read) => {
                for &byte in &port_data[..bytes_read] {
                    // Escape all IAC bytes
                    self.tcp_writer.write_all(&[byte]).map_err(Error::Tcp)?;
                    if byte == codes::IAC {
                        self.tcp_writer.write_all(&[byte]).map_err(Error::Tcp)?;
                    }
                }
            }
            Err(error) => match error.kind() {
                io::ErrorKind::TimedOut => {}
                _ => return Err(Error::Serial(error)),
            },
        }

        // Flush the buffered data to be sent
        self.port_writer.flush().map_err(Error::Serial)?;
        self.tcp_writer.flush().map_err(Error::Tcp)?;

        Ok(())
    }

    fn process_tcp_data(&mut self, bytes: &[u8]) -> Result<(), Error> {
        for &byte in bytes {
            if let Some(event) = self.parser.process_byte(byte).map_err(Error::Parsing)? {
                let answer_size = self.process_event(event).map_err(Error::Serial)?;
                self.tcp_writer
                    .write_all(&self.tcp_answer_buf[..answer_size])
                    .map_err(Error::Tcp)?;
            }
        }
        Ok(())
    }

    fn process_event(&mut self, event: parser::Event) -> Result<usize, io::Error> {
        match event {
            parser::Event::Data(byte) => {
                self.port_writer.write_all(&[byte])?;
                Ok(0)
            }
            parser::Event::Command(command) => self.process_command(command),
            parser::Event::Negotiation(negotiation) => self.process_negotiation(negotiation),
            parser::Event::Subnegotiation(subnegotiation) => {
                self.process_subnegotiation(subnegotiation)
            }
        }
    }

    fn process_command(&mut self, command: Command) -> Result<usize, io::Error> {
        match command {
            _ => Ok(0),
        }
    }

    fn process_negotiation(&mut self, negotiation: Negotiation) -> Result<usize, io::Error> {
        match negotiation.get_answer() {
            Some(answer) => {
                answer.serialize(&mut self.tcp_answer_buf[..negotiation::SIZE]);
                Ok(negotiation::SIZE)
            }
            None => Ok(0),
        }
    }

    fn process_subnegotiation(
        &mut self,
        subnegotiation: Subnegotiation,
    ) -> Result<usize, io::Error> {
        let answer_opt = match subnegotiation {
            Subnegotiation::SetSignature { data, size } => {
                // An empty signature constitutes a signature query
                if size == 0 {
                    let mut data = [0; subnegotiation::MAX_DATA_SIZE];
                    let size = self.signature.len() as u8;
                    data.copy_from_slice(&self.signature);
                    Some(Subnegotiation::SetSignature { data, size })
                } else {
                    self.signature.copy_from_slice(&data[..size as usize]);
                    Some(subnegotiation)
                }
            }

            Subnegotiation::SetBaudRate(val) => {
                if val == 0 {
                    Some(Subnegotiation::SetBaudRate(self.port.baud_rate()?))
                } else {
                    self.port.set_baud_rate(val)?;
                    Some(subnegotiation)
                }
            }

            Subnegotiation::SetDataSize(val) => match u8_to_data_bits(val) {
                Some(data_bits) => {
                    self.port.set_data_bits(data_bits)?;
                    Some(subnegotiation)
                }
                None => Some(Subnegotiation::SetDataSize(data_bits_to_u8(
                    self.port.data_bits()?,
                ))),
            },

            Subnegotiation::SetParity(val) => match u8_to_parity(val) {
                Some(parity) => {
                    self.port.set_parity(parity)?;
                    Some(subnegotiation)
                }
                None => Some(Subnegotiation::SetParity(parity_to_u8(self.port.parity()?))),
            },

            Subnegotiation::SetStopSize(val) => match u8_to_stop_bits(val) {
                Some(stop_bits) => {
                    self.port.set_stop_bits(stop_bits)?;
                    Some(subnegotiation)
                }
                None => Some(Subnegotiation::SetStopSize(stop_bits_to_u8(
                    self.port.stop_bits()?,
                ))),
            },

            Subnegotiation::SetControl(val) => self.handle_set_control(val)?,

            Subnegotiation::FlowControlSuspend => {
                self.suspended_flow_control = self.port.flow_control()?;
                self.port.set_flow_control(FlowControl::None)?;
                Some(subnegotiation)
            }

            Subnegotiation::FlowControlResume => {
                self.port.set_flow_control(self.suspended_flow_control)?;
                Some(subnegotiation)
            }

            Subnegotiation::PurgeData(val) => self.handle_purge_data(val)?,

            _ => None,
        };

        match answer_opt {
            Some(answer) => Ok(answer.serialize_server(&mut self.tcp_answer_buf)),
            None => Ok(0),
        }
    }

    fn handle_set_control(&mut self, val: u8) -> Result<Option<Subnegotiation>, io::Error> {
        match val {
            0 => Ok(Some(Subnegotiation::SetControl(flow_control_to_u8(
                self.port.flow_control()?,
            )))),
            1 | 2 | 3 => {
                self.port
                    .set_flow_control(u8_to_flow_control(val).unwrap())?;
                Ok(Some(Subnegotiation::SetControl(val)))
            }
            4 => match self.break_state {
                true => Ok(Some(Subnegotiation::SetControl(5))),
                false => Ok(Some(Subnegotiation::SetControl(6))),
            },
            5 => {
                self.port.set_break()?;
                self.break_state = true;
                Ok(Some(Subnegotiation::SetControl(val)))
            }
            6 => {
                self.port.clear_break()?;
                self.break_state = false;
                Ok(Some(Subnegotiation::SetControl(val)))
            }
            7 => match self.port.read_data_set_ready()? {
                true => Ok(Some(Subnegotiation::SetControl(8))),
                false => Ok(Some(Subnegotiation::SetControl(9))),
            },
            8 => {
                self.port.write_data_terminal_ready(true)?;
                Ok(Some(Subnegotiation::SetControl(val)))
            }
            9 => {
                self.port.write_data_terminal_ready(false)?;
                Ok(Some(Subnegotiation::SetControl(val)))
            }
            10 => match self.port.read_clear_to_send()? {
                true => Ok(Some(Subnegotiation::SetControl(11))),
                false => Ok(Some(Subnegotiation::SetControl(12))),
            },
            11 => {
                self.port.write_request_to_send(true)?;
                Ok(Some(Subnegotiation::SetControl(val)))
            }
            12 => {
                self.port.write_request_to_send(false)?;
                Ok(Some(Subnegotiation::SetControl(val)))
            }
            _ => Ok(None),
        }
    }

    fn handle_purge_data(&mut self, val: u8) -> Result<Option<Subnegotiation>, io::Error> {
        match val {
            1 => {
                self.port.clear(ClearBuffer::Input)?;
                Ok(Some(Subnegotiation::PurgeData(val)))
            }
            2 => {
                self.port.clear(ClearBuffer::Output)?;
                Ok(Some(Subnegotiation::PurgeData(val)))
            }
            3 => {
                self.port.clear(ClearBuffer::Input)?;
                self.port.clear(ClearBuffer::Output)?;
                Ok(Some(Subnegotiation::PurgeData(val)))
            }
            _ => Ok(None),
        }
    }
}

impl Negotiation {
    fn get_answer(&self) -> Option<Negotiation> {
        match (self.intent, self.option) {
            (
                negotiation::Intent::Will,
                negotiation::Option::Binary
                | negotiation::Option::ComPort
                | negotiation::Option::SuppressGoAhead,
            ) => Some(Negotiation {
                intent: negotiation::Intent::Do,
                option: self.option,
            }),
            (
                negotiation::Intent::Do,
                negotiation::Option::Binary
                | negotiation::Option::ComPort
                | negotiation::Option::SuppressGoAhead,
            ) => None,
            (negotiation::Intent::Will, _) => Some(Negotiation {
                intent: negotiation::Intent::Dont,
                option: self.option,
            }),
            (negotiation::Intent::Do, _) => Some(Negotiation {
                intent: negotiation::Intent::Wont,
                option: self.option,
            }),
            _ => panic!(),
        }
    }
}
