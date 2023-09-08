use std::sync::atomic::{AtomicI32, AtomicBool, Ordering};
use std::sync::Arc;

use rppal::i2c::I2c;
use rppal::i2c::Error as I2cError;

const FUEL_GAUGE_ADDR: u16 = 0x36;
const SOC_REGISTER: u8 = 0x04;
const VCELL_REGISTER: u8 = 0x02;

pub struct BatteryStats {
    pub state_of_charge_percent: i32,
    pub cell_millivolts: i32,
}

fn get_battery_stats() -> Result<BatteryStats, I2cError> {
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
    //println!("State of Charge: {:.2}%", soc_percentage);

    // Read from VCELL register
    let mut vcell_buf = [0u8; 2];
    i2c.write_read(&[VCELL_REGISTER], &mut vcell_buf)?;
    // Datasheet for MAX17048 says VCELL has units: 78.125μV
    // We'll return it as mV
    let vcell_value = ((vcell_buf[0] as u16) << 8 | vcell_buf[1] as u16) as f32 * 78.125 / 1_000.0;
    //println!("VCELL: {:.2}", vcell_value);

    Ok(
        BatteryStats {
            state_of_charge_percent: soc_percentage.round() as i32,
            cell_millivolts: vcell_value.round() as i32,
        }
    )
}

pub fn store_battery_stats(
    atomic_soc: &Arc<(AtomicI32, AtomicBool)>,
    atomic_voltage: &Arc<(AtomicI32, AtomicBool)>,
    ) {
    let new_stats = get_battery_stats();
    match new_stats {
        Ok(stats) => {
            atomic_soc.0.store(stats.state_of_charge_percent, Ordering::Relaxed);
            atomic_soc.1.store(true, Ordering::Relaxed);

            atomic_voltage.0.store(stats.cell_millivolts, Ordering::Relaxed);
            atomic_voltage.1.store(true, Ordering::Relaxed);
        }
        Err(_) => { println!("Failed to fetch battery stats from I2C fuel gauge"); }
    }
}

