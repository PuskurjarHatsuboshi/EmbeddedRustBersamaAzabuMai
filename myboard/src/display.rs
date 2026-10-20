use core::cell::RefCell;
use core::fmt::Write;
use eg_seven_segment::SevenSegmentStyleBuilder;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_rp::{
    gpio::Output,
    peripherals::SPI0,
    spi::{Blocking, Spi},
};
use embassy_sync::{
    blocking_mutex::Mutex, blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal,
};
use embassy_time::{Delay, Timer};
use embedded_graphics::{
    image::Image, pixelcolor::Rgb565, prelude::*, primitives::Rectangle, text::Text,
};
use mipidsi::{
    Builder,
    interface::SpiInterface,
    models::ILI9341Rgb565,
    options::{ColorOrder, Orientation, Rotation},
};
use tinybmp::Bmp;

use crate::time::TIME;
pub enum DisplayEvent {
    Red,
    Green,
}

#[embassy_executor::task]
pub async fn display_task(
    spi: Spi<'static, SPI0, Blocking>,
    cs: Output<'static>,
    dc: Output<'static>,
    reset: Output<'static>,
) {
    let mut buffer = [0_u8; 512];
    let mut delay = Delay;
    let spi_bus: Mutex<CriticalSectionRawMutex, _> = Mutex::new(RefCell::new(spi));
    let spi_device = SpiDevice::new(&spi_bus, cs);
    let spi_interface = SpiInterface::new(spi_device, dc, &mut buffer);
    let mut display = Builder::new(ILI9341Rgb565, spi_interface)
        .display_size(240, 320)
        .orientation(Orientation::new().rotate(Rotation::Deg90).flip_horizontal())
        .color_order(ColorOrder::Bgr)
        .reset_pin(reset)
        .init(&mut delay)
        .unwrap(); // delay provider from your MCU
    display.clear(Rgb565::WHITE).unwrap();

    // Gambar diri sendiri :)
    let bmp_data = include_bytes!("./maimai.bmp");
    let bmp = Bmp::<Rgb565>::from_slice(bmp_data).unwrap();
    Image::new(&bmp, Point::new(0, 0))
        .draw(&mut display)
        .unwrap();
    let mut hour_str = heapless::String::<8>::new();
    let mut minute_str = heapless::String::<8>::new();

    loop {
        if TIME.signaled() {
            match TIME.wait().await {
                crate::time::TimeEvent::Time { hour, minute } => {
                    hour_str.clear();
                    minute_str.clear();
                    write!(&mut hour_str, "{:02}", hour).unwrap();
                    write!(&mut minute_str, "{:02}", minute).unwrap();
                }
            }
        }

        display
            .fill_solid(
                &Rectangle::new(Point::new(213, 35), Size::new(107, 150)),
                Rgb565::WHITE,
            )
            .unwrap();
        let segment_color = Rgb565::new(3, 51, 29); // #19ceef
        let style = SevenSegmentStyleBuilder::new()
            .digit_size(Size::new(30, 70)) // digits are 10x20 pixels
            .digit_spacing(5) // 5px spacing between digits
            .segment_width(5) // 5px wide segments
            .segment_color(segment_color) // active segments are green
            .build();

        Text::new(&hour_str, Point::new(243, 105), style)
            .draw(&mut display)
            .unwrap();

        Text::new(&minute_str, Point::new(243, 185), style)
            .draw(&mut display)
            .unwrap();

        Timer::after_secs(1).await;
    }
}
