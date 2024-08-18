#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::rng::Rng;
use embassy_stm32::{bind_interrupts, peripherals, rng, Config};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    HASH_RNG => rng::InterruptHandler<peripherals::RNG>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.hsi48 = Some(Default::default()); // needed for RNG
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let mut rng = Rng::new(p.RNG, Irqs);

    let mut buf = [0u8; 16];

    loop {
        unwrap!(rng.async_fill_bytes(&mut buf).await);
        info!("random bytes: {:02x}", buf);
        Timer::after_millis(1000).await;
    }
}
