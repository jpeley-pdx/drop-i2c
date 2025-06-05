#![no_main]
#![no_std]

use core::str;

use cortex_m_rt::entry;
// use cortex_m::delay::Delay;
use rtt_target::{rprintln, rtt_init_print};
use panic_rtt_target as _;

use microbit::{
    hal::twim,
    hal::Timer,
    pac::twim0::frequency::FREQUENCY_A,
    hal::uarte,
    hal::uarte::{Baudrate, Parity},
};
use embedded_hal::delay::DelayNs;
use microbit::hal::prelude::*;
use lsm303agr::{AccelOutputDataRate, MagOutputDataRate, Lsm303agr, AccelMode, MagMode};
use heapless::Vec;
use nb::block;
use core::fmt::Write;



#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    let mut timer = Timer::new(board.TIMER0);

 
    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_accel_mode_and_odr(&mut timer, AccelMode::Normal, AccelOutputDataRate::Khz1_344).unwrap();
    sensor.set_mag_mode_and_odr(&mut timer, MagMode::HighResolution ,MagOutputDataRate::Hz100).unwrap();
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();

    loop {
        let mut buffer: Vec<u8, 32> = Vec::new();

        rprintln!("{}", sensor.accel_status().unwrap().xyz_new_data() );
        
        if sensor.accel_status().unwrap().xyz_new_data()  {
            let data = sensor.acceleration().unwrap();            
            rprintln!("Accelerometer: x {} y {} z {}\r\n", data.x_raw(), data.y_raw(), data.z_raw());

        } 

        if sensor.mag_status().unwrap().xyz_new_data()  {
            let data = sensor.magnetic_field().unwrap();
            rprintln!("Magnetometer: x {} y {} z {}\r\n", data.x_raw(), data.y_raw(), data.z_raw());
        }

        timer.delay_ms(500);
    }

}
