#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_rp::gpio::{Input, Level, Output};
use panic_halt as _;

use crate::{button::BUTTON, mic::MICROPHONE};

mod button;
mod mic;

#[embassy_executor::main(
    executor = "embassy_rp::executor::Executor",
    entry = "cortex_m_rt::entry"
)]
async fn main(_spawner: Spawner) -> ! {
    // Grab our peripherals
    let peripherals = embassy_rp::init(Default::default());

    // Configure GPIO15 as an output
    let mut led_pin = Output::new(peripherals.PIN_15, Level::Low);
    let button_pin = Input::new(peripherals.PIN_14, embassy_rp::gpio::Pull::Up);
    let mic_pin = Input::new(peripherals.PIN_13, embassy_rp::gpio::Pull::Down);

    _spawner.spawn(button::button_task(button_pin).unwrap());
    _spawner.spawn(mic::microphone_task(mic_pin).unwrap());

    let mut is_up = false;
    loop {
        if is_up {
            is_up = false;
            led_pin.set_low();
        } else {
            is_up = true;
            led_pin.set_high();
        }
        select(BUTTON.wait(), MICROPHONE.wait()).await;
    }
}
