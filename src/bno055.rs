extern crate nalgebra as na;

use std::thread;
use std::time::Duration;

use i2cdev::core::I2CDevice;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

const MODE_CONFIG: u8 = 0x00;
const MODE_NDOF: u8 = 0x0C;

const OPR_MODE_REG: u8 = 0x3D;

const POWER_MODE_NORMAL: u8 = 0x0;
const POWER_MODE_REG: u8 = 0x03E;

const EULER_VECTOR_REG: u8 = 0x1A;
const GRAVITY_VECTOR_REG: u8 = 0x2E;

//Selects internal or external clock?
const SYS_TRIGGER_ADDR: u8 = 0x3f;
const SYSTEM_STATUS_ADDR: u8 = 0x39;
const SYSTEM_ERROR_ADDR: u8 = 0x3a;

const PAGE_ID_ADDR: u8 = 0x07;

use i2c_helpers;

#[derive(Debug)]
pub struct Bno055Status
{
    pub status: u8,
    pub error: Option<u8>
}

pub struct Bno055
{
    i2c_device: LinuxI2CDevice
}

impl Bno055
{
    pub fn new(mut i2c_device: LinuxI2CDevice) -> Result<Bno055, LinuxI2CError>
    {
        //Send some dummy data
        i2c_device.smbus_write_byte_data(PAGE_ID_ADDR, 0);

        //Enter config mode
        i2c_device.smbus_write_byte_data(OPR_MODE_REG, MODE_CONFIG)?;
        //Wait for the mode to change
        thread::sleep(Duration::from_millis(20));

        i2c_device.smbus_write_byte_data(PAGE_ID_ADDR, 0);

        //Reset the chip
        i2c_device.smbus_write_byte_data(SYS_TRIGGER_ADDR, 0x20);
        //Sleep until the chip is ready
        thread::sleep(Duration::from_millis(650));

        //Set power mode
        i2c_device.smbus_write_byte_data(POWER_MODE_REG, POWER_MODE_NORMAL)?;
        //use internal oscilator
        i2c_device.smbus_write_byte_data(SYS_TRIGGER_ADDR, 0x0)?;

        //Set operation mode
        i2c_device.smbus_write_byte_data(OPR_MODE_REG, MODE_NDOF)?;

        //Wait for the mode to change
        thread::sleep(Duration::from_millis(20));

        Ok (Bno055 {
            i2c_device
        })
    }

    pub fn get_chip_id(&mut self) -> Result<u8, LinuxI2CError>
    {
        self.i2c_device.smbus_read_byte_data(0x00)
    }

    pub fn get_accel_rev_id(&mut self) -> Result<u8, LinuxI2CError>
    {
        self.i2c_device.smbus_read_byte_data(0x01)
    }

    pub fn get_gravity_vector(&mut self) -> Result<na::Vector3<i16>, LinuxI2CError>
    {
        self.read_vector3(GRAVITY_VECTOR_REG)
    }

    pub fn get_euler_vector(&mut self) -> Result<na::Vector3<f32>, LinuxI2CError>
    {
        let raw = self.read_vector3(EULER_VECTOR_REG)?;

        Ok(
            na::Vector3::new(
                raw.x as f32 / 16.0,
                raw.y as f32 / 16.0,
                raw.z as f32 / 16.0
            )
        )
    }

    pub fn get_system_status(&mut self) -> Result<Bno055Status, LinuxI2CError>
    {
        let status = self.i2c_device.smbus_read_byte_data(SYSTEM_STATUS_ADDR)?;

        let error = if status == 0x01 {
            Some(self.i2c_device.smbus_read_byte_data(SYSTEM_ERROR_ADDR)?)
        }
        else {
            None
        };
        Ok (
            Bno055Status
            {
                status,
                error
            }
        )
    }

    fn read_vector3(&mut self, start_addr: u8) -> Result<na::Vector3<i16>, LinuxI2CError>
    {
        Ok(
            na::Vector3::new(
                i2c_helpers::read_i16_i2c_big_endian(&mut self.i2c_device, start_addr)?,
                i2c_helpers::read_i16_i2c_big_endian(&mut self.i2c_device, start_addr + 2)?,
                i2c_helpers::read_i16_i2c_big_endian(&mut self.i2c_device, start_addr + 4)?
            )
        )
    }

}
