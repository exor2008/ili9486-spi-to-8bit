#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::clocks::ClockConfig;
use embassy_rp::config::Config;
use embassy_rp::gpio::{Level, Output, Pull};
use embassy_rp::spi::{Config as SpiConfig, Phase, Polarity, Spi};
use embassy_rp::{
    bind_interrupts,
    gpio::Input,
    peripherals::PIO0,
    pio::{InterruptHandler, Pio},
};
use embassy_time::Timer;
use heapless::Vec;
use ili9486display::spi_in::SpiIn;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Started");
    let config = ClockConfig::system_freq(200_000_000);
    let cfg = Config::new(config);
    let p = embassy_rp::init(cfg);
    // let p = embassy_rp::init(Default::default());

    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0, Irqs);

    let mut spi_in = SpiIn::new(
        &mut common,
        sm0,
        p.DMA_CH0,
        p.PIN_10,
        p.PIN_11,
        p.PIN_12,
        p.PIN_13,
        p.PIN_14,
    );

    let mut data = [0u32];

    loop {
        spi_in.read(&mut data).await;
        let command = if (data[0] & 256) == 0 { "Command" } else { "" };
        info!("Read: {:#x} {}", data[0] & 0xFF, command);
    }

    // let miso = p.PIN_12;
    // let mosi = p.PIN_11;
    // let clk = p.PIN_10;
    // let mut cs = Output::new(p.PIN_13, Level::High);
    // let mut dc = Output::new(p.PIN_14, Level::Low);

    // let mut cfg = SpiConfig::default();
    // cfg.frequency = 16_000_000;

    // let mut spi = Spi::new(p.SPI1, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, cfg);

    // loop {
    //     for i in 1..=5 {
    //         cs.set_low();
    //         // Timer::after_millis(100).await;
    //         let tx_buf = [i];
    //         let mut rx_buf = [0_u8; 1];
    //         dc.set_low();
    //         spi.transfer(&mut rx_buf, &tx_buf).await.unwrap();
    //         info!("Command {}", i);
    //         dc.set_high();
    //         for j in 0..i {
    //             let tx_buf = [j];
    //             spi.transfer(&mut rx_buf, &tx_buf).await.unwrap();
    //             info!("Data {}", j);
    //         }
    //         cs.set_high();
    //         Timer::after_secs(1).await;
    //     }
    //     Timer::after_secs(5).await;
    // }
}
