#![no_main]
#![no_std]

use cortex_m_rt::entry;
// use cortex_m::delay::Delay;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use embedded_hal::delay::DelayNs;
use microbit::{
    hal::Timer,
    hal::gpio::Level,
    hal::pwm::{Channel, CounterMode, Prescaler, Pwm, Seq},
    hal::time::Hertz,
    hal::twim,
    pac::twim0::frequency::FREQUENCY_A,
};

use lsm303agr::{AccelMode, AccelOutputDataRate, Lsm303agr, MagMode, MagOutputDataRate};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    let mut timer = Timer::new(board.TIMER0);
    let pwm0 = Pwm::new(board.PWM0);
    pwm0.set_output_pin(
        Channel::C0,
        board
            .speaker_pin
            .into_push_pull_output(Level::Low)
            .degrade(),
    );
    pwm0.set_prescaler(Prescaler::Div128);
    pwm0.set_period(Hertz(1u32));
    pwm0.set_counter_mode(CounterMode::Up);
    pwm0.set_max_duty(200);
    // pwm0.enable();

    // pwm0.set_seq_refresh(Seq::Seq0, 100);
    // pwm0.set_seq_end_delay(Seq::Seq0, 100);

    // Configure 50% duty cycle
    let max_duty = pwm0.max_duty();
    pwm0.set_duty_on_common(max_duty / 2);

    // pwm0.enable();
    // rprintln!("{:?}", max_duty );
    // rprintln!("{:int}", pwm0.period() );

    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

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

    let mut duty_cycle = 50;
    // let mut period = 2000u32;

    // let mut dir = false;

    // let mut x_accum: i32 = 0;
    // let mut x_avg = 0;
    let mut x_raw = 0;
    // let mut x_diff = 0;

    // let mut y_accum: i32 = 0;
    // let mut y_avg = 0;
    let mut y_raw = 0;
    // let mut y_diff = 0;

    let mut z_accum: i32 = 0;
    let mut z_avg = 0;
    let mut z_raw = 0;
    // let mut z_diff = 0;

    let mut sample_count = 0;

    // let mut stat_count = 0;

    let mut trigger_up = false;
    let mut trigger_down = false;

    // let mut z_check = 0;

    loop {
        if sensor.accel_status().unwrap().xyz_new_data() {
            let data = sensor.acceleration().unwrap();

            x_raw = data.x_raw() as i16;
            y_raw = data.y_raw() as i16;
            z_raw = data.z_raw() as i16;

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

            // rprintln!("Diff   : x {} | y {} | z {}\r\n", x_diff, y_diff, z_diff);
        }
        // rprintln!("Accelerometer: x {} y {} z {}\r\n", data.x_raw(), data.y_raw(), data.z_raw());
        // rprintln!("Accelerometer: x avg {}\r\n", x_avg);

        if (!trigger_down) & (!trigger_up) {
            pwm0.stop();

            if (z_raw > 20000) & (z_avg > 0) && !trigger_down {
                trigger_up = true;
                rprintln!("UP Pos   : x {} | y {} | z {}\r\n", x_raw, y_raw, z_raw);
            } else if (z_raw < 10000) & (z_avg > 0) && !trigger_up {
                trigger_down = true;
                rprintln!("DOWN Pos : x {} | y {} | z {}\r\n", x_raw, y_raw, z_raw);
            }

            if (z_raw < -20000) & (z_avg < 0) && !trigger_up {
                trigger_up = true;
                rprintln!("UP Neg   : x {} | y {} | z {}\r\n", x_raw, y_raw, z_raw);
            } else if (z_raw > -10000) & (z_avg < 0) && !trigger_down {
                trigger_down = true;
                rprintln!("DOWN Neg : x {} | y {} | z {}\r\n", x_raw, y_raw, z_raw);
            }
        }

        if trigger_up {
            if duty_cycle < 51 {
                duty_cycle = 200;
                trigger_up = false;
                pwm0.stop();
            } else {
                duty_cycle -= 1;

                pwm0.set_max_duty(duty_cycle);
                // pwm0.stop();
                pwm0.start_seq(Seq::Seq0);
            }
            // rprintln!("{:?}", pwm0.max_duty() );
            timer.delay_ms(5);
        }

        if trigger_down {
            if duty_cycle > 199 {
                duty_cycle = 50;
                trigger_down = false;
                pwm0.stop();
            } else {
                duty_cycle += 1;

                pwm0.set_max_duty(duty_cycle);
                // pwm0.stop();
                pwm0.start_seq(Seq::Seq0);
            }
            // rprintln!("{:?}", pwm0.max_duty() );
            timer.delay_ms(5);
        }

        // if (! trigger_down) & (! trigger_up) {
        //     pwm0.stop();
        // }

        // timer.delay_ms(100);
    }
}
