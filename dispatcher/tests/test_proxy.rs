use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::path::PathBuf;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

#[path = "../src/bin/proxy.rs"]
pub mod proxy;
use proxy::{gateway, proxy_thread, GatewayArgs};

#[path = "../src/bin/server.rs"]
pub mod server;
use server::{join_multicast, join_unicast, listener};

#[path = "../src/bin/client.rs"]
pub mod client;
use client::{client_check_ipv6_interfaces, client_socket_stream, new_sender};

#[path = "./test_client.rs"]
pub mod test_client;
use test_client::{truncate, TESTDATA, TESTINGDIR};

#[test]
fn test_proxy_thread_ipv4() {
    let client_target = "127.0.0.1:8890".to_string();
    let proxy_listen = "0.0.0.0:8890".to_string();
    let proxy_target = "127.0.0.1:8891".to_string();
    //let proxy_target = "0.0.0.0:8891".to_string();
    let server_listen = "0.0.0.0:8891".to_string();

    let data = PathBuf::from(TESTDATA);
    let pathstr = &[TESTINGDIR, "streamoutput_proxy_thread_ipv4_output.log"].join(&"");
    let output = PathBuf::from(pathstr);
    assert!(data.is_file());

    let _l = listener(server_listen, output);
    sleep(Duration::from_millis(15));

    let listen_socket_addr: SocketAddr = proxy_listen.to_socket_addrs().unwrap().next().unwrap();
    let listen_socket = join_unicast(listen_socket_addr).expect("creating socket");
    let targets = vec![proxy_target];
    //let _g = gateway(&vec![], &proxy_listen, true);
    let _p = proxy_thread(listen_socket, &targets, false);
    sleep(Duration::from_millis(15));

    let _c = client_socket_stream(&data, vec![client_target], false);

    let output = PathBuf::from(pathstr);
    let bytesize = truncate(output);
    assert!(bytesize > 0);
}
