#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use embedded_alloc::LlffHeap as Heap;
use {defmt_rtt as _, panic_probe as _};
#[global_allocator]
static HEAP: Heap = Heap::empty();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024 * 8;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe {
            HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE);
        }
    }

    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut str = "Hello Alloc".to_string();
    str += " Happy";
    info!("str len = {}", str.len());

    let mut vec = Vec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3u8);
    info!("vec len = {}", vec.len());
    vec.clear();
    info!("vec len = {}", vec.len());

    // for i in 0..(1024 - str.len()) {
    // for i in 0..256u32 + 128 {
    for i in 0..1024u32 {
        vec.push(i.wrapping_add(0) as u8);
    }
    info!(
        "vec len = {}, min = {}, max = {}",
        vec.len(),
        vec.iter().min(),
        vec.iter().max()
    );

    let mut led = Output::new(p.PC13, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(1000).await;

        info!("low");
        led.set_low();
        Timer::after_millis(1000).await;
    }
}
