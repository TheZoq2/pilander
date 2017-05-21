extern crate i2cdev;

use std::thread;
use std::time::Duration;

use i2cdev::linux::{LinuxI2CDevice};

use std::collections::VecDeque;


use std::fs::OpenOptions;
use std::io::prelude::*;

mod bmp085;
mod i2c_helpers;

pub fn pressure_logger(mut bmp085: bmp085::Bmp085)
{
    //Read a reference value
    let uncompensated_temp = bmp085.read_uncompensated_temp().unwrap();
    let uncompensated_pressure = bmp085.read_uncompensated_pressure().unwrap();
    let pressure = bmp085.calcuate_real_pressure(uncompensated_temp, uncompensated_pressure);

    let reference_pressure = pressure;

    let mut previous_pressures = VecDeque::with_capacity(8);
    for _ in 0..8
    {
        previous_pressures.push_back(0);
    }

    loop
    {
        //Read a reference value
        let uncompensated_temp = bmp085.read_uncompensated_temp().unwrap();
        let uncompensated_pressure = bmp085.read_uncompensated_pressure().unwrap();
        let pressure = bmp085.calcuate_real_pressure(uncompensated_temp, uncompensated_pressure);

        //Store a new value for the average calculation
        previous_pressures.pop_front();
        previous_pressures.push_back(pressure);

        //Calculate the average pressure
        let mut sum = 0;
        for i in 0..previous_pressures.len()
        {
            sum += previous_pressures[i];
        }
        let average = sum / previous_pressures.len() as i32;

        let altitude = bmp085::altitude_from_pressure(pressure, reference_pressure);
        let avg_altitude = bmp085::altitude_from_pressure(average, reference_pressure);

        let result_string = format!("{}\"p\": {}, \"avg_t\": {}, \"alt\": {}, \"avg_alt\":{}{},",
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

fn main() {
    let sensor_addr = 0x77;

    let device = LinuxI2CDevice::new("/dev/i2c-1", sensor_addr).unwrap();

    let bmp085 = bmp085::Bmp085::init(device).unwrap();

    pressure_logger(bmp085);
}

