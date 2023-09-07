use std::net::TcpStream;

// check that gstreamer is up and publishing a TCP stream
pub fn check_tcp_service(port: u16) -> bool {
    let addr = format!("localhost:{}", port);
    TcpStream::connect(addr).is_ok()
}

