use parser::{Error, Event};
use rfc2217_rs::*;

#[test]
fn test_negotiation() {
    let mut neg: [u8; 3] = [0; negotiation::SIZE];
    Negotiation {
        intent: negotiation::Intent::Will,
        option: negotiation::Option::Binary,
    }
    .serialize(&mut neg);

    let mut parser = Parser::new();

    let mut result: Result<Option<Event>, Error> = Ok(None);
    for byte in neg {
        result = parser.process_byte(byte);
    }

    assert_eq!(
        result,
        Ok(Some(Event::Negotiation(Negotiation {
            intent: negotiation::Intent::Will,
            option: negotiation::Option::Binary,
        })))
    );
}

#[test]
fn test_negotiation_unsupported() {
    let mut neg: [u8; 3] = [0; negotiation::SIZE];
    Negotiation {
        intent: negotiation::Intent::Wont,
        option: negotiation::Option::Unsupported(66),
    }
    .serialize(&mut neg);

    let mut parser = Parser::new();

    let mut result: Result<Option<Event>, Error> = Ok(None);
    for byte in neg {
        result = parser.process_byte(byte);
    }

    assert_eq!(
        result,
        Ok(Some(Event::Negotiation(Negotiation {
            intent: negotiation::Intent::Wont,
            option: negotiation::Option::Unsupported(neg[2])
        })))
    );
}

#[test]
fn test_command_unsupported() {
    let mut command = [0; command::SIZE];
    Command::Unsupported(239).serialize(&mut command);
    let mut parser = Parser::new();

    let mut result: Result<Option<Event>, Error> = Ok(None);
    for byte in command {
        result = parser.process_byte(byte);
    }

    assert_eq!(
        result,
        Ok(Some(Event::Command(Command::Unsupported(command[1]))))
    );
}

#[test]
fn test_signature_subnegotiation_containing_iac() {
    let mut subneg = [0; 13];
    let mut signature = [0; subnegotiation::MAX_DATA_SIZE];
    signature[..6].copy_from_slice(&[63, 111, 32, 255, 10, 44]);

    Subnegotiation::SetSignature {
        data: signature,
        size: 6,
    }
    .serialize_client(&mut subneg);

    let mut parser = Parser::new();

    let mut result: Result<Option<Event>, Error> = Ok(None);
    for byte in subneg {
        result = parser.process_byte(byte);
    }

    assert_eq!(
        result,
        Ok(Some(Event::Subnegotiation(Subnegotiation::SetSignature {
            data: signature,
            size: 6
        })))
    );
}

#[test]
fn test_baud_subnegotiation_generated() {
    let mut subneg: [u8; 10] = [0; 10];
    let expected_baudrate = 9600;
    Subnegotiation::SetBaudRate(expected_baudrate).serialize_client(&mut subneg);

    let mut parser = Parser::new();

    let mut result: Result<Option<Event>, Error> = Ok(None);
    for byte in subneg {
        result = parser.process_byte(byte);
    }

    assert_eq!(
        result,
        Ok(Some(Event::Subnegotiation(Subnegotiation::SetBaudRate(
            expected_baudrate
        ))))
    );
}

#[test]
fn test_baud_subnegotiation_containing_iac() {
    let mut subneg: [u8; 10] = [0; 10];
    let expected_baudrate = 0x0000FFFF;
    Subnegotiation::SetBaudRate(expected_baudrate).serialize_client(&mut subneg);

    for byte in subneg {
        print!("byte: {}", byte)
    }

    let mut parser = Parser::new();

    let mut result: Result<Option<Event>, Error> = Ok(None);
    for byte in subneg {
        result = parser.process_byte(byte);
    }

    assert_eq!(
        result,
        Ok(Some(Event::Subnegotiation(Subnegotiation::SetBaudRate(
            expected_baudrate
        ))))
    );
}

#[test]
fn test_parity_subnegotiation() {
    let mut subneg = [0; 7];
    Subnegotiation::SetParity(1).serialize_client(&mut subneg);

    let mut parser = Parser::new();

    let mut result: Result<Option<Event>, Error> = Ok(None);
    for byte in subneg {
        result = parser.process_byte(byte);
    }

    assert_eq!(
        result,
        Ok(Some(Event::Subnegotiation(Subnegotiation::SetParity(1))))
    );
}

#[test]
fn test_flow_control_suspend_subnegotiation() {
    let mut subneg = [0; 6];
    Subnegotiation::FlowControlSuspend.serialize_client(&mut subneg);

    let mut parser = Parser::new();

    let mut result: Result<Option<Event>, Error> = Ok(None);
    for byte in subneg {
        result = parser.process_byte(byte);
    }

    assert_eq!(
        result,
        Ok(Some(Event::Subnegotiation(
            Subnegotiation::FlowControlSuspend
        )))
    );
}
