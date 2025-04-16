#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::time::mhz;
use embassy_stm32::{spi as stm32_spi, Config};
use embassy_stm32h7_examples::sensorhub::{Attr, PhysicalSensor, SensorOps, SensorType};
use embassy_time::{Delay, Timer};
use embedded_hal_1::spi::{Operation, SpiDevice};
use embedded_hal_bus::spi;
use heapless::{String, Vec};
use icm426xx::ICM42688;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    //let mut x = core::mem::MaybeUninit::uninit();

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

    info!("Hello World!");
    let sensor_name = String::try_from("icm42688_dev.acc").unwrap();
    let vendor_name = String::try_from("bosch").unwrap();
    let mut icm42688_dev = PhysicalSensor::new(SensorType::Accelermeter, 0, sensor_name, vendor_name);

    icm42688_dev.publish_default_attributes();

    let attr_name = String::try_from("rates").unwrap();
    let val = Vec::from_slice(&[12, 25, 50, 100, 200, 400, 800, 1600]).unwrap();
    icm42688_dev.update_attributes(attr_name, Attr::Rates(val));

    let attr_name = String::try_from("ranges").unwrap();
    let val = Vec::from_slice(&[(-16, 16), (-8, 8), (-4, 4), (-2, 2)]).unwrap();
    icm42688_dev.update_attributes(attr_name, Attr::Ranges(val));

    icm42688_dev.open(120);

    let mut spi_config = stm32_spi::Config::default();
    spi_config.frequency = mhz(16);

    // PB7-CS, PB8-int1, PB9-int2
    //let mut spi = stm32_spi::Spi::new(p.SPI3, p.PB3, p.PB5, p.PB4, p.DMA1_CH3, p.DMA1_CH4, spi_config);
    let mut spi = stm32_spi::Spi::new_blocking(p.SPI3, p.PB3, p.PB5, p.PB4, spi_config);
    let mut cspin = Output::new(p.PB7, Level::High, Speed::High);

    let mut spidev = spi::ExclusiveDevice::new_no_delay(spi, cspin).unwrap();

    let mut icm = ICM42688::new(spidev);
    let mut icm: ICM42688<
        spi::ExclusiveDevice<stm32_spi::Spi<'_, embassy_stm32::mode::Blocking>, Output<'_>, spi::NoDelay>,
        icm426xx::Ready,
    > = icm.initialize(Delay).unwrap();
    let mut bank = icm.ll().bank::<{ icm426xx::register_bank::BANK0 }>();

    let who_am_i = bank.who_am_i().read().unwrap().value();
    info!("read chip id = 0x{:x}", who_am_i);
    loop {
        //let x = bank.accel_data_x0().read().unwrap().value();
        //let mut x = bank.accel_data_x0().read().unwrap();
        let (x0, y0, z0) = (
            bank.accel_data_x0().read().unwrap().accel_data_x_7_0(),
            bank.accel_data_y0().read().unwrap().accel_data_y_7_0(),
            bank.accel_data_z0().read().unwrap().accel_data_z_7_0(),
        );
        let (x1, y1, z1) = (
            bank.accel_data_x1().read().unwrap().accel_data_x_15_8(),
            bank.accel_data_y1().read().unwrap().accel_data_y_15_8(),
            bank.accel_data_z1().read().unwrap().accel_data_z_15_8(),
        );
        let (x, y, z) = (
            x0 as u16 | (x1 as u16) << 8,
            y0 as u16 | (y1 as u16) << 8,
            z0 as u16 | (z1 as u16) << 8,
        );
        let (x, y, z) = (x as i16, y as i16, z as i16);

        info!("{}", (x, y, z));
        Timer::after_millis(1000).await;
    }
}
