use std::io::stdout;
use std::io::{BufWriter, Write};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
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
  gateway --listen_addr [SOCKET_ADDR] --downstream_addr [SOCKET_ADDR] ...

  e.g.
  gateway --listen_addr 0.0.0.0:9920 --downstream_addr [::1]:9921 --tee

FLAGS:
  -h, --help    Prints help information
  -t, --tee     Copy input to stdout

"#;

pub struct GatewayArgs {
    downstream_addrs: Vec<String>,
    listen_addrs: Vec<String>,
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
        listen_addrs: pargs.values_from_str("--listen_addr")?,
        downstream_addrs: pargs.values_from_str("--downstream_addr")?,
        tee,
    };

    Ok(args)
}

pub fn proxy_thread(
    listen_socket: UdpSocket,
    downstream_addrs: &Vec<String>,
    tee: bool,
) -> JoinHandle<()> {
    let mut output_buffer = BufWriter::new(stdout());
    let targets: Vec<(SocketAddr, UdpSocket)> = downstream_addrs
        .into_iter()
        .map(|a| {
            let addr = a
                .to_socket_addrs()
                .unwrap()
                .next()
                .expect("parsing address");
            (
                addr,
                match addr.is_ipv4() {
                    true => new_sender(&addr).expect("ipv4 output socket"),
                    false => client_check_ipv6_interfaces(&addr).expect("ipv6 output socket"),
                },
            )
        })
        .collect();
    let mut buf = [0u8; 1024]; // receive buffer
    Builder::new()
        .name(format!("{:#?}", listen_socket))
        .spawn(move || {
            //listen_socket.read_timeout().unwrap();
            listen_socket.set_broadcast(true).unwrap();
            loop {
                match listen_socket.recv_from(&mut buf[0..]) {
                    Ok((c, _remote_addr)) => {
                        for (target_addr, target_socket) in &targets {
                            target_socket
                                .send_to(&buf[0..c], &target_addr)
                                .expect("sending to server socket");
                        }
                        if tee {
                            let o = output_buffer
                                .write(&buf[0..c])
                                .expect("writing to output buffer");
                            #[cfg(debug_assertions)]
                            assert!(c == o);
                        }
                    }
                    Err(err) => {
                        //output_buffer.flush().unwrap();
                        eprintln!("proxy_thread: got an error: {}", err);
                        #[cfg(debug_assertions)]
                        panic!("proxy_thread: got an error: {}", err);
                    }
                }
                output_buffer.flush().unwrap();
            }
        })
        .unwrap()
}

pub fn gateway(downstream_addrs: &Vec<String>, listen_addrs: &Vec<String>, tee: bool) {
    let mut threads = vec![];
    for listen_addr in listen_addrs {
        let addr = listen_addr
            .to_socket_addrs()
            .unwrap()
            .next()
            .expect("parsing socket address");
        let listen_socket = match addr.ip().is_multicast() {
            false => join_unicast(addr).expect("failed to create socket listener!"),
            true => {match join_multicast(addr) {
                Ok(s) => s,
                Err(e) => panic!("failed to create multicast listener on address {}! are you sure this is a valid multicast channel?\n{:?}", addr, e),
            }
            },
        };
        #[cfg(debug_assertions)]
        println!(
            "proxy: forwarding {:?} -> {:?}",
            listen_socket, downstream_addrs
        );
        threads.push(proxy_thread(listen_socket, downstream_addrs, tee));
    }
    for thread in threads {
        thread.join().unwrap();
    }
}

pub fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {}.", e);
            exit(1);
        }
    };

    let mut threads = vec![];

    let _ = threads.push(gateway(
        &args.downstream_addrs,
        &args.listen_addrs,
        args.tee,
    ));
}
