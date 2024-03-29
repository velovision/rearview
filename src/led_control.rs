use std::sync::mpsc::{TryRecvError, Receiver};
use std::thread;
use std::time::Duration;
use std::cmp::max;

use rppal;
use rppal::gpio::{ Gpio, Trigger };

use system_shutdown::shutdown;

pub fn shutdown_at_pin() {
    /*
    Shut down deivce upon rising edge on pin 17
    Raspberry Pi is in responsible for shutting down the system.
    */
    thread::spawn( || {
        let gpio = Gpio::new().unwrap();
        let mut _pin = match gpio.get(17) {
            Ok(pin) => {
                let mut input_pin = pin.into_input_pulldown();
                input_pin.set_interrupt( Trigger::RisingEdge ).unwrap();
                loop {
                    if input_pin.is_high() {
                        match shutdown() {
                            Ok(_) => println!("Shutting down from button presss."),
                            Err(error) => eprintln!("Failed to shut down from button press: {}", error), 
                        }
                    }
                    thread::sleep(Duration::from_millis(100));
                }
            }
            Err(e) => {
                println!("Failed to get GPIO pin: {}, probably because it's in use by another thread, meaning GPIO is blinking", e);
                return;
            }
        };
    });
}

pub fn start_listener(rx: Receiver<(bool, u64, u64)>) {
    /*
    The channel accepts (bool, u64, u64), where
        bool: Whether LED should be on at all
        first u64: milliseconds LED is turn on, given bool is true
        second u64:  milliseconds LED is turned off, given bool is true
    */
    let gpio = Gpio::new().unwrap();
    let mut pin = match gpio.get(21) {
        Ok(pin) => pin.into_output(),
        Err(e) => {
            println!("Failed to get GPIO pin: {}, probably because it's in use by another thread, meaning GPIO is blinking", e);
            return;
        }
    };
    thread::spawn(move || {
        // let mut last_message = "No message received yet.".to_string(); // Default message
        let mut last_message = (false, 0, 0);


        loop {
            match rx.try_recv() {
                Ok(message) => {
                    last_message = message;
                    let (activate, on_ms, off_ms) = message;
                    if activate {
                        pin.set_high();
                        thread::sleep(Duration::from_millis(on_ms));
                        pin.set_low();
                        thread::sleep(Duration::from_millis(max(off_ms-10,1)));
                    } else {
                        thread::sleep(Duration::from_millis(100));
                    }
                }
                Err(TryRecvError::Empty) => {
                    let (activate, on_ms, off_ms) = last_message;
                    if activate {
                        pin.set_high();
                        thread::sleep(Duration::from_millis(on_ms));
                        pin.set_low();
                        thread::sleep(Duration::from_millis(max(off_ms-10,1)));
                    } else {
                        thread::sleep(Duration::from_millis(100));
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    println!("Sender has disconnected");
                    pin.set_low();
                    break;
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    });
}