/*
Supreme Server
Accepts HTTP requests from Velovision iPhone app to control Velovision Rearview Raspberry Pi
*/
use std::thread;
use std::time::Duration;
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicI32, Ordering};

use tiny_http::{Server, Response};
use system_shutdown::shutdown;

mod gstreamer_monitor;
mod cpu_temp;
mod fuel_gauge;
mod led_control;

fn main() {
    let address = "0.0.0.0:8000";

    let server: Server = Server::http(address).unwrap();
    println!("Server started at {}", address);

    // LED gets controlled by whomever sent the last blinking instruction, consisting of:
    // bool: Enable LED at all, On duration (ms), Off duration (ms). Recommended to keep durations > 10ms.
    let (led_tx, led_rx) = mpsc::channel::<(bool, u64, u64)>(); 
    led_control::start_listener(led_rx);

    // Start battery state of charge checker thread
    let battery_soc: Arc<AtomicI32> = Arc::new(AtomicI32::new(200)); // unrealistic initial value to be able to know that it's being updated
    let battery_soc_clone = battery_soc.clone();

    let led_tx_clone = led_tx.clone();
    let _soc_writer = thread::spawn(move || {
        loop {
            let updated_soc = fuel_gauge::store_battery_soc(&battery_soc_clone);

            // Flash LED before shutting down due to low battery
            // We cannot rely on the battery management IC hardware voltage cutoff because the unstable voltage causes random reboots and boot loops when the load during boot causes voltage drop.
            let shutdown_soc = 5; //% 
            if updated_soc <= shutdown_soc {
                led_tx_clone.send((true, 25, 25)).unwrap();
                thread::sleep(Duration::from_millis(3000));
                led_tx_clone.send((false, 0, 0)).unwrap();
                match shutdown() {
                    Ok(_) => println!("Shutting down due to low battery."),
                    Err(error) => eprintln!("Low battery but failed to shut down: {}", error), 
                }
            }
            thread::sleep(Duration::from_millis(1000))
        }
    });

    for request in server.incoming_requests() {
        let mut response = Response::from_string("");

        let url = request.url();
        match request.method() {
            // GET: Idempotent data retrieval
            &tiny_http::Method::Get => {
                match url {
                    "/" => {
                        response = Response::from_string("Welcome to Velovision Rearview").with_status_code(200);
                    },
                    "/camera-stream-status" => {
                        response = Response::from_string( format!("{}", gstreamer_monitor::check_tcp_service(5000)) ).with_status_code(200)
                    },
                    "/battery-percent" => {
                        let v = battery_soc.load(Ordering::Relaxed);
                        response = Response::from_string(format!("{}", v)).with_status_code(200);
                    },
                    "/cpu-temp" => {
                        let temp = cpu_temp::read_cpu_temp("/sys/class/thermal/thermal_zone0/temp");
                        match temp {
                            Ok(temp) => { response = Response::from_string(temp).with_status_code(200) },
                            Err(_) => { response = Response::from_string("Failed to read CPU temperature").with_status_code(500) }
                        }
                    }
                    _ => {
                        eprintln!("Unknown GET request");
                        response = Response::from_string("Unknown GET request").with_status_code(501);
                    },
                }
            },
            // PUT: Idempotent data submission
            &tiny_http::Method::Put => {
                match url {
                   "/blink-on" => {
                        led_tx.send((true, 100, 1000)).unwrap();
                        response = Response::from_string("Turned on LED").with_status_code(200);
                    },
                    "/blink-off" => {
                        led_tx.send((false, 0, 0)).unwrap();
                        response = Response::from_string("Turned off LED").with_status_code(200);
                    },
                    "/camera-stream-on" => {
                        match systemctl::restart("camera-mjpeg-over-tcp.service") {
                            Ok(_) => { response = Response::from_string("Turned camera stream on").with_status_code(200); },
                            Err(_) => { response = Response::from_string("Failed to turn on camera stream").with_status_code(500); }
                        }
                    },
                    "/camera-stream-off" => {
                        match systemctl::stop("camera-mjpeg-over-tcp.service") {
                            Ok(_) => { response = Response::from_string("Turned camera stream off").with_status_code(200); },
                            Err(_) => { response = Response::from_string("Failed to turn off camera stream").with_status_code(500); }
                        }
                    },
                    _ => {
                        eprintln!("Unknown PUT request");
                        response = Response::from_string("Unknown PUT request").with_status_code(501);
                    },
                }
            }
            _ => () // other HTTP methods not implemented
        }
        let _ = request.respond(response);
    }
}
