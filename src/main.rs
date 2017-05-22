extern crate i2cdev;
extern crate nalgebra as na;

use std::thread;
use std::time::Duration;

use i2cdev::linux::{LinuxI2CDevice};

use std::collections::VecDeque;


use std::fs::OpenOptions;
use std::io::prelude::*;

mod bmp085;
mod bno055;
mod i2c_helpers;

pub fn pressure_logger(mut bmp085: bmp085::Bmp085)
{
    //Read a reference value
    let uncompensated_temp = bmp085.read_uncompensated_temp().unwrap();
    let uncompensated_pressure = bmp085.read_uncompensated_pressure().unwrap();
    let pressure = bmp085.calcuate_real_pressure(uncompensated_temp, uncompensated_pressure);

    let reference_pressure = pressure;

    let mut previous_pressures = VecDeque::with_capacity(8);

    let avg_sample_amount = 8;
    loop
    {
        //Read a reference value
        let uncompensated_temp = bmp085.read_uncompensated_temp().unwrap();
        let uncompensated_pressure = bmp085.read_uncompensated_pressure().unwrap();
        let pressure = bmp085.calcuate_real_pressure(uncompensated_temp, uncompensated_pressure);

        //Store a new value for the average calculation
        if previous_pressures.len() > avg_sample_amount
        {
            previous_pressures.pop_front();
        }
        previous_pressures.push_back(pressure);

        //Calculate the average pressure
        let mut sum = 0;
        for i in 0..previous_pressures.len()
        {
            sum += previous_pressures[i];
        }
        let average = sum / (previous_pressures.len() as i32);

        let altitude = bmp085::altitude_from_pressure(pressure, reference_pressure);
        let avg_altitude = bmp085::altitude_from_pressure(average, reference_pressure);

        let result_string = format!("{}\"p\": {}, \"avg_p\": {}, \"alt\": {}, \"avg_alt\":{}{},",
                        "{",
                        pressure,
                        average,
                        altitude,
                        avg_altitude,
                        "}"
                    );

        println!("result: {}", result_string);
        let mut file = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(true)
                        .open("log.json").unwrap();

        file.write_all(result_string.as_bytes()).unwrap();
    }
}

fn test_bno()
{
    let bno_addr = 0x28;

    let bno_device = LinuxI2CDevice::new("/dev/i2c-1", bno_addr).unwrap();
    let mut bno = bno055::Bno055::new(bno_device).unwrap();

    println!("Bno id: 0x{:X}", bno.get_chip_id().unwrap());
    println!("accel rev: 0x{:X}", bno.get_accel_rev_id().unwrap());
    let status = bno.get_system_status().unwrap();
    println!("Bno status: 0x{:X}, {:?}", status.status, status.error);

    loop {
        println!("heading: {}",
                 bno.get_euler_vector().unwrap(),
                 //bno.get_gravity_vector().unwrap()
                );
        thread::sleep(Duration::from_millis(100))
    }
}

fn main() {
    let bmp_addr = 0x77;

    let bmp_i2c = LinuxI2CDevice::new("/dev/i2c-1", bmp_addr).unwrap();

    let mut bmp = bmp085::Bmp085::init(bmp_i2c).unwrap();

    test_bno();
    //pressure_logger(bmp);
}

