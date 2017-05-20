extern crate i2cdev;

use std::thread;
use std::time::Duration;

use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

fn read_byte_i2c(i2c_device: &mut LinuxI2CDevice,  memory_address: u8)
        -> Result<u8, LinuxI2CError>
{
    i2c_device.smbus_read_byte_data(memory_address)
}

fn read_i16_i2c(i2c_device: &mut LinuxI2CDevice, memory_address: u8)
        -> Result<i16, LinuxI2CError>
{
    let msb = try!(i2c_device.smbus_read_byte_data(memory_address));
    let lsb = try!(i2c_device.smbus_read_byte_data(memory_address+1));

    Ok((msb as i16)<<8 | lsb as i16)
}

struct Bmp085
{
    device: LinuxI2CDevice,

    //Oversampling settings
    oversampling: u8,

    //Calibration values
    ac1: i16,
    ac2: i16,
    ac3: i16,
    ac4: u16,
    ac5: u16,
    ac6: u16,
    b1: i16,
    b2: i16,
    mb: i16,
    mc: i16,
    md: i16
}

impl Bmp085
{
    pub fn init(mut device: LinuxI2CDevice) -> Result<Bmp085, LinuxI2CError>
    {
        //Read all the callibration values
        let ac1 = try!(read_i16_i2c(&mut device, 0xAA));
        let ac2 = try!(read_i16_i2c(&mut device, 0xAC));
        let ac3 = try!(read_i16_i2c(&mut device, 0xAE));
        let ac4 = try!(device.smbus_read_word_data(0xB0));
        let ac5 = try!(device.smbus_read_word_data(0xB2));
        let ac6 = try!(device.smbus_read_word_data(0xB4));
        let b1 = try!(read_i16_i2c(&mut device, 0xB6));
        let b2 = try!(read_i16_i2c(&mut device, 0xB8));
        let mb = try!(read_i16_i2c(&mut device, 0xBA));
        let mc = try!(read_i16_i2c(&mut device, 0xBC));
        let md = try!(read_i16_i2c(&mut device, 0xBE));

        Ok(
            Bmp085 {
                device,
                oversampling: 0,

                ac1,
                ac2,
                ac3,
                ac4,
                ac5,
                ac6,
                b1,
                b2,
                mb,
                mc,
                md,
            }
        )
    }

    pub fn read_uncompensated_temp(&mut self) -> Result<u16, LinuxI2CError>
    {
        // Request a temperature reading
        try!(self.device.smbus_write_byte_data(0xf4, 0x2e));

        // Wait for the device to read the data
        thread::sleep(Duration::from_millis(5));

        self.device.smbus_read_word_data(0xf6)
    }

    pub fn read_uncompensated_pressure(&mut self) -> Result<u32, LinuxI2CError>
    {
        // Request a pressure reading with the specified oversampling setting
        try!(self.device.smbus_write_byte_data(0xf4, 0x34 + (self.oversampling << 6)));

        // Wait for the data to be ready
        thread::sleep(Duration::from_millis(2 + ((self.oversampling as u64) << 3)));

        let msb = try!(self.device.read_)
    }
}

fn main() {
    let SENSOR_ADDR = 0x77;

    let device = LinuxI2CDevice::new("/dev/i2c-1", SENSOR_ADDR).unwrap();

    let bmp085 = Bmp085::init(device);

    println!("Hello world");
}
