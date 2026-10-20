use ds323x::{DateTimeAccess, Ds323x, NaiveDate, Timelike};
use embassy_rp::{
    i2c::{Blocking, I2c},
    peripherals::I2C0,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::Timer;


#[derive(Clone, Copy)]
pub enum TimeEvent {
    Time { hour: u32, minute: u32 },
}

#[derive(Clone, Copy)]
pub struct TimeUpdate {
    pub timestamp: u64,
}

pub static TIME: Signal<CriticalSectionRawMutex, TimeEvent> = Signal::new();

pub static TIME_UPDATE: Signal<CriticalSectionRawMutex, TimeUpdate> = Signal::new();

#[embassy_executor::task]
pub async fn time_task(i2c: I2c<'static, I2C0, Blocking>) {
    let mut rtc = Ds323x::new_ds3231(i2c);

    loop {
        if TIME_UPDATE.signaled() {
            let time_update = TIME_UPDATE.wait().await;
            // Format: yyyyMMddhhmmss
            let second = time_update.timestamp % 1_00;
            let minute = (time_update.timestamp / 1_00) % 1_00;
            let hour = (time_update.timestamp / 1_00_00) % 1_00;
            let day = (time_update.timestamp / 1_00_00_00) % 1_00;
            let month = (time_update.timestamp / 1_00_00_00_00) % 1_00;
            let year = (time_update.timestamp / 1_00_00_00_00_00) % 1_0000;
            let new_datetime = NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
                .unwrap()
                .and_hms_opt(hour as u32, minute as u32, second as u32)
                .unwrap();
            rtc.set_datetime(&new_datetime).unwrap();
        }
        let now = rtc.datetime().unwrap();
        TIME.signal(TimeEvent::Time {
            hour: now.hour(),
            minute: now.minute(),
        });

        Timer::after_secs(1).await;
    }
}
