use std::path::Path;
use std::io;

fn main() {
    // This will execute every time you run `cargo build` or `cargo run`
    check_installation().expect("Installation check failed. Please run `sudo ./install.sh dev or sudo ./install.sh prod` to configure installation");

    println!("cargo:warning=To run the program, use: 'sudo target/debug/supreme-server' (because the program interacts with systemd)");
}

fn check_installation() -> Result<(), io::Error> {
    /*
    This program requires the following directories and files to exist.
    An installation script must configure these directories and files.
    This function merely checks that they exist.

    /opt/velovision
        ├── supreme-server // this executable binary
        └── standalone_videos // velovision-standalone-mode.service records videos to this directory
            ├── log0000.mkv // example video files
            ├── log0001.mkv
            └── log0002.mkv

    /etc/systemd/system
        ├── velovision-supreme-server.service // Runs this HTTP server at port 8000. Do not enable at development time because we run the program with `cargo run` instead of `sudo systemctl start velovision-supreme-server.service`
        ├── velovision-camera-mjpeg-over-tcp.service // Runs gstreamer to stream camera over TCP at port 5000
        └── velovision-standalone-mode.service // Runs gstreamer to record to local disk, which records videos to /opt/velovision/standalone_videos. 
    */
    let mut path = Path::new("/opt/velovision/standalone_videos");
    // raise error if path does not exist
    print!("Checking if directory {} exists...", path.display());
    if !path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("Directory {} does not exist", path.display())));
    }

    path = Path::new("/etc/systemd/system/velovision-supreme-server.service");
    if !path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("File {} does not exist", path.display())));
    }

    path = Path::new("/etc/systemd/system/velovision-camera-mjpeg-over-tcp.service");
    if !path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("File {} does not exist", path.display())));
    }

    path = Path::new("/etc/systemd/system/velovision-standalone-mode.service");
    if !path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("File {} does not exist", path.display())));
    }

    Ok(())
}