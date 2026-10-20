use embassy_rp::gpio::Output;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

pub enum LedEvent {
    On,
    Off,
}
pub static LED: Signal<CriticalSectionRawMutex, LedEvent> = Signal::new();

#[embassy_executor::task]
pub async fn led_task(mut led: Output<'static>) {
    loop {
        let led_event = LED.wait().await;
        match led_event {
            LedEvent::On => led.set_high(),
            LedEvent::Off => led.set_low(),
        }
    }
}
