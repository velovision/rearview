/*
Supreme Server
Accepts HTTP requests from Velovision iPhone app to control Velovision Rearview Raspberry Pi
*/
use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex};

use tiny_http::{Server, Response};

mod gpio_control;
mod gstreamer_monitor;
mod cpu_temp;
mod fuel_gauge;

fn main() {
    let address = "0.0.0.0:8000";

    let server: Server = Server::http(address).unwrap();
    println!("Server started at {}", address);

    // Idempotent control of GPIO pin output blink on another thread
    let (blink_tx, blink_rx): (Sender<()>, Receiver<()>) = mpsc::channel();
    let blink_rx = Arc::new(Mutex::new(blink_rx));

    for mut request in server.incoming_requests() {
        let mut response = Response::from_string("");

        let url = request.url();
        match request.method() {
            // GET: Idempotent data retrieval
            &tiny_http::Method::Get => {
                match url {
                    // `curl http://192.168.9.1:8000/`
                    "/" => {
                        response = Response::from_string("Welcome to Velovision Rearview").with_status_code(200);
                    },
                    // `curl http://192.168.9.1:8000/camera-stream-status`
                    "/camera-stream-status" => {
                        response = Response::from_string( format!("{}", gstreamer_monitor::check_tcp_service(5000)) ).with_status_code(200)
                    },
                    "/battery-percent" => {
                        let battery_percent = fuel_gauge::get_battery_soc();
                        match battery_percent {
                            Ok(battery_percent) => { response = Response::from_string(format!("{:.2}", battery_percent)).with_status_code(200); }
                            Err(_) => { response = Response::from_string("Failed to read battery percent").with_status_code(500) }
                        }
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
                    // `curl -X PUT http://192.168.9.1:8000/blink-on`
                    "/blink-on" => {
                        let blink_rx_clone = Arc::clone(&blink_rx);
                        thread::spawn(move || { gpio_control::blink(blink_rx_clone); });
                        response = Response::from_string("Turned on LED").with_status_code(200);
                    },
                    "/blink-off" => {
                        blink_tx.send(()).unwrap();
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

        // POST request example
        // Test with: curl -X POST -d "jason" 192.168.9.1:8000
        if request.method() == &tiny_http::Method::Post {
            // Get post content
            let mut post_content = String::new();
            request.as_reader().read_to_string(&mut post_content).unwrap();
            println!("POST content: {}", post_content);

            let reply_content = format!("hello {}", post_content);

            response = Response::from_string(reply_content);
        }
        let _ = request.respond(response);
    }
}
