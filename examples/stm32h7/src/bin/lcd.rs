#![no_std]
#![no_main]

use core::fmt::Write;
use core::str::from_utf8;

use cortex_m_rt::entry;
use defmt::*;
use display_interface_spi::SPIInterface;
use embassy_executor::{Executor, Spawner};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::time::mhz;
use embassy_stm32::{spi, Config};
use embassy_time::{Delay, Timer};
use embedded_graphics::image::{ImageRawLE, *};
use embedded_graphics::mono_font::ascii::{FONT_10X20, FONT_6X10};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::{Rgb565, Rgb888};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::text::{Alignment, LineHeight, Text, TextStyleBuilder};
use heapless::String;
use st7789::ST7789;
use static_cell::StaticCell;
use tinybmp::Bmp;
use {defmt_rtt as _, panic_probe as _};

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV8), // used by SPI3. 100Mhz.
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
    }
    let p = embassy_stm32::init(config);

    let mut spi_config = spi::Config::default();
    spi_config.frequency = mhz(24);

    let spi = spi::Spi::new_txonly(p.SPI4, p.PE12, p.PE14, p.DMA1_CH3, spi_config);
    //let mut led = Output::new(p.PC13, Level::High, Speed::Low);
    let cs = Output::new(p.PE11, Level::Low, Speed::Low);
    let dc = Output::new(p.PE15, Level::Low, Speed::Low);
    let bl = Output::new(p.PD15, Level::Low, Speed::Low);
    let di = SPIInterface::new(spi, dc, cs);
    let mut lcd = ST7789::new(di, None::<Output>, Some(bl), 240, 320);
    let mut delay = Delay;
    lcd.init(&mut delay).unwrap();
    lcd.set_orientation(st7789::Orientation::Landscape).unwrap();
    lcd.set_backlight(st7789::BacklightState::On, &mut delay).unwrap();
    lcd.clear(Rgb565::BLACK).unwrap();

    // 3 lines composing a big "F"
    let line1 = Line::new(Point::new(100, 20), Point::new(100, 220))
        .into_styled(PrimitiveStyle::with_stroke(RgbColor::WHITE, 10));
    let line2 = Line::new(Point::new(100, 20), Point::new(160, 20))
        .into_styled(PrimitiveStyle::with_stroke(RgbColor::WHITE, 10));
    let line3 = Line::new(Point::new(100, 105), Point::new(160, 105))
        .into_styled(PrimitiveStyle::with_stroke(RgbColor::WHITE, 10));

    // triangle to be shown "in the scroll zone"
    let triangle = Triangle::new(Point::new(240, 100), Point::new(240, 140), Point::new(320, 120))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN));

    // draw the "F" + scroll-section arrow triangle
    line1.draw(&mut lcd).unwrap();
    line2.draw(&mut lcd).unwrap();
    line3.draw(&mut lcd).unwrap();
    triangle.draw(&mut lcd).unwrap();

    let raw_image_data = ImageRawLE::new(include_bytes!("../../assets/ferris.raw"), 86);
    let ferris = Image::new(&raw_image_data, Point::new(60, 60));
    ferris.draw(&mut lcd).unwrap();

    // let zero_data = include_bytes!("../../assets/zero.bmp");
    // let zero_bmp = Bmp::<Rgb888>::from_slice(zero_data).unwrap();
    // let zero = Image::new(&zero_bmp, Point::new(60, 60));
    // zero.draw(&mut lcd).unwrap();

    // Create a small and a large character style.
    let small_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    let large_style = MonoTextStyle::new(&FONT_10X20, Rgb565::RED);
    let mut_style = MonoTextStyle::new(&FONT_10X20, Rgb565::YELLOW);

    // Draw the first text at (20, 30) using the small character style.
    let next = Text::new("Hello ", Point::new(120, 120), small_style)
        .draw(&mut lcd)
        .unwrap();

    // Draw the second text after the first text using the large character style.
    let _next = Text::new("Rust", next, large_style).draw(&mut lcd).unwrap();

    let mut scroll = 1u16; // absolute scroll offset
    let mut direction = true; // direction
    let scroll_delay = 20u32; // delay between steps
    let mut counter = 0u32;
    let rectangle = Rectangle::new(Point::new(0, 180), Size::new(240, 100));
    loop {
        Timer::after_millis(scroll_delay.into()).await;
        let text: String<32> = String::try_from(counter).unwrap();
        lcd.fill_solid(&rectangle, Rgb565::BLACK).unwrap();
        let _ = Text::new(text.as_str(), Point::new(200, 200), mut_style)
            .draw(&mut lcd)
            .unwrap();
        lcd.set_scroll_offset(scroll).unwrap();
        counter += 1;

        if scroll % 80 == 0 {
            direction = !direction;
        }

        match direction {
            true => scroll += 1,
            false => scroll -= 1,
        }
    }
}
