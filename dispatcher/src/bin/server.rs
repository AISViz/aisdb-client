// https://bluejekyll.github.io/blog/posts/multicasting-in-rust/

extern crate socket2;

use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::thread::JoinHandle;

#[path = "../socket.rs"]
pub mod socket;
use socket::new_socket;

/// server: client socket handler
/// binds a new socket connection on the network multicast channel
fn join_multicast(addr: SocketAddr) -> io::Result<UdpSocket> {
    #[cfg(debug_assertions)]
    println!("server broadcasting to: {}", addr.ip());
    assert!(addr.ip().is_multicast());
    match addr.ip() {
        IpAddr::V4(ref mdns_v4) => {
            // join to the multicast address, with all interfaces
            let socket = new_socket(&addr)?;
            socket.join_multicast_v4(mdns_v4, &Ipv4Addr::new(0, 0, 0, 0))?;
            Ok(socket.into())
        }
        IpAddr::V6(ref mdns_v6) => {
            let ipv6_interface: u32 = 0;
            // IPv6 requires explicitly defining the host socket interface
            // this varies between hosts, and don't know how to check,
            // so try them all until one works
            //let result = socket.set_multicast_if_v6(ipv6_interface);
            /*
            loop {
                if ipv6_interface > 32 {
                    panic!("no suitable devices!");
                }
                #[cfg(debug_assertions)]
                println!(
                    "bind_socket: attempting to bind {} on ipv6 interface {}",
                    addr, ipv6_interface
                );
                */
            let socket = new_socket(&addr)?;
            socket.join_multicast_v6(mdns_v6, 0)?;
            socket.set_multicast_if_v6(ipv6_interface)?;
            Ok(socket.into())
            /*
            // join to the multicast address, with all interfaces (ipv6 uses indexes not addresses)
            //socket.set_only_v6(true)?;
                match bind_socket(&socket, &addr) {
                    Err(e) => match e.raw_os_error() {
                        Some(0) => {
                            ipv6_interface += 1;
                        }
                        _ => {
                            panic!("{}", e);
                        }
                    },
                    Ok(_) => {
                        //break;
                        //return Ok(socket.into());
                    }
                }
                */
            /*
            if let Err(e) = bind_socket(&socket, &addr) {
                match e.raw_os_error() {
                    Some(19) => {
                        ipv6_interface += 1;
                    }
                    _ => {
                        panic!("{}", e);
                    }
                }
            } else {
                return Ok(socket.into());
            }
            */
            //}
        }
    }
    //};
    //if let Err(e) = bind_socket(&socket, &addr) {
    //    panic!("failed to bind socket!\t{:?}", e)
    // }
    //Ok(socket.into())
}

pub fn join_unicast(addr: SocketAddr) -> io::Result<UdpSocket> {
    let socket = new_socket(&addr)?;
    socket.bind(&socket2::SockAddr::from(addr))?;
    Ok(socket.into())
}

/// server socket listener
pub fn listener(
    response: String,
    addr: SocketAddr,
    logfile: PathBuf,
    #[cfg(test)] downstream_done: Arc<AtomicBool>,
) -> JoinHandle<()> {
    // A barrier to not start the client test code until after the server is running
    let upstream_barrier = Arc::new(Barrier::new(2));
    let downstream_barrier = Arc::clone(&upstream_barrier);

    //let file = File::open(&logfile).unwrap_or_else(|_| panic!("opening {:?}", &logfile));
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&logfile);
    let mut writer = match file {
        Ok(f) => f,
        Err(e) => {
            panic!("{:?}: {}", &logfile, e);
        }
    };
    //let mut writer = BufWriter::with_capacity(1024, file);
    let listener = match addr.ip().is_multicast() {
        false => join_unicast(addr).expect("failed to create socket listener!"),
        true => {match join_multicast(addr) {
            Ok(s) => s,
            Err(e) => panic!("failed to create multicast listener on address {}! are you sure this is a valid multicast channel?\n{:?}", addr, e),
        }},
    };

    let join_handle = std::thread::Builder::new()
        .name(format!("{}:server", response))
        .spawn(move || {
            #[cfg(debug_assertions)]
            println!("{}:server: joined: {}", response, addr);

            #[cfg(test)]
            upstream_barrier.wait();

            #[cfg(debug_assertions)]
            println!("{}:server: is ready", response);

            //#[cfg(debug_assertions)]
            #[cfg(test)]
            println!(
                "{}:server: client complete {}",
                response,
                downstream_done.load(std::sync::atomic::Ordering::Relaxed)
            );

            // loop until the client indicates it is done
            let mut buf = [0u8; 1024]; // receive buffer
                                       /*
                                       //std::thread::spawn(move || {
                                           for sig in signals.forever() {
                                               println!("Received signal {:?}", sig);
                                               break
                                           }
                                       //});
                                       */
            loop {
                //while !downstream_done.load(std::sync::atomic::Ordering::Relaxed) {
                match listener.recv_from(&mut buf) {
                    Ok((c, _remote_addr)) => {
                        //let data = &buf[.._len];
                        //let _ = writer.write(data).unwrap_or_else(|_| panic!("writing to {:?}", &logfile));

                        /*
                        #[cfg(debug_assertions)]
                        println!(
                            "\n{}:server: got bytes: {} from: {}\nmsg: {}",
                            response,
                            c,
                            _remote_addr,
                            String::from_utf8_lossy(&buf[..c]),
                        );
                        */

                        let _ = writer
                            .write(&buf[..c])
                            .unwrap_or_else(|_| panic!("writing to {:?}", &logfile));
                        buf = [0u8; 1024];

                        /*
                        // create a socket to send the response
                        let responder =
                        new_socket(&remote_addr).expect("failed to create responder");

                        let remote_socket = SockAddr::from(remote_addr);

                        // we send the response that was set at the method beginning
                        responder
                        .send_to(response.as_bytes(), &remote_socket)
                        .expect("failed to respond");

                        #[cfg(debug_assertions)]
                        println!(
                        "{}:server: sent response {} to: {}",
                        response, response, _remote_addr
                        );
                        */
                    }
                    Err(err) => {
                        writer.flush().unwrap();
                        //file.flush().unwrap();
                        eprintln!("{}:server: got an error: {}", response, err);
                        #[cfg(debug_assertions)]
                        panic!("{}:server: got an error: {}", response, err);
                        //break
                    }
                }
            }

            /*
            #[cfg(debug_assertions)]
            println!(
                "{}:server: client complete {}",
                response,
                downstream_done.load(std::sync::atomic::Ordering::Relaxed)
            );

            println!("{}:server: client is done", response);

            writer.flush().unwrap();
            //file.flush().unwrap();
            */
        })
        .unwrap();

    downstream_barrier.wait();
    join_handle
}

/// ensure the server is stopped
pub struct NotifyServer(pub Arc<AtomicBool>);
impl Drop for NotifyServer {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

struct ServerArgs {
    client_addr: Vec<String>,
    path: String,
    port: u16,
}

fn parse_args() -> Result<ServerArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();
    /*
    if pargs.contains(["-h", "--help"]) || pargs.clone().finish().is_empty() {
    print!("{}", HELP);
    std::process::exit(0);
    }
    */
    let args = ServerArgs {
        port: pargs.value_from_str("--port")?,
        path: pargs.value_from_str("--path")?,
        client_addr: pargs.values_from_str("--client_addr")?,
        //pargs .opt_value_from_str("--listen_addr")? .unwrap_or("127.0.0.1".to_string()),
    };

    Ok(args)
}

pub fn main() {
    // todo: read args from command line
    //const PORT: u16 = 9923;
    // CIDR group 224 => multicast address range
    //let addr: IpAddr = Ipv4Addr::new(224, 0, 0, 110).into();
    //assert!(addr.is_multicast());

    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {}.", e);
            std::process::exit(1);
        }
    };

    for hostname in args.client_addr {
        let hostaddr = IpAddr::from_str(&hostname).unwrap();
        let socketaddr = SocketAddr::new(hostaddr, args.port);
        #[cfg(test)]
        let downstream_done = Arc::new(AtomicBool::new(false));

        listener(
            hostaddr.to_string(),
            socketaddr,
            PathBuf::from_str(&args.path).unwrap(),
            #[cfg(test)]
            downstream_done,
        );
    }

    //let addr: IpAddr = Ipv4Addr::new(127, 0, 0, 1).into();

    //pub static IPV4: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 110).into();
    //pub static IPV6: Ipv6Addr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0110).into();

    // start client listener
    //let _notify = NotifyServer(Arc::clone(&downstream_done));
    //multicast_listener("0", downstream_done, socketaddr);
}
