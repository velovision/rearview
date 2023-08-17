use std::sync::atomic::{AtomicI32, Ordering};

use rppal;
use rppal::i2c::I2c;
use rppal::i2c::Error as I2cError;

const FUEL_GAUGE_ADDR: u16 = 0x36;
const SOC_REGISTER: u8 = 0x04;

fn get_battery_soc() -> Result<i32, I2cError> {
    /*
    Do not call from more than one location because this function requires access to i2c bus
    This function is used by an updater thread to store the latest battery state of charge to an atomic variable.
    Load from that atomic variable instead of calling this function.
    */
    let mut i2c = I2c::new()?;
    i2c.set_slave_address(FUEL_GAUGE_ADDR)?;

    // Read two bytes (upper and lower bytes) from the SOC register
    let mut buf = [0u8; 2];
    i2c.write_read(&[SOC_REGISTER], &mut buf)?;

    let soc_percentage = ((buf[0] as u16) << 8 | buf[1] as u16) as f32 / 256.0;
    // println!("State of Charge: {:.2}%", soc_percentage);
    let soc_percentage_int: i32 = soc_percentage.round() as i32;

    return Ok(soc_percentage_int);
}

pub fn store_battery_soc(
    value: &AtomicI32, 
) -> i32 {
    let new_soc : i32 = get_battery_soc().expect("Failed to read battery state of charge from I2C fuel gauge");
    value.store(new_soc , Ordering::Relaxed);

    return new_soc;
    // if shutdown_if_low {
    //     if new_soc < shutdown_soc {
    //         // // blink rapidly to signify low battery
    //         // blink(blink_rx, 100, 100);
    //         // thread::sleep(Duration::from_millis(2000));
    //         // blink_tx.send(()).unwrap();

    //         // shut down system
    //         match shutdown() {
    //             Ok(_) => println!("Shutting down due to low battery."),
    //             Err(error) => eprintln!("Low battery but failed to shut down: {}", error),
    //         }

    //     }
}