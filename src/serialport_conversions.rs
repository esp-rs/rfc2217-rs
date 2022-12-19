use serialport::{DataBits, FlowControl, Parity, StopBits};

// Required functions for conversions between serialport Enums and rfc2217 option values
pub(crate) const fn data_bits_to_u8(data_bits: DataBits) -> u8 {
    match data_bits {
        DataBits::Five => 5,
        DataBits::Six => 6,
        DataBits::Seven => 7,
        DataBits::Eight => 8,
    }
}

pub(crate) const fn u8_to_data_bits(value: u8) -> Option<DataBits> {
    match value {
        5 => Some(DataBits::Five),
        6 => Some(DataBits::Six),
        7 => Some(DataBits::Seven),
        8 => Some(DataBits::Eight),
        _ => None,
    }
}

pub(crate) const fn parity_to_u8(parity: Parity) -> u8 {
    match parity {
        Parity::None => 1,
        Parity::Odd => 2,
        Parity::Even => 3,
    }
}

pub(crate) const fn u8_to_parity(value: u8) -> Option<Parity> {
    match value {
        1 => Some(Parity::None),
        2 => Some(Parity::Odd),
        3 => Some(Parity::Even),
        _ => None,
    }
}

pub(crate) const fn stop_bits_to_u8(stop_bits: StopBits) -> u8 {
    match stop_bits {
        StopBits::One => 1,
        StopBits::Two => 2,
    }
}

pub(crate) const fn u8_to_stop_bits(value: u8) -> Option<StopBits> {
    match value {
        1 => Some(StopBits::One),
        2 => Some(StopBits::Two),
        _ => None,
    }
}

pub(crate) const fn flow_control_to_u8(flow_control: FlowControl) -> u8 {
    match flow_control {
        FlowControl::None => 1,
        FlowControl::Software => 2,
        FlowControl::Hardware => 3,
    }
}

pub(crate) const fn u8_to_flow_control(value: u8) -> Option<FlowControl> {
    match value {
        1 => Some(FlowControl::None),
        2 => Some(FlowControl::Software),
        3 => Some(FlowControl::Hardware),
        _ => None,
    }
}
