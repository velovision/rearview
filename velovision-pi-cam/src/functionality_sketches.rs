use std::error::Error;
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::net::TcpStream;

use rppal::gpio::Gpio;

use systemctl;

pub fn check_tcp_service(port: u16) -> bool {
    let addr = format!("localhost:{}", port);
    match TcpStream::connect(addr) {
        Ok(_) => true,
        Err(_) => false,
    }
}

struct CameraStreamStatus {
    systemd_active: bool,
    tcp_stream_exists: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let gpio = Gpio::new().unwrap();
    let mut pin = match gpio.get(21) {
        Ok(pin) => pin.into_output(),
        Err(e) => panic!("Error getting GPIO pin: {}", e),
    };

    println!("Setting pin to high");
    pin.set_high();
    thread::sleep(Duration::from_secs(1));
    println!("Setting pin to low");
    pin.set_low();
    thread::sleep(Duration::from_secs(1));
    pin.set_high();

    println!("Re-starting camera systemd service");
    systemctl::restart("camera-mjpeg-over-tcp.service").unwrap();

    let camera_stream_status = Arc::new(
        Mutex::new(
            CameraStreamStatus {
                systemd_active: false,
                tcp_stream_exists: false,
            }
        )
    );

    // just loop this active check
    loop {
        let is_active = systemctl::is_active("camera-mjpeg-over-tcp.service").unwrap();
        println!("Systemd service is active: {}", is_active);

        let tcp_service_exists = check_tcp_service(5000);
        println!("TCP stream exists: {}", tcp_service_exists);

        println!("\n");
        thread::sleep(Duration::from_secs(1));
    }
    // println!("Stopping service");
    // systemctl::stop("camera-mjpeg-over-tcp.service").unwrap();


    // let mut pin = gpio.get(23).unwrap().into_input_pullup();
    // while pin.is_high() {
    //     println!("High");
    //     thread::sleep(Duration::from_millis(100));
    // }
    // println!("Low")
    Ok(())
}

