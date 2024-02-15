use clap::Parser;
use std::net::IpAddr;
use rfc2217_rs::Server;

#[derive(Parser, Debug)]
struct Args {
    #[clap(long = "serial_port", short = 'p', default_value = "/dev/ttyUSB0")]
    serial_port: String,
    #[clap(long = "address", short = 'a', default_value = "127.0.0.1")]
    address: IpAddr,
    #[clap(long = "tcp_port", default_value = "7878")]
    tcp_port: u16,
}

fn main() {
    let Args { address, tcp_port, serial_port } = Args::parse();

    let mut server = Server::new(&serial_port, (address, tcp_port)).unwrap();

    loop {
        server.run().unwrap();
    }
}
