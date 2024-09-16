#![no_std]
#![no_main]

use core::fmt::Write;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{AnyPin, Level, Output, Pin, Speed};
use embassy_stm32::mode::Async;
//use static_cell::StaticCell;
use embassy_stm32::spi::{Config as SpiConfig, Spi};
use embassy_stm32::usart::{Config, Uart};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use embassy_time::{Duration, Timer};
use heapless::String;
use {defmt_rtt as _, icm426xx, panic_probe as _};

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
async fn usart_task1() {
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
        core::write!(&mut s, "Hello USART Task1: {}!\r\n", n).unwrap();

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
        core::write!(&mut s, "Hello USART Task2: {}!\r\n", n).unwrap();

        usart.write(s.as_bytes()).await.ok();
        Timer::after(Duration::from_millis(1000)).await;
    }
}

// it's hard to pass the compiler...
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

#[embassy_executor::task]
async fn usart_task4(mut usart: Uart<'static, Async>) {
    //for n in 0.. {
    let mut s: String<128> = String::new();
    //core::write!(&mut s, "Hello USART Task4: {}!\r\n", n).unwrap();

    //usart.read(unsafe { s.as_mut_vec() }.as_mut_slice()).await.ok();
    //usart.write(s.as_bytes()).await.ok();

    let mut s = [8u8; 8];
    loop {
        usart.read(&mut s).await;
        usart.write(&s).await.ok();
    }
    Timer::after(Duration::from_millis(1000)).await;
    //}
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    //let p = embassy_stm32::init(Default::default());
    let mut config = embassy_stm32::Config::default();
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

    unwrap!(spawner.spawn(blinky(p.PC13.degrade())));

    // valid, but unsafe ?
    //unwrap!(spawner.spawn(usart_task1()));

    // safe & valid, but not flexible enough
    // if we want to change the uart, must update the function
    //unwrap!(spawner.spawn(usart_task2(p.USART1, p.PA10, p.PA9, p.DMA1_CH0, p.DMA1_CH1)));

    // it's hard to pass the compiler...
    //unwrap!(spawner.spawn(usart_task3(p.USART1, p.PA10, p.PA9, p.DMA1_CH0, p.DMA1_CH1)));

    let config = Config::default();
    //let usart = Uart::new(p.USART1, p.PA10, p.PA9, Irqs, p.DMA1_CH0, p.DMA1_CH1, config).unwrap();
    let usart = Uart::new(p.USART1, p.PA10, p.PA9, Irqs, p.DMA1_CH0, p.DMA1_CH1, config).unwrap();
    //let usart = Uart::new(p.USART1, p.PA10, p.PA9, Irqs, p.DMA1_CH0, p.DMA1_CH1, config).unwrap();
    // safe & valid, and flexible, can change usart without update the function
    unwrap!(spawner.spawn(usart_task4(usart)));

    let spi_config = SpiConfig::default();
    let spidev = Spi::new(p.SPI3, p.PB3, p.PB5, p.PB4, p.DMA1_CH3, p.DMA1_CH4, spi_config);

    let mut _icm = icm426xx::ICM42688::new(spidev);

    loop {
        info!("Hello, World!");
        Timer::after(Duration::from_millis(5000)).await;
    }
}
