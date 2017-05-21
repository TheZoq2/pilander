use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

pub fn read_i16_i2c(i2c_device: &mut LinuxI2CDevice, memory_address: u8)
        -> Result<i16, LinuxI2CError>
{
    try!(i2c_device.write(&[memory_address]));
    let msb = try!(i2c_device.smbus_read_byte());
    let lsb = try!(i2c_device.smbus_read_byte());

    Ok((((msb as u16)<<8) | lsb as u16) as i16)
}

pub fn read_u16_i2c(i2c_device: &mut LinuxI2CDevice, memory_address: u8) -> Result<u16, LinuxI2CError>
{
    try!(i2c_device.write(&[memory_address]));
    let msb = try!(i2c_device.smbus_read_byte());
    let lsb = try!(i2c_device.smbus_read_byte());

    Ok(((msb as u16)<<8) | lsb as u16)
}

