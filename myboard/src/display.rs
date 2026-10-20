use core::cell::RefCell;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_rp::{
    gpio::Output,
    peripherals::SPI0,
    spi::{Blocking, Spi},
};
use embassy_sync::{
    blocking_mutex::Mutex, blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal,
};
use embassy_time::Delay;
use embedded_graphics::{image::Image, pixelcolor::Rgb565, prelude::*, primitives::Rectangle};
use mipidsi::{
    Builder,
    interface::SpiInterface,
    models::ILI9341Rgb565,
    options::{ColorOrder, Orientation, Rotation},
};
use tinybmp::Bmp;
pub enum DisplayEvent {
    Red,
    Green,
}
pub static DISPLAY: Signal<CriticalSectionRawMutex, DisplayEvent> = Signal::new();

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
    display.clear(Rgb565::WHITE);

    // Gambar diri sendiri :)
    let bmp_data = include_bytes!("./maimai.bmp");
    let bmp = Bmp::<Rgb565>::from_slice(bmp_data).unwrap();
    Image::new(&bmp, Point::new(0, 0))
        .draw(&mut display)
        .unwrap();

    loop {
        let display_event = DISPLAY.wait().await;

        match display_event {
            DisplayEvent::Red => display
                .fill_solid(
                    &Rectangle::new(Point::new(213, 0), Size::new(107, 240)),
                    Rgb565::RED,
                )
                .unwrap(),
            DisplayEvent::Green => display
                .fill_solid(
                    &Rectangle::new(Point::new(213, 0), Size::new(107, 240)),
                    Rgb565::GREEN,
                )
                .unwrap(),
        }
    }
}
