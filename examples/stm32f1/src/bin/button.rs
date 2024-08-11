#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{AnyPin, Level, Output, Pin, Pull, Speed};
use embassy_stm32::Config;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

static BLINK_MS: AtomicU32 = AtomicU32::new(0);

#[embassy_executor::task]
async fn led_task(led: AnyPin) {
    let mut led = Output::new(led, Level::Low, Speed::Low);

    loop {
        let delay_ms = BLINK_MS.load(Ordering::Relaxed);
        error!("adjust delay_ms to {}", delay_ms);
        Timer::after(Duration::from_millis(delay_ms.into())).await;
        led.toggle();
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());

    // my bluepill has no normal key
    // so I use PB12 and PA8 to simulate a key
    let _out = Output::new(p.PB12, Level::Low, Speed::Low);
    //let mut button = Input::new(p.PA8, Pull::Down);
    let mut button = ExtiInput::new(p.PA8, p.EXTI8, Pull::Up);

    BLINK_MS.store(2000, Ordering::Relaxed);

    spawner.spawn(led_task(p.PC13.degrade())).unwrap();

    loop {
        info!("wait_for_rising_edge");
        button.wait_for_rising_edge().await;
        warn!("trigger!");

        let mut delay = BLINK_MS.load(Ordering::Relaxed);
        delay -= 100;
        if delay < 500 {
            delay = 2000;
        }
        BLINK_MS.store(delay, Ordering::Relaxed);
    }
}
