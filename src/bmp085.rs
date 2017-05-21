use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

use i2c_helpers::{read_i16_i2c_little_endian, read_u16_i2c_little_endian};

use std::thread;
use std::time::Duration;

/**
  Note that these functions are a bit off when calculating absolute values for pressure
 */
#[derive(Debug)]
struct Bmp085Parameters
{
    // Oversampling settings. Should be between 0 and 3. 0 Seems to return
    // incorrect values
    pub oversampling: u8,

    // Calibration values
    pub ac1: i16,
    pub ac2: i16,
    pub ac3: i16,
    pub ac4: u16,
    pub ac5: u16,
    pub ac6: u16,
    pub b1: i16,
    pub b2: i16,
    pub mb: i16,
    pub mc: i16,
    pub md: i16
}

impl Bmp085Parameters
{
    pub fn init_as_datasheet() -> Bmp085Parameters
    {
        Bmp085Parameters
        {
            oversampling: 0,

            ac1: 408,
            ac2: -72,
            ac3: -14383,
            ac4: 32741,
            ac5: 32757,
            ac6: 23153,
            b1: 6190,
            b2: 4,
            mb: -32768,
            mc: -8711,
            md: 2868,
        }
    }

    fn calculate_b5(&self, uncompensated_temp: u16) -> i32
    {
        let x1 = (((uncompensated_temp as i32) - (self.ac6 as i32))*(self.ac5 as i32)) >> 15;
        let x2 = ((self.mc as i32) << 11) as i32/(x1 as i32 + self.md as i32);

        x1 as i32 + x2
    }
    pub fn calculate_real_temp(&self, uncompensated_temp: u16) -> i16
    {
        return ((self.calculate_b5(uncompensated_temp) + 8) >> 4) as i16;
    }

    pub fn calcuate_real_pressure(&self, uncompensated_temp: u16, uncompensated_pressure: u32)
        -> i32
    {
        let b5 = self.calculate_b5(uncompensated_temp);

        let b6 = b5 - 4000;

        let x1 = ((self.b2 as i32) * ((b6 * b6) >> 12)) >> 11;
        let x2 = ((self.ac2 as i32) * b6) >> 11;
        let x3 = x1 + x2;

        let b3 = ((((self.ac1 as i32)*4 + x3)<<self.oversampling) + 2)>>2;


        // Shadowing previous x values
        let x1 = ((self.ac3 as i32) * b6) >> 13;
        let x2 = ((self.b1 as i32) * ((b6 * b6) >> 12)) >> 16;
        let x3 = ((x1 + x2) + 2) >> 2;

        let b4 = ((self.ac4 as u32) * ((x3 + 32768) as u32)) >> 15;
        let b7 = ((uncompensated_pressure as i32) - b3 as i32) * (50000 >> self.oversampling);
        let b7 = b7 as u32;

        let p = if b7 < 0x80000000
        {
            (b7 * 2) / b4
        }
        else
        {
            (b7/b4) * 2
        } as i32;

        // New shadowing of xes
        let x1 = (p >> 8) * (p >> 8);
        let x1 = (x1 * 3038) >> 16;
        let x2 = (-7357 * p as i32) >> 16;
        p + ((x1 + x2 + 3791) / 16)
    }
}

pub struct Bmp085
{
    device: LinuxI2CDevice,

    params: Bmp085Parameters
}

impl Bmp085
{
    pub fn init(mut device: LinuxI2CDevice) -> Result<Bmp085, LinuxI2CError>
    {
        //Read all the callibration values
        let ac1 = try!(read_i16_i2c_little_endian(&mut device, 0xAA));
        let ac2 = try!(read_i16_i2c_little_endian(&mut device, 0xAC));
        let ac3 = try!(read_i16_i2c_little_endian(&mut device, 0xAE));
        let ac4 = try!(read_u16_i2c_little_endian(&mut device, 0xB0));
        let ac5 = try!(read_u16_i2c_little_endian(&mut device, 0xB2));
        let ac6 = try!(read_u16_i2c_little_endian(&mut device, 0xB4));
        let b1 = try!(read_i16_i2c_little_endian(&mut device, 0xB6));
        let b2 = try!(read_i16_i2c_little_endian(&mut device, 0xB8));
        let mb = try!(read_i16_i2c_little_endian(&mut device, 0xBA));
        let mc = try!(read_i16_i2c_little_endian(&mut device, 0xBC));
        let md = try!(read_i16_i2c_little_endian(&mut device, 0xBE));

        Ok(
            Bmp085 {
                device,

                params: Bmp085Parameters
                {
                    // For some reason, oversampling=0 does not give the correct
                    // pressure value
                    oversampling: 3,

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
            }
        )
    }

    pub fn read_uncompensated_temp(&mut self) -> Result<u16, LinuxI2CError>
    {
        // Request a temperature reading
        try!(self.device.smbus_write_byte_data(0xf4, 0x2e));

        // Wait for the device to read the data
        thread::sleep(Duration::from_millis(5));

        read_u16_i2c_little_endian(&mut self.device, 0xf6)
    }

    pub fn read_uncompensated_pressure(&mut self) -> Result<u32, LinuxI2CError>
    {
        // Request a pressure reading with the specified oversampling setting
        try!(self.device.smbus_write_byte_data(0xf4, 0x34 + (self.params.oversampling << 6)));

        // Wait for the data to be ready
        thread::sleep(Duration::from_millis(2 + ((self.params.oversampling as u64) << 3)));

        // Set the target address
        try!(self.device.write(&[0xF6]));

        let msb = try!(self.device.smbus_read_byte());
        let lsb = try!(self.device.smbus_read_byte());
        let xlsb = try!(self.device.smbus_read_byte());

        Ok((((msb as u32) << 16) | ((lsb as u32) << 8) | (xlsb as u32)) >> (8-self.params.oversampling as u32))
    }

    pub fn calculate_real_temp(&self, uncompensated_temp: u16) -> i16
    {
        self.params.calculate_real_temp(uncompensated_temp)
    }

    pub fn calcuate_real_pressure(&self, uncompensated_temp: u16, uncompensated_pressure: u32)
        -> i32
    {
        self.params.calcuate_real_pressure(uncompensated_temp, uncompensated_pressure)
    }

    pub fn print_params(&self)
    {
        println!("Params: {:?}", self.params);
    }
}

pub fn altitude_from_pressure(pressure: i32, reference_pressure: i32) -> f32
{
    44330. * (1. - ((pressure as f32) / (reference_pressure as f32)).powf(1./5.255))
}




#[cfg(test)]
mod bmp085_test
{
    use super::*;

    #[test]
    fn real_value_calculation()
    {
        let params = Bmp085Parameters::init_as_datasheet();

        let ut = 27898;
        let up = 23843;

        assert_eq!(params.calculate_real_temp(ut), 150);
        assert_eq!(params.calcuate_real_pressure(ut, up), 69965);
    }
}
