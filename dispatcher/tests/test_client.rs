use std::fs::{File, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

#[cfg(unix)]
use std::os::unix::net::UnixListener;

#[path = "../src/bin/client.rs"]
mod client;
use client::client_socket_stream;

#[path = "../src/bin/server.rs"]
mod server;
use server::listener;

fn truncate(path: PathBuf) -> i32 {
    let info = match File::open(&path) {
        Ok(f) => f.metadata().unwrap().len(),
        Err(e) => {
            eprintln!("{}", e);
            0
        }
    };

    File::create(&path).expect("creating file");
    sleep(Duration::from_millis(15));
    info.try_into().unwrap()
}

fn test_client(pathstr: &str, listen_addr: String, target_addr: String, tee: bool) {
    let bytesize = truncate(PathBuf::from_str(pathstr).unwrap());
    let _l = listener(listen_addr, PathBuf::from_str(pathstr).unwrap());
    let _c = client_socket_stream(&PathBuf::from(TESTDATA), vec![target_addr], tee);
    println!("log size: {}", bytesize);
    assert!(bytesize > 0);
}

//const TESTDATA: &str = "./tests/test_data_20211101.nm4";
//const TESTDATA: &str = "./tests/test_data_random.bin";
const TESTDATA: &str = "./readme.md";
const TESTINGDIR: &str = "./tests/";

#[test]
fn test_client_socket_stream_unicast_ipv4() {
    let pathstr = &[TESTINGDIR, "streamoutput_client_ipv4_unicast.log"].join(&"");
    let listen_addr = "0.0.0.0:9910".to_string();
    let target_addr = "127.0.0.1:9910".to_string();
    test_client(pathstr, listen_addr, target_addr, false)
}

#[test]
fn test_client_socket_stream_multicast_ipv4() {
    let pathstr = &[TESTINGDIR, "streamoutput_client_ipv4_multicast.log"].join(&"");
    let target_addr = "224.0.0.110:9911".to_string();
    let listen_addr = target_addr.clone();
    test_client(pathstr, listen_addr, target_addr, false)
}

#[test]
fn test_client_socket_stream_unicast_ipv6() {
    let pathstr = &[TESTINGDIR, "streamoutput_client_ipv6_unicast.log"].join(&"");
    let listen_addr = "[::1]:9912".to_string();
    let target_addr = "[::0]:9912".to_string();
    test_client(pathstr, listen_addr, target_addr, false)
}

#[test]
fn test_client_socket_stream_multicast_ipv6() {
    let pathstr = &[TESTINGDIR, "streamoutput_client_ipv6_multicast.log"].join(&"");
    let listen_addr = "[ff02::0110]:9913".to_string();
    let target_addr = "[ff02::0110]:9913".to_string();
    test_client(pathstr, listen_addr, target_addr, false)
}

#[test]
fn test_client_socket_tee() {
    let pathstr = &[TESTINGDIR, "streamoutput_client_tee.log"].join(&"");
    let target_addr = "127.0.0.1:9914".to_string();
    let listen_addr = "0.0.0.0:9914".to_string();
    test_client(pathstr, listen_addr, target_addr, true)
}

/*
#[test]
fn test_client_multiple_servers() {
    let pathstr = &[TESTINGDIR, "streamoutput_client_ipv6_unicast.log"].join(&"");
    let listen_addr_1 = "[::]:9915".to_string();
    let listen_addr_2 = "[::]:9916".to_string();

    let target_addr_1 = "[::1]:9915".to_string();
    let target_addr_2 = "[::1]:9916".to_string();
    //test_client(pathstr, listen_addr, target_addr, false)

    let bytesize = truncate(PathBuf::from_str(pathstr).unwrap());
    let _l = listener(listen_addr, PathBuf::from_str(pathstr).unwrap());
    let _c = client_socket_stream(PathBuf::from(TESTDATA), target_addr, tee);
    println!("log size: {}", bytesize);
    assert!(bytesize > 0);
}
*/
