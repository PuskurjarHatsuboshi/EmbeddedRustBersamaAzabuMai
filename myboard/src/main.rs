#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_time::Timer;
use panic_halt as _;

#[embassy_executor::main(
    executor = "embassy_rp::executor::Executor",
    entry = "cortex_m_rt::entry"
)]
async fn main(_spawner: Spawner) -> ! {
    // Grab our peripherals
    let peripherals = embassy_rp::init(Default::default());

    // Configure GPIO15 as an output
    let mut led_pin = Output::new(peripherals.PIN_15, Level::Low);

    loop {
        led_pin.set_high();
        Timer::after_millis(250).await;
        led_pin.set_low();
        Timer::after_millis(250).await;
    }
}
