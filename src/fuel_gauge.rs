use rppal;
use rppal::i2c::I2c;
use rppal::i2c::Error as I2cError;

const FUEL_GAUGE_ADDR: u16 = 0x36;
const SOC_REGISTER: u8 = 0x04;

pub fn get_battery_soc() -> Result<f32, I2cError> {
    let mut i2c = I2c::new()?;
    i2c.set_slave_address(FUEL_GAUGE_ADDR)?;

    // Read two bytes (upper and lower bytes) from the SOC register
    let mut buf = [0u8; 2];
    i2c.write_read(&[SOC_REGISTER], &mut buf)?;

    let soc_percentage = ((buf[0] as u16) << 8 | buf[1] as u16) as f32 / 256.0;
    // println!("State of Charge: {:.2}%", soc_percentage);

    return Ok(soc_percentage);
}