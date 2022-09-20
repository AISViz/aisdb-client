use std::io::stdout;
use std::io::{BufWriter, Write};
use std::net::SocketAddr;
use std::process::exit;
use std::thread::{Builder, JoinHandle};

#[path = "./client.rs"]
pub mod client;
use client::{client_check_ipv6_interfaces, new_sender};

#[path = "./server.rs"]
pub mod server;
use server::{join_multicast, join_unicast};

const HELP: &str = r#"
DISPATCH: GATEWAY

USAGE:
  gateway --listen_addr [SOCKET_ADDR] --server_addr [SOCKET_ADDR] --server_addr [...]

  e.g.
  gateway --listen_addr 0.0.0.0:9920 --server_addr [::1]:9921

FLAGS:
  -h, --help    Prints help information
  -t, --tee     Copy input to stdout

"#;

struct GatewayArgs {
    server_addrs: Vec<String>,
    listen_addr: String,
    tee: bool,
}

fn parse_args() -> Result<GatewayArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();
    if pargs.contains(["-h", "--help"]) || pargs.clone().finish().is_empty() {
        print!("{}", HELP);
        exit(0);
    }
    let tee = pargs.contains(["-t", "--tee"]);

    let args = GatewayArgs {
        listen_addr: pargs.value_from_str("--listen_addr")?,
        server_addrs: pargs.values_from_str("--server_addr")?,
        tee,
    };

    Ok(args)
}

pub fn gateway(server_addrs: Vec<String>, listen_addr: String, tee: bool) -> JoinHandle<()> {
    let mut targets = vec![];

    for server_addr in server_addrs {
        let target_addr: SocketAddr = server_addr.parse().expect("parsing server address");
        let target_socket = match target_addr.is_ipv4() {
            true => new_sender(&target_addr).expect("creating ipv4 send socket!"),
            false => {
                client_check_ipv6_interfaces(&target_addr).expect("creating ipv6 send socket!")
            }
        };
        targets.push((target_addr, target_socket));
        println!("logging {}: listening for", server_addr,);
    }

    let addr: SocketAddr = listen_addr
        .parse()
        .unwrap_or_else(|e| panic!("parsing socket address '{}' {}", listen_addr, e));

    let listen_socket = match addr.ip().is_multicast() {
        false => join_unicast(addr).expect("failed to create socket listener!"),
        true => {match join_multicast(addr) {
            Ok(s) => s,
            Err(e) => panic!("failed to create multicast listener on address {}! are you sure this is a valid multicast channel?\n{:?}", addr, e),
        }},
    };

    let mut output_buffer = BufWriter::new(stdout());

    Builder::new()
        .name(format!("{}", addr))
        .spawn(move || {
            let mut buf = [0u8; 1024]; // receive buffer
            loop {
                match listen_socket.recv_from(&mut buf[0..]) {
                    Ok((c, _remote_addr)) => {
                        for (target_addr, target_socket) in &targets {
                            target_socket
                                .send_to(&buf[0..c], &target_addr)
                                .expect("sending to server socket");
                            if tee {
                                let o = output_buffer
                                    .write(&buf[0..c])
                                    .expect("writing to output buffer");
                                output_buffer.flush().unwrap();
                                assert!(c == o);
                            }
                        }
                    }
                    Err(err) => {
                        output_buffer.flush().unwrap();
                        eprintln!("{}:server: got an error: {}", addr, err);
                        #[cfg(debug_assertions)]
                        panic!("{}:server: got an error: {}", addr, err);
                    }
                }
            }
        })
        .unwrap()
        .join()
        .unwrap()
}

pub fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {}.", e);
            exit(1);
        }
    };
    let _ = gateway(args.server_addrs, args.listen_addr, args.tee);
}
