# rfc2217-rs

rfc2217-rs is an [IETF RFC2217](https://www.rfc-editor.org/rfc/rfc2217.html) implementation in rust, enabling Com Port functionality over TCP/IP connections.

# What is RFC2217

[IETF RFC2217](https://www.rfc-editor.org/rfc/rfc2217.html) defines a [Telnet](https://www.rfc-editor.org/rfc/rfc854.txt) extension providing serial port functionality. It was initially intended to be used with modems, serial printers, fax machines and similar equipment, but it provides the ability to connect any serial based device to the network.

# Library features
This library provides a server implementation, a protocol parser and data structures with binary serialization/deserialization support.

The library is ```std``` and ```no_std``` compatible, however the server implementation is only available in the ```std``` mode. To use the library in ```no_std``` mode, set the ```std``` feature to ```false```.

# How to use
* Using the Server
```rust
use rfc2217_rs::Server;
// --snip--
let mut server = Server::new("/dev/ttyUSB1", "127.0.0.1:7878").unwrap();
// --snip--
loop {
    let result = server.run();
    if let Err(error) == result {
        handle_err(error);
    }
}
```
* Using the Parser
```rust
use rfc2217_rs::Parser;
use rfc2217_rs::parser::Event;
// --snip--
let mut parser = Parser::new();
// --snip--
loop {
    // --snip--
    let result = parser.process_byte(byte);
    match result {
        Ok(event) => handle_event(event),
        Err(error) => handle_error(error),
    }
}
```
* Using the data structure serialization/deserialization
```rust
use rfc2217_rs::Command;
// --snip--
let command_buf = [0; command::SIZE];
let command = Command::NoOp;
command.serialize(&mut command_buf);
let deserialized_command = Command::deserialize(&command_buf);
```

# How it works
![diagram](diagram_light.svg#gh-light-mode-only)
![diagram](diagram_dark.svg#gh-dark-mode-only)

# License
Licensed under either of:

* Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
at your option.

# Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
