/*
Supreme Server
Accepts HTTP requests from Velovision iPhone app to control Velovision Rearview Raspberry Pi
*/
use std::time::Duration;
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicI32, AtomicBool, Ordering};
use std::thread;
use std::path::Path;
use std::io;

use tiny_http::{Server, Response};
use system_shutdown::shutdown;
use serde_json::json;

mod tcp_stream_monitor;
mod cpu_temp;
mod fuel_gauge;
mod led_control;
mod standalone_filesystem;

fn main() {
    check_installation().expect("Some required directories or files are missing. See src/main.rs:check_installation for details.");

    let address = "0.0.0.0:8000";

    let server: Server = Server::http(address).unwrap();
    println!("Server started at {}", address);

    // LED gets controlled by whomever sent the last blinking instruction, consisting of:
    // bool: Enable LED at all, On duration (ms), Off duration (ms). Recommended to keep durations > 10ms.
    let (led_tx, led_rx) = mpsc::channel::<(bool, u64, u64)>(); 
    led_control::start_listener(led_rx);

    // Atomic lacks float, so we will round the state of charge (soc) to the nearest percent
    // Atomic also lacks Result, so the AtomicBool signifies sucess or failure
    let battery_soc: Arc<(AtomicI32, AtomicBool)> = Arc::new((AtomicI32::new(100), AtomicBool::new(false)));
    let battery_soc_clone = battery_soc.clone();
    // and multiply voltage by 1000, so that float 3.82 (Volts) will be int 3820 (milliVolts).
    let battery_voltage: Arc<(AtomicI32, AtomicBool)> = Arc::new((AtomicI32::new(4000), AtomicBool::new(false))); 

    let battery_voltage_clone = battery_voltage.clone();
    let led_tx_clone = led_tx.clone();
    let _battery_checker = thread::spawn(move || {
        loop {
            fuel_gauge::store_battery_stats(&battery_soc_clone, &battery_voltage_clone);

            // Flash LED before shutting down due to low battery as determined by cell voltage
            // Hardware cutoff is at 3.0V. We shut down at 3.4V to allow for typical 0.3V sag at
            // boot.
            let shutdown_millivolts = 3400;

            let latest_millivolts = battery_voltage_clone.0.load(Ordering::Relaxed);
            let battery_voltage_success = battery_voltage_clone.1.load(Ordering::Relaxed);

            if latest_millivolts <= shutdown_millivolts && !battery_voltage_success {
                led_tx_clone.send((false, 0, 0)).unwrap();
                led_tx_clone.send((true, 50, 50)).unwrap();
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

    let led_tx_clone = led_tx.clone();
    let (restart_streaming_toggle_tx, restart_streaming_toggle_rx) = mpsc::channel::<()>(); 
    standalone_filesystem::start_streaming_mode(restart_streaming_toggle_rx, led_tx_clone);

    restart_streaming_toggle_tx.send(()).unwrap(); // send a signal to start streaming mode immediately after boot.
    // At any later time, send a signal through the same channel TX to put device into streaming mode and wait for a minute for a connection.
    // The device will always want to revert back to standalone mode if no connection is made.

    for mut request in server.incoming_requests() {
        let mut response = Response::from_string("");

        let url = request.url();
        match *request.method() {
            // GET: Idempotent data retrieval
            tiny_http::Method::Get => {
                match url {
                    "/" => {
                        response = Response::from_string("Welcome to Velovision Rearview").with_status_code(200);
                    },
                    "/camera-stream-status" => {
                        response = Response::from_string( format!("{}", tcp_stream_monitor::check_tcp_service(5000)) ).with_status_code(200)
                    },
                    "/battery-percent" => {
                        let latest_soc = battery_soc.0.load(Ordering::Relaxed);
                        let battery_soc_success = battery_soc.1.load(Ordering::Relaxed);

                        if battery_soc_success {
                            response = Response::from_string(format!("{}", latest_soc)).with_status_code(200);
                        } else {
                            response = Response::from_string("Failed to get battery state of charge").with_status_code(500);
                        }
                    },
                    "/battery-millivolts" => {
                        let latest_millivolts = battery_voltage.0.load(Ordering::Relaxed);
                        let battery_voltage_success = battery_voltage.1.load(Ordering::Relaxed);

                        if battery_voltage_success {
                            response = Response::from_string(format!("{}", latest_millivolts)).with_status_code(200);
                        } else {
                            response = Response::from_string("Failed to get battery voltage").with_status_code(500);
                        }
                    },
                    "/cpu-temp" => {
                        let temp = cpu_temp::read_cpu_temp("/sys/class/thermal/thermal_zone0/temp");
                        match temp {
                            Ok(temp) => { response = Response::from_string(temp).with_status_code(200) },
                            Err(_) => { response = Response::from_string("Failed to read CPU temperature").with_status_code(500) }
                        }
                    }
                    "/list-local-videos" => {
                        /* Returns JSON of absolute path of videos and their dates, sorted old -> new 
                        Example format:
                        [
                            {
                                "path": "/opt/standalone_mode/videos/loop0001.mkv",
                                "date_updated":"2023-06-17T09:13:00"
                            },
                            ...
                        ]

                        Use the path in a POST request to /download-video to download the video file
                        */
                        let path = "/opt/velovision/standalone_videos";
                        let sorted_files = standalone_filesystem::files_sorted_by_date(path).unwrap();
                        let json_list: Vec<_> = sorted_files.into_iter().map(|(path, date)| {
                            let date_str = standalone_filesystem::format_system_time_to_string(date);
                            json!({
                                "path": path.to_str().unwrap_or(""),
                                "date_updated": date_str
                            })
                        }).collect();
                        let json_string = serde_json::to_string(&json_list).unwrap();
                        response = Response::from_string(json_string);
                    }
                    
                    _ => {
                        eprintln!("Unknown GET request");
                        response = Response::from_string("Unknown GET request").with_status_code(501);
                    },
                }
            },
            // PUT: Idempotent data submission
            tiny_http::Method::Put => {
                match url {
                   "/blink-on" => {
                        led_tx.send((true, 100, 1000)).unwrap();
                        response = Response::from_string("Turned on LED").with_status_code(200);
                    },
                    "/blink-off" => {
                        led_tx.send((false, 0, 0)).unwrap();
                        response = Response::from_string("Turned off LED").with_status_code(200);
                    },
                    "/restart-stream-mode" => {
                        restart_streaming_toggle_tx.send(()).unwrap();
                        response = Response::from_string("Restarted streaming mode").with_status_code(200);
                    },
                    _ => {
                        eprintln!("Unknown PUT request");
                        response = Response::from_string("Unknown PUT request").with_status_code(501);
                    },
                }
            },
            tiny_http::Method::Post=> {
                match url {
                    "/download-video" => {
                        /*
                        Example usage:
                        curl -X POST -o DOWNLOAD_AS_NAME.mkv -d "/PATH/TO/VIDEO/ON/PI.mkv" http://192.168.9.1:8000/download-video

                        Get path to video (/PATH/TO/VIDEO/ON/PI.mkv) from GET /list-local-videos.
                        Recommended to use the date_updated field from the same GET request to rename downloaded video (DOWNLOAD_AS_NAME)
                        */
                        let mut post_content = String::new();
                        request.as_reader().read_to_string(&mut post_content).unwrap();
                        // println!("POST content: {}", post_content);

                        response = standalone_filesystem::yield_video_file(post_content)
                    },
                    _ => {
                        eprintln!("Unknown POST request");
                        response = Response::from_string("Unknown POST request").with_status_code(501);
                    },
                }       
            },
            _ => () // other HTTP methods not implemented
        }
        let _ = request.respond(response);
    }
}

fn check_installation() -> Result<(), io::Error> {
    /*
    This program requires the following directories and files to exist.
    An installation script must configure these directories and files.
    This function merely checks that they exist.

    /opt/velovision
        ├── supreme-server // this executable binary. Not required in development because we use `cargo run` instead of `sudo systemctl start velovision-supreme-server.service`
        ├── scripts
            └── standalone_gstreamer.sh // Offloaded the standalone mode gstreamer pipeline logic to an external script.
        └── standalone_videos // velovision-standalone-mode.service records videos to this directory
            ├── log0000.mkv // example video files
            ├── log0001.mkv
            └── log0002.mkv

    /etc/systemd/system
        ├── velovision-supreme-server.service // Runs this HTTP server at port 8000. Do not enable at development time because we run the program with `cargo run` instead of `sudo systemctl start velovision-supreme-server.service`
        ├── velovision-camera-mjpeg-over-tcp.service // Runs gstreamer to stream camera over TCP at port 5000
        └── velovision-standalone-mode.service // Runs gstreamer to record to local disk, which records videos to /opt/velovision/standalone_videos. 
    */
    let path = Path::new("/opt/velovision/standalone_videos");
    // raise error if path does not exist
    if !path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("Directory {} does not exist", path.display())));
    }

    let files_to_check = [
        "/opt/velovision/scripts/standalone_gstreamer.sh",
        "/etc/systemd/system/velovision-supreme-server.service",
        "/etc/systemd/system/velovision-camera-mjpeg-over-tcp.service",
        "/etc/systemd/system/velovision-standalone-mode.service"
    ];

    for path_str in &files_to_check {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, format!("File {} does not exist", path.display())));
        }
    }

    Ok(())
}
