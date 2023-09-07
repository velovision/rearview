use std::net::TcpStream;
use std::process::Command;

// check that gstreamer is up and publishing a TCP stream
pub fn check_tcp_service(port: u16) -> bool {
    let addr = format!("localhost:{}", port);
    match TcpStream::connect(addr) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn is_client_connected(ip: &str, port: u16) -> bool {
    /* 
    Run the netstat command to retrieve active connections

    Used to detect if a client is connected to the video stream port at 5000,
    and if not, standalone mode is started.
    */
    let output = Command::new("netstat")
        .arg("-an")
        .output()
        .expect("Failed to run netstat command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Search for lines that represent established connections to the specified port
    for line in stdout.lines() {
        if line.contains(ip) && line.contains(&format!(":{} ", port)) && line.contains("ESTABLISHED") {
            return true;
        }
    }

    false
}