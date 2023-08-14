use std::thread;
use std::time::Duration;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

use rppal::gpio::Gpio;

pub fn blink(rx: Arc<Mutex<Receiver<()>>>) {
    // Idempotent control of GPIO pin output blink
    // clear the channel of all messages
    let r = rx.lock().unwrap();
    while r.try_recv().is_ok() {}
    drop(r);

    let gpio = Gpio::new().unwrap();
    let mut pin = match gpio.get(21) {
        Ok(pin) => pin.into_output(),
        Err(e) => {
            print!("Failed to get GPIO pin: {}, probably because it's in use by another thread, meaning GPIO is blinking", e);
            return;
        }
    };

    loop {
        let r = rx.lock().unwrap();
        if r.try_recv().is_ok() {
            break;
        }
        drop(r);  // Release the lock manually (this is optional as Rust would automatically drop the lock at the end of the loop iteration)
    
        pin.set_high();
        thread::sleep(Duration::from_millis(100));
        pin.set_low();
        thread::sleep(Duration::from_millis(1000));
    }
}