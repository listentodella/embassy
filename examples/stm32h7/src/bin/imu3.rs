#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::block_on;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::time::mhz;
use embassy_stm32::{spi as stm32_spi, Config};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::once_lock::OnceLock;
use embassy_time::{Delay, Timer};
use embedded_hal_bus::spi;
use icm426xx::ICM42688;
use {defmt_rtt as _, panic_probe as _};

static IMU: OnceLock<
    Mutex<
        ThreadModeRawMutex,
        ICM42688<
            spi::ExclusiveDevice<stm32_spi::Spi<'static, embassy_stm32::mode::Async>, Output<'static>, Delay>,
            icm426xx::Ready,
        >,
    >,
> = OnceLock::new();

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

    let mut spi_config = stm32_spi::Config::default();
    spi_config.frequency = mhz(16);

    // PB7-CS, PB8-int1, PB9-int2
    let spi = stm32_spi::Spi::new(p.SPI3, p.PB3, p.PB5, p.PB4, p.DMA1_CH3, p.DMA1_CH4, spi_config);
    let cspin = Output::new(p.PB7, Level::High, Speed::High);

    let spidev = spi::ExclusiveDevice::new(spi, cspin, Delay).unwrap();
    let icm = ICM42688::new(spidev);
    let icm = icm.initialize(Delay).await.unwrap();
    let f = async { IMU.init(Mutex::<ThreadModeRawMutex, _>::new(icm)) };
    let _ = block_on(f);

    _spawner.spawn(watchdog_task()).unwrap();
    loop {
        {
            let mut imu = IMU.get().await.lock().await;
            let mut bank = imu.ll().bank::<{ icm426xx::register_bank::BANK0 }>();
            let who_am_i = bank.who_am_i().async_read().await.unwrap().value();
            info!("read chip id = 0x{:x}", who_am_i);
        }

        Timer::after_millis(1000).await;
    }
}

#[embassy_executor::task]
async fn watchdog_task() {
    loop {
        {
            let mut imu = IMU.get().await.lock().await;
            let mut bank = imu.ll().bank::<{ icm426xx::register_bank::BANK0 }>();
            let who_am_i = bank.who_am_i().async_read().await.unwrap().value();
            warn!("wdg read chip id = 0x{:x}", who_am_i);
        }
        Timer::after_millis(1000).await;
    }
}
