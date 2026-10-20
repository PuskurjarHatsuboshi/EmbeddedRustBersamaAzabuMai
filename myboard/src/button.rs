use embassy_rp::gpio::Input;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::Timer;

#[derive(Clone, Copy)]
pub enum ButtonEvent {
    Click,
}

pub static BUTTON: Signal<CriticalSectionRawMutex, ButtonEvent> = Signal::new();

#[embassy_executor::task]
pub async fn button_task(mut button: Input<'static>) {
    loop {
        button.wait_for_low().await;
        BUTTON.signal(ButtonEvent::Click);
        button.wait_for_high().await;
        Timer::after_millis(250).await;
    }
}
