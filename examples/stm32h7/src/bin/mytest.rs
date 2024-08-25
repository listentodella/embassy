#![no_std]
#![no_main]

use core::fmt::Write;
use defmt::*;
use embassy_executor::{Executor, Spawner};
use embassy_stm32::gpio::{AnyPin, Level, Output, Pin, Speed};
use embassy_stm32::usart::{Config, RxPin, TxPin, Uart};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use embassy_time::{Duration, Timer};
//use static_cell::StaticCell;
use heapless::String;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

#[embassy_executor::task]
async fn blinky(pin: AnyPin) {
    let mut led = Output::new(pin, Level::High, Speed::Low);
    loop {
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;
        led.set_low();
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
async fn usart_task() {
    let config = Config::default();
    let usart = unsafe { peripherals::USART1::steal() };
    let mut usart = unsafe {
        Uart::new(
            usart,
            peripherals::PA10::steal(),
            peripherals::PA9::steal(),
            Irqs,
            peripherals::DMA1_CH0::steal(),
            peripherals::DMA1_CH1::steal(),
            config,
        )
    }
    .unwrap();

    for n in 0.. {
        let mut s: String<128> = String::new();
        core::write!(&mut s, "Hello DMA World {}!\r\n", n).unwrap();

        usart.write(s.as_bytes()).await.ok();
        Timer::after(Duration::from_millis(1000)).await;
    }
}

#[embassy_executor::task]
async fn usart_task2(
    uart: peripherals::USART1,
    pin1: peripherals::PA10,
    pin2: peripherals::PA9,
    dma1_ch0: peripherals::DMA1_CH0,
    dma1_ch1: peripherals::DMA1_CH1,
) {
    let config = Config::default();
    let mut usart = Uart::new(uart, pin1, pin2, Irqs, dma1_ch0, dma1_ch1, config).unwrap();

    for n in 0.. {
        let mut s: String<128> = String::new();
        core::write!(&mut s, "Hello USART World {}!\r\n", n).unwrap();

        usart.write(s.as_bytes()).await.ok();
        Timer::after(Duration::from_millis(1000)).await;
    }
}

// #[embassy_executor::task]
// async fn usart_task3(
//     uart: peripherals::USART1,
//     pin1: impl RxPin,
//     pin2: impl TxPin,
//     dma1_ch0: peripherals::DMA1_CH0,
//     dma1_ch1: peripherals::DMA1_CH1,
// ) {
//     let config = Config::default();
//     let mut usart = Uart::new(uart, pin1, pin2, Irqs, dma1_ch0, dma1_ch1, config).unwrap();

//     for n in 0.. {
//         let mut s: String<128> = String::new();
//         core::write!(&mut s, "Hello USART World {}!\r\n", n).unwrap();

//         usart.write(s.as_bytes()).await.ok();
//         Timer::after(Duration::from_millis(1000)).await;
//     }
// }

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    unwrap!(spawner.spawn(blinky(p.PC13.degrade())));

    // valid, but unsafe ?
    //unwrap!(spawner.spawn(usart_task()));

    // valid
    unwrap!(spawner.spawn(usart_task2(p.USART1, p.PA10, p.PA9, p.DMA1_CH0, p.DMA1_CH1)));

    //unwrap!(spawner.spawn(usart_task3(p.USART1, p.PA10, p.PA9, p.DMA1_CH0, p.DMA1_CH1)));

    loop {
        info!("Hello, World!");
        Timer::after(Duration::from_millis(5000)).await;
    }
}
