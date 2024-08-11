#![no_std]
#![no_main]

use core::fmt::Write;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::usart::{Config as UsartConfig, UartTx};
use embassy_stm32::Config;
use embassy_time::{Instant, Timer};
use heapless::String;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());

    let mut msg: String<32> = String::new();
    let mut usart = UartTx::new_blocking(p.USART1, p.PA9, UsartConfig::default()).unwrap();

    info!("usart tx only");
    loop {
        let ts = Instant::now().as_micros();

        core::writeln!(&mut msg, "{}:hello usart", ts).unwrap();

        info!("{}", msg.as_str());
        usart.blocking_write(msg.as_bytes()).unwrap();

        Timer::after_secs(1).await;

        msg.clear();
    }
}
