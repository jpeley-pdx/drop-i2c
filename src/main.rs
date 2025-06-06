//! Embedded Rust Programming - Prog 2
//! Drop
//!
//! https://canvas.pdx.edu/courses/101554/assignments/1057943?module_item_id=4682020
//!
//! Write a Rust program for the MicroBit with the following specification.

//! While the board is not falling (IMU ≥ 0.5g) it should be silent,
//! and the board should display a single dot on the center LED.
//!
//! While the board is falling (IMU < 0.5g) it should "yell"
//! with a 1KHz square-wave tone on the speaker and show an exclamation point on the display.

#![no_main]
#![no_std]

use cortex_m_rt::entry;
use embedded_hal::{delay::DelayNs, digital::OutputPin};
use microbit::{
    gpio::DisplayPins,
    hal::Timer,
    hal::gpio::Level,
    hal::pwm::{Channel, CounterMode, Prescaler, Pwm, Seq},
    hal::time::Hertz,
    hal::twim,
    pac::twim0::frequency::FREQUENCY_A,
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

// Crate for the ST Micro LSM303AGR Accelerometer - Magnetometer Combo
// https://docs.rs/lsm303agr/latest/lsm303agr/index.html
// https://www.st.com/resource/en/datasheet/lsm303agr.pdf
use lsm303agr::{AccelMode, AccelOutputDataRate, Lsm303agr, MagMode, MagOutputDataRate};

/// This function handles the LED on/off states
fn set_led(state: bool, x: i8, y: i8, display_pins: &mut DisplayPins) {
    match (state, y) {
        (true, 0) => {
            display_pins.row1.set_high().unwrap();
        }
        (true, 1) => {
            display_pins.row2.set_high().unwrap();
        }
        (true, 2) => {
            display_pins.row3.set_high().unwrap();
        }
        (true, 3) => {
            display_pins.row4.set_high().unwrap();
        }
        (true, 4) => {
            display_pins.row5.set_high().unwrap();
        }

        (false, 0) => {
            display_pins.row1.set_low().unwrap();
        }
        (false, 1) => {
            display_pins.row2.set_low().unwrap();
        }
        (false, 2) => {
            display_pins.row3.set_low().unwrap();
        }
        (false, 3) => {
            display_pins.row4.set_low().unwrap();
        }
        (false, 4) => {
            display_pins.row5.set_low().unwrap();
        }

        (_, _) => {}
    }

    match (state, x) {
        (true, 0) => {
            display_pins.col1.set_low().unwrap();
        }
        (true, 1) => {
            display_pins.col2.set_low().unwrap();
        }
        (true, 2) => {
            display_pins.col3.set_low().unwrap();
        }
        (true, 3) => {
            display_pins.col4.set_low().unwrap();
        }
        (true, 4) => {
            display_pins.col5.set_low().unwrap();
        }

        (false, 0) => {
            display_pins.col1.set_high().unwrap();
        }
        (false, 1) => {
            display_pins.col2.set_high().unwrap();
        }
        (false, 2) => {
            display_pins.col3.set_high().unwrap();
        }
        (false, 3) => {
            display_pins.col4.set_high().unwrap();
        }
        (false, 4) => {
            display_pins.col5.set_high().unwrap();
        }

        (_, _) => {}
    }
}

#[entry]
fn main() -> ! {
    // Setup the debug print
    rtt_init_print!();

    // Get control of the board functions
    let mut board = microbit::Board::take().unwrap();

    // Acquire a timer and the PWM
    let mut timer = Timer::new(board.TIMER0);
    let pwm0 = Pwm::new(board.PWM0);
    pwm0.set_output_pin(
        Channel::C0,
        board
            .speaker_pin
            .into_push_pull_output(Level::Low)
            .degrade(),
    );

    // Acquire the I2C bus
    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

    // Connect the Accel driver to the device
    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor
        .set_accel_mode_and_odr(&mut timer, AccelMode::Normal, AccelOutputDataRate::Hz100)
        .unwrap();
    sensor
        .set_mag_mode_and_odr(
            &mut timer,
            MagMode::HighResolution,
            MagOutputDataRate::Hz100,
        )
        .unwrap();
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();

    // Set the initial values for the PWM
    pwm0.set_prescaler(Prescaler::Div128);
    pwm0.set_period(Hertz(1u32));
    pwm0.set_counter_mode(CounterMode::Up);
    pwm0.set_max_duty(200);
    // pwm0.enable(); // Don't enable just yet.

    // Configure 50% duty cycle
    let max_duty = pwm0.max_duty();
    pwm0.set_duty_on_common(max_duty / 2);

    // This stop is to prevent the speaker from making sound after every this is setup
    pwm0.stop();

    let mut duty_cycle = 50;

    // let mut x_accum: i32 = 0; // Used for test. Not needed.
    // let mut x_avg = 0;        // Used for test. Not needed.
    // let mut x_diff = 0;       // Used for test. Not needed.
    let mut x_raw = 0;

    // let mut y_accum: i32 = 0; // Used for test. Not needed.
    // let mut y_avg = 0;        // Used for test. Not needed.
    // let mut y_diff = 0;       // Used for test. Not needed.
    let mut y_raw = 0;

    // let mut z_diff = 0;       // Used for test. Not needed.
    let mut z_accum: i32 = 0;
    let mut z_avg = 0;
    let mut z_raw = 0;

    let mut sample_count = 0;

    // init the sound trigger variables
    let mut trigger_up = false;
    let mut trigger_down = false;

    set_led(true, 2, 2, &mut board.display_pins);

    // Main Loop
    loop {
        // Check the sensor to see if there is any new data.
        if sensor.accel_status().unwrap().xyz_new_data() {
            let data = sensor.acceleration().unwrap();

            // read the raw data and convert to signed int.
            // the data is provided as a 2s commplement and
            // left justifiec
            x_raw = data.x_raw() as i16;
            y_raw = data.y_raw() as i16;
            z_raw = data.z_raw() as i16;

            // sample the accel data over 10 samples
            // and calculate the average
            if sample_count < 10 {
                // x_accum += x_raw as i32;
                // y_accum += y_raw as i32;
                z_accum += z_raw as i32;

                sample_count += 1;

                // x_avg = x_accum / sample_count;
                // y_avg = y_accum / sample_count;
                z_avg = z_accum / sample_count;

                // let x_d = x_avg - x_raw as i32;
                // let y_d = y_avg - y_raw as i32;
                // let z_d = z_avg - z_raw as i32;
            } else {
                // x_accum = x_raw as i32;
                // y_accum = y_raw as i32;
                z_accum = z_raw as i32;
                sample_count = 0;
            }
        }
        // rprintln!("Accelerometer: x {} y {} z {}\r\n", data.x_raw(), data.y_raw(), data.z_raw());
        // rprintln!("Accelerometer: x avg {}\r\n", x_avg);

        // if both the up and down triggers are idle
        if (!trigger_down) & (!trigger_up) {
            pwm0.stop(); // make sure audio is stopped

            // determine if the device is going up or down.
            // Set the correct trigger for that condition.
            // this section is for the board in the speaker
            // up orientation
            if (z_raw > 20000) & (z_avg > 0) && !trigger_down {
                trigger_up = true;
                rprintln!("UP Pos   : x {} | y {} | z {}\r\n", x_raw, y_raw, z_raw);
            } else if (z_raw < 10000) & (z_avg > 0) && !trigger_up {
                trigger_down = true;
                rprintln!("DOWN Pos : x {} | y {} | z {}\r\n", x_raw, y_raw, z_raw);
            }

            // determine if the device is going up or down.
            // Set the correct trigger for that condition.
            // this section is for the board in the speaker
            // down orientation
            if (z_raw < -20000) & (z_avg < 0) && !trigger_up {
                trigger_up = true;
                rprintln!("UP Neg   : x {} | y {} | z {}\r\n", x_raw, y_raw, z_raw);
            } else if (z_raw > -10000) & (z_avg < 0) && !trigger_down {
                trigger_down = true;
                rprintln!("DOWN Neg : x {} | y {} | z {}\r\n", x_raw, y_raw, z_raw);
            }
        }

        // if the board is moving up, output an ascending sound
        if trigger_up {
            if duty_cycle < 51 {
                duty_cycle = 200;
                trigger_up = false;
                pwm0.stop();
                set_led(false, 2, 0, &mut board.display_pins);
                set_led(false, 2, 1, &mut board.display_pins);
                set_led(false, 2, 4, &mut board.display_pins);
            } else {
                duty_cycle -= 1;
                pwm0.set_max_duty(duty_cycle);
                pwm0.start_seq(Seq::Seq0);
                set_led(true, 2, 0, &mut board.display_pins);
                set_led(true, 2, 1, &mut board.display_pins);
                set_led(true, 2, 4, &mut board.display_pins);
            }
            // rprintln!("{:?}", pwm0.max_duty() );
            timer.delay_ms(5); // delay so it is smooth
        }

        // if the board is moving down, output an decending sound
        if trigger_down {
            if duty_cycle > 199 {
                duty_cycle = 50;
                trigger_down = false;
                pwm0.stop();
                set_led(false, 2, 0, &mut board.display_pins);
                set_led(false, 2, 1, &mut board.display_pins);
                set_led(false, 2, 4, &mut board.display_pins);
            } else {
                duty_cycle += 1;
                pwm0.set_max_duty(duty_cycle);
                pwm0.start_seq(Seq::Seq0);
                set_led(true, 2, 0, &mut board.display_pins);
                set_led(true, 2, 1, &mut board.display_pins);
                set_led(true, 2, 4, &mut board.display_pins);
            }
            // rprintln!("{:?}", pwm0.max_duty() );
            timer.delay_ms(5); // delay so it is smooth
        }

        set_led(true, 2, 2, &mut board.display_pins);
        // set_led(false, 2, 2, &mut board.display_pins);
    }
}
