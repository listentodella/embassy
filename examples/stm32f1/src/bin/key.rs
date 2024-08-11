#![no_std]
#![no_main]

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut led = Output::new(p.PC13, Level::High, Speed::Low);

    // my bluepill has no normal key
    // so I use PB12 and PA8 to simulate a key
    let _out = Output::new(p.PB12, Level::Low, Speed::Low);
    //let mut key = Input::new(p.PA8, Pull::Down);
    let mut key = ExtiInput::new(p.PA8, p.EXTI8, Pull::Up);

    loop {
        info!("waiting for any edge");
        key.wait_for_any_edge().await;
        error!("trigger!");
        led.toggle();
    }
}
