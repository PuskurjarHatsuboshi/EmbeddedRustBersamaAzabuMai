use ds323x::{DateTimeAccess, Ds323x, NaiveDate, Timelike};
use embassy_rp::{
    i2c::{Blocking, I2c},
    peripherals::I2C0,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal, watch::Watch};
use embassy_time::Timer;

use crate::led::LED;

#[derive(Clone, Copy)]
pub enum TimeEvent {
    Time { hour: u8, minute: u8 },
}

#[derive(Clone, Copy)]
pub struct TimeUpdate {
    hour: u8,
    minute: u8,
}

pub static TIME: Signal<CriticalSectionRawMutex, TimeEvent> = Signal::new();

pub static TIME_UPDATE: Signal<CriticalSectionRawMutex, TimeUpdate> = Signal::new();

#[embassy_executor::task]
pub async fn time_task(i2c: I2c<'static, I2C0, Blocking>) {
    let mut rtc = Ds323x::new_ds3231(i2c);

    loop {
        let now = rtc.datetime().unwrap();
        TIME.signal(TimeEvent::Time {
            hour: now.hour() as u8,
            minute: now.minute() as u8,
        });

        Timer::after_secs(1).await;
    }
}
