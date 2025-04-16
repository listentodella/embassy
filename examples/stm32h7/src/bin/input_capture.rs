#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::time::khz;
use embassy_stm32::timer::input_capture::{CapturePin, InputCapture};
use embassy_stm32::timer::{self, Channel};
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    TIM15 => timer::CaptureCompareInterruptHandler<peripherals::TIM15>;
});
/// Connect PA2 and PC13 with a 1k Ohm resistor

#[embassy_executor::task]
async fn blinky(led: peripherals::PC13) {
    let mut led = Output::new(led, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(300).await;

        info!("low");
        led.set_low();
        Timer::after_millis(300).await;
    }
}

#[embassy_executor::task]
async fn capture_task(mut ic: InputCapture<'static, peripherals::TIM15>) {
    info!("wait for risign edge");
    ic.wait_for_rising_edge(Channel::Ch1).await;

    let capture_value = ic.get_capture_value(Channel::Ch1);
    info!("new capture! {}", capture_value);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    unwrap!(spawner.spawn(blinky(p.PC13)));

    let ch1 = CapturePin::new_ch1(p.PA2, Pull::None);
    let ic = InputCapture::new(
        p.TIM15,
        Some(ch1),
        None,
        None,
        None,
        Irqs,
        khz(1000),
        Default::default(),
    );

    unwrap!(spawner.spawn(capture_task(ic)));

    loop {
        Timer::after(Duration::from_millis(5000)).await;
    }
}
