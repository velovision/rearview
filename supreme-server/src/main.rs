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
use gstreamer_monitor::check_tcp_service;

fn main() {
    let address = "0.0.0.0:8000";

    let server: Server = Server::http(address).unwrap();
    println!("Server started at {}", address);

    // Idempotent control of GPIO pin output blink on another thread
    let (blink_tx, blink_rx): (Sender<()>, Receiver<()>) = mpsc::channel();
    let blink_rx = Arc::new(Mutex::new(blink_rx));

    for mut request in server.incoming_requests() {
        let mut response = Response::from_string("");

        if request.method() == &tiny_http::Method::Get {
            // Get url
            let url = request.url();
            // use switch case on URL
            match url {
                "/" => {
                    println!("Root request");
                    response = Response::from_string("hello root");
                },
                // Test with `curl http://192.168.9.1:8000/blink-on`
                "/blink-on" => {
                    let blink_rx_clone = Arc::clone(&blink_rx);
                    response = Response::from_string("Turning on LED");
                
                    thread::spawn(move || {
                        gpio_control::blink(blink_rx_clone);
                    });
                },
                // Test with `curl http://192.168.9.1:8000/blink-off`
                "/blink-off" => {
                    response = Response::from_string("Turning off LED");
                
                    blink_tx.send(()).unwrap();
                },
                "/camera-stream-on" => {
                    response = Response::from_string("Turning on camera stream");
                    systemctl::restart("camera-mjpeg-over-tcp.service").unwrap();
                },
                "/camera-stream-off" => {
                    response = Response::from_string("Turning off camera stream");
                    systemctl::stop("camera-mjpeg-over-tcp.service").unwrap();
                },
                "/tcp-stream-status" => {
                    let tcp_service_exists = check_tcp_service(5000);
                    let reply_content = format!("{}", tcp_service_exists);
                    response = Response::from_string(reply_content);
                },
                _ => {
                    println!("Some other request");
                    response = Response::from_string("hello non-root");
                },
            }
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
