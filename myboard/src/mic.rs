use embassy_rp::gpio::Input;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::Timer;

#[derive(Clone, Copy)]
pub enum MicrophoneEvent {
    Loud,
}

pub static MICROPHONE: Signal<CriticalSectionRawMutex, MicrophoneEvent> = Signal::new();

#[embassy_executor::task]
pub async fn microphone_task(mut mic: Input<'static>) {
    loop {
        mic.wait_for_high().await;
        MICROPHONE.signal(MicrophoneEvent::Loud);
        mic.wait_for_low().await;
        Timer::after_millis(250).await;
    }
}
