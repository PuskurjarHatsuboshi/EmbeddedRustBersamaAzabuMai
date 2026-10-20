#![no_std]
#![no_main]

use crate::{button::BUTTON, mic::MICROPHONE};
use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_rp::{
    bind_interrupts, dma,
    gpio::{Input, Level, Output},
    i2c::{self, I2c},
    peripherals::{DMA_CH0, DMA_CH1},
    spi::{Config, Spi},
};
use panic_halt as _;

mod button;
mod display;
mod led;
mod mic;
mod time;

#[embassy_executor::main(
    executor = "embassy_rp::executor::Executor",
    entry = "cortex_m_rt::entry"
)]
async fn main(_spawner: Spawner) -> ! {
    let peripherals = embassy_rp::init(Default::default());

    let led_pin = Output::new(peripherals.PIN_15, Level::Low);
    let button_pin = Input::new(peripherals.PIN_14, embassy_rp::gpio::Pull::Up);
    let mic_pin = Input::new(peripherals.PIN_13, embassy_rp::gpio::Pull::Down);

    let cs = peripherals.PIN_17;
    let clk = peripherals.PIN_18;
    let mosi = peripherals.PIN_19;
    let dc = peripherals.PIN_20;
    let reset = peripherals.PIN_21;

    let i2c_sda = peripherals.PIN_4;
    let i2c_scl = peripherals.PIN_5;

    let mut config = Config::default();
    config.frequency = 40_000_000;
    let spi = Spi::new_blocking_txonly(peripherals.SPI0, clk, mosi, config);

    let mut config = i2c::Config::default();
    config.frequency = 100_000;
    let i2c = I2c::new_blocking(peripherals.I2C0, i2c_scl, i2c_sda, config);

    let cs = Output::new(cs, Level::Low);
    let dc = Output::new(dc, Level::Low);
    let reset = Output::new(reset, Level::Low);

    _spawner.spawn(button::button_task(button_pin).unwrap());
    _spawner.spawn(mic::microphone_task(mic_pin).unwrap());
    _spawner.spawn(display::display_task(spi, cs, dc, reset).unwrap());
    _spawner.spawn(led::led_task(led_pin).unwrap());
    _spawner.spawn(time::time_task(i2c).unwrap());

    let mut is_up = false;
    loop {
        if is_up {
            is_up = false;
            led::LED.signal(led::LedEvent::Off);
        } else {
            is_up = true;
            led::LED.signal(led::LedEvent::On);
        }
        select(BUTTON.wait(), MICROPHONE.wait()).await;
    }
}
