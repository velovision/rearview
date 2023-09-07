use std::time::SystemTime;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Read, Cursor};
use std::time::Duration;
use std::process::Command;
use std::thread;
use std::sync::mpsc::{Sender, Receiver};


use tiny_http::Response;

pub fn start_streaming_mode(rx: Receiver<()>, led_tx_clone: Sender<(bool, u64, u64)>) {
    /*
    The channel accepts a unit object. When a unit object is received, streaming mode is re-started and waits for another minute for connection
    */
    thread::spawn(move || {
        loop {
            match rx.try_recv() {
                Ok(_signal) => {
                    // Streaming mode and wait a minute
                    led_tx_clone.send((true, 1200, 100)).unwrap(); // Majority on, short off = streaming mode
                    systemctl::disable("velovision-standalone-mode.service").unwrap(); // standalone mode does not start on boot by default
                    systemctl::stop("velovision-standalone-mode.service").unwrap(); // ensure camera isn't being used by standalone mode

                    systemctl::enable("velovision-camera-mjpeg-over-tcp.service").unwrap(); // streaming service starts on boot by default
                    systemctl::start("velovision-camera-mjpeg-over-tcp.service").unwrap();

                    thread::sleep(Duration::from_secs(60));
                    while rx.try_recv().is_ok() {} // ignore any messages received during the minute

                    // If client isn't connected, switch to standalone mode
                    if !is_client_connected("192.168.9.1", 5000) {
                        led_tx_clone.send((true, 100, 1200)).unwrap(); // Short on, majority off = standalone mode
                        systemctl::disable("velovision-camera-mjpeg-over-tcp.service").unwrap(); // streaming service does not start on boot by default
                        systemctl::stop("velovision-camera-mjpeg-over-tcp.service").unwrap(); // ensure camera isn't being used by streaming mode

                        systemctl::enable("velovision-standalone-mode.service").unwrap(); // standalone mode starts on boot by default
                        systemctl::start("velovision-standalone-mode.service").unwrap();
                    }
                }
                _ => {
                     // no message, revert to standalone mode unless client is connected
                    if !is_client_connected("192.168.9.1", 5000) {
                        led_tx_clone.send((true, 100, 1200)).unwrap(); // Short on, majority off = standalone mode
                        systemctl::disable("velovision-camera-mjpeg-over-tcp.service").unwrap(); // streaming service does not start on boot by default
                        systemctl::stop("velovision-camera-mjpeg-over-tcp.service").unwrap(); // ensure camera isn't being used by streaming mode

                        systemctl::enable("velovision-standalone-mode.service").unwrap(); // standalone mode starts on boot by default
                        systemctl::start("velovision-standalone-mode.service").unwrap();
                    }

                }            
            }
            thread::sleep(Duration::from_millis(100));
        }
    });
}

pub fn format_system_time_to_string(st: SystemTime) -> String {
    let duration_since_epoch = st.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let secs_since_epoch = duration_since_epoch.as_secs();

    let year = 1970 + (secs_since_epoch / (60 * 60 * 24 * 365));
    let month = 1 + ((secs_since_epoch / (60 * 60 * 24 * 30)) % 12);
    let day = 1 + ((secs_since_epoch / (60 * 60 * 24)) % 30);  // Simplified, months are treated as if all had 30 days
    let hour = (secs_since_epoch / (60 * 60)) % 24;
    let min = (secs_since_epoch / 60) % 60;
    let sec = secs_since_epoch % 60;

    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}", year, month, day, hour, min, sec)
}

pub fn files_sorted_by_date<P: AsRef<Path>>(path: P) -> io::Result<Vec<(PathBuf, SystemTime)>> {
    let mut entries: Vec<_> = fs::read_dir(path)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                Some((path, entry.metadata().ok()?.modified().unwrap_or(SystemTime::UNIX_EPOCH)))
            } else {
                None
            }
        })
        .collect();

    // Sort entries based on their last modified time
    entries.sort_by_key(|&(_, time)| time);

    Ok(entries)
}

pub fn yield_video_file(post_content: String) -> Response<Cursor<Vec<u8>>> {
    // validate that path in post_content exists
    let path = Path::new(&post_content);
    if !path.exists() {
        Response::from_string("Path does not exist").with_status_code(400);
    }

    // validate that path is .mkv video file
    let extension = path.extension().unwrap();
    if extension != "mkv" {
        return Response::from_string("Path is not a .mkv video file").with_status_code(400);
    }

    let mut file = fs::File::open(path).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    let header = tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"video/x-matroska"[..]).unwrap();

    Response::from_data(contents).with_header(header).with_status_code(200)

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