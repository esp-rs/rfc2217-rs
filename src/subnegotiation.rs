use crate::codes;

pub const MAX_DATA_SIZE: usize = 256;
pub const NONDATA_SIZE: usize = 6;
pub const MAX_SIZE: usize = MAX_DATA_SIZE + NONDATA_SIZE;

// RFC2217 subnegotiation options, defined here: https://www.rfc-editor.org/rfc/rfc2217.html
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Subnegotiation {
    SetSignature {
        data: [u8; MAX_DATA_SIZE],
        size: u8,
    },
    SetBaudRate(u32),
    SetDataSize(u8),
    SetParity(u8),
    SetStopSize(u8),
    SetControl(u8),
    NotifyLineState(u8),
    NotifyModemState(u8),
    FlowControlSuspend,
    FlowControlResume,
    SetLinestateMask(u8),
    SetModemStateMask(u8),
    PurgeData(u8),
    Unsupported {
        base_option_code: u8,
        option_code: u8,
        data: [u8; MAX_DATA_SIZE],
        data_cnt: u8,
    },
}

// The codes for client to server and server to client ComPort options differ by 100,
// this indicates which one the serializer should pick
#[derive(Clone, Copy)]
enum OptionKind {
    ClientToServer,
    ServerToClient,
}

impl Subnegotiation {
    pub fn serialize_client(&self, buf: &mut [u8]) -> usize {
        self.serialize(buf, OptionKind::ClientToServer)
    }

    pub fn serialize_server(&self, buf: &mut [u8]) -> usize {
        self.serialize(buf, OptionKind::ServerToClient)
    }

    fn serialize(&self, buf: &mut [u8], option_kind: OptionKind) -> usize {
        let start = |option_code: u8| -> [u8; 4] {
            [
                codes::IAC,
                codes::SB,
                codes::COM_PORT_OPTION,
                match option_kind {
                    OptionKind::ClientToServer => option_code,
                    OptionKind::ServerToClient => option_code + 100,
                },
            ]
        };

        let end = [codes::IAC, codes::SE];

        let mut subnegotiate = |option_code: u8, data: &[u8]| -> usize {
            buf[..4].copy_from_slice(&start(option_code));
            buf[4..4 + data.len()].copy_from_slice(data);
            buf[4 + data.len()..NONDATA_SIZE + data.len()].copy_from_slice(&end);
            NONDATA_SIZE + data.len()
        };

        match *self {
            Self::SetSignature { data, size } => {
                buf[..4].copy_from_slice(&start(0));
                let data_slice = &data[..size as usize];
                let mut i = 4;
                for byte in data_slice {
                    buf[i] = *byte;
                    i += 1;
                    // Make sure to escape IAC bytes in the signature
                    if *byte == codes::IAC {
                        buf[i] = *byte;
                        i += 1;
                    }
                }
                buf[i..i + 2].copy_from_slice(&end);
                i + 2
            }

            Self::SetBaudRate(baud) => subnegotiate(1, &u32::to_be_bytes(baud)),
            Self::SetDataSize(data_size) => subnegotiate(2, &[data_size]),
            Self::SetParity(parity) => subnegotiate(3, &[parity]),
            Self::SetStopSize(stopsize) => subnegotiate(4, &[stopsize]),
            Self::SetControl(control) => subnegotiate(5, &[control]),
            Self::NotifyLineState(linestate) => subnegotiate(6, &[linestate]),
            Self::NotifyModemState(modemstate) => subnegotiate(7, &[modemstate]),
            Self::FlowControlSuspend => subnegotiate(8, &[]),
            Self::FlowControlResume => subnegotiate(9, &[]),
            Self::SetLinestateMask(linestate_mask) => subnegotiate(10, &[linestate_mask]),
            Self::SetModemStateMask(modemstate_mask) => subnegotiate(11, &[modemstate_mask]),
            Self::PurgeData(purge_data) => subnegotiate(12, &[purge_data]),
            Self::Unsupported {
                base_option_code,
                option_code,
                data,
                data_cnt,
            } => {
                buf[..4].copy_from_slice(&[codes::IAC, codes::SB, base_option_code, option_code]);
                buf[4..4 + data_cnt as usize].copy_from_slice(&data[..data_cnt as usize]);
                buf[4 + data_cnt as usize..NONDATA_SIZE + data_cnt as usize].copy_from_slice(&end);
                NONDATA_SIZE + data_cnt as usize
            }
        }
    }

    pub fn deserialize(buf: &[u8]) -> Self {
        assert!(
            buf[0] == codes::IAC
                && buf[1] == codes::SB
                && buf[buf.len() - 2] == codes::IAC
                && buf[buf.len() - 1] == codes::SE
        );

        let base_option_code = buf[2];
        let option_code = buf[3];
        let data_len = buf.len() - NONDATA_SIZE;
        let data = &buf[4..4 + data_len];

        match base_option_code {
            codes::COM_PORT_OPTION => match option_code {
                0 | 100 => {
                    let mut data_no_escapes = [0; MAX_DATA_SIZE];
                    let mut i = 0;
                    let mut iac_occured = false;
                    for &byte in data {
                        if byte == codes::IAC {
                            if iac_occured {
                                iac_occured = false;
                                continue;
                            } else {
                                iac_occured = true;
                            }
                        }
                        data_no_escapes[i] = byte;
                        i += 1;
                    }
                    Self::SetSignature {
                        data: data_no_escapes,
                        size: i as u8,
                    }
                }
                1 | 101 => {
                    let baud_rate = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                    Self::SetBaudRate(baud_rate)
                }
                2 | 102 => Self::SetDataSize(data[0]),
                3 | 103 => Self::SetParity(data[0]),
                4 | 104 => Self::SetStopSize(data[0]),
                5 | 105 => Self::SetControl(data[0]),
                6 | 106 => Self::NotifyLineState(data[0]),
                7 | 107 => Self::NotifyModemState(data[0]),
                8 | 108 => Self::FlowControlSuspend,
                9 | 109 => Self::FlowControlResume,
                10 | 110 => Self::SetLinestateMask(data[0]),
                11 | 111 => Self::SetModemStateMask(data[0]),
                12 | 112 => Self::PurgeData(data[0]),
                _ => panic!("Option code is not a Com Port option code"),
            },
            _ => {
                let mut data_arr = [0; MAX_DATA_SIZE];
                data_arr.copy_from_slice(data);
                Self::Unsupported {
                    base_option_code: base_option_code,
                    option_code: option_code,
                    data: data_arr,
                    data_cnt: data_len as u8,
                }
            }
        }
    }
}
