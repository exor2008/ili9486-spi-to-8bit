#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::clocks::{ClockConfig, RoscRng};
use embassy_rp::config::Config;
use embassy_rp::gpio::{Level, Output, Pull};
use embassy_rp::spi::{Config as SpiConfig, Phase, Polarity, Spi};
use embassy_rp::{
    bind_interrupts,
    gpio::Input,
    peripherals::{PIO0, PIO1},
    pio::{InterruptHandler, Pio},
};
use embassy_time::Duration;
use embassy_time::Timer;
use ili9486display::pio_spi_slave::SpiIn;
use rand_core::RngCore;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    PIO1_IRQ_0 => InterruptHandler<PIO1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Started");
    // let config = ClockConfig::system_freq(200_000_000);
    // let cfg = Config::new(config);
    // let p = embassy_rp::init(cfg);
    let p = embassy_rp::init(Default::default());

    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0, Irqs);

    let mut spi_in = SpiIn::new(
        &mut common,
        sm0,
        p.DMA_CH0,
        p.PIN_17, // mosi
        p.PIN_18, // dc
        p.PIN_19, // clk
        p.PIN_20, // cs
        p.PIN_2,  // out 0
        p.PIN_3,  // out 1
        p.PIN_4,  // out 2
        p.PIN_5,  // out 3
        p.PIN_6,  // out 4
        p.PIN_7,  // out 5
        p.PIN_8,  // out 6
        p.PIN_9,  // out 7
        p.PIN_10, // dc out
        p.PIN_11, // cs out
        p.PIN_12, // read
        p.PIN_13, // write
    );

    info!("Begin");

    loop {
        spi_in.work().await;
    }

    // let miso = p.PIN_12;
    // let mosi = p.PIN_11;
    // let clk = p.PIN_10;
    // let mut cs = Output::new(p.PIN_13, Level::High);
    // let mut dc = Output::new(p.PIN_14, Level::Low);

    // let mut cfg = SpiConfig::default();
    // cfg.frequency = 16_000_000;

    // let mut spi = Spi::new(p.SPI1, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, cfg);

    // cs.set_high();
    // dc.set_low();

    // Timer::after_secs(5).await;

    // cs.set_low();
    // spi.write(&[0x0, 0x11]).await.unwrap();
    // spi.write(&[0x0, 0x29]).await.unwrap();

    // spi.write(&[0x0, 0x3a]).await.unwrap();
    // dc.set_high();
    // spi.write(&[0x0, 0x55]).await.unwrap();
    // dc.set_low();

    // // cs.set_high();

    // loop {
    //     spi.write(&[0x0, 0x2a]).await.unwrap();
    //     dc.set_high();
    //     // spi.write(&[0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x3f])
    //     spi.write(&[0x0, 0x0, 0x0, 10, 0x0, 0x0, 0x0, 0xff])
    //         .await
    //         .unwrap();
    //     dc.set_low();

    //     spi.write(&[0x0, 0x2b]).await.unwrap();
    //     dc.set_high();
    //     // spi.write(&[0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0xdf])
    //     spi.write(&[0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xff])
    //         .await
    //         .unwrap();
    //     dc.set_low();

    //     let mut rng = RoscRng;
    //     let mut pixels = [0; 320 * 480];
    //     rng.fill_bytes(&mut pixels);

    //     spi.write(&[0x0, 0x2c]).await.unwrap();
    //     dc.set_high();
    //     spi.write(&pixels).await.unwrap();
    //     dc.set_low();
    // }
}

// display.write(&[0x3a, 0x55 | 256]).await; // pixel format
// display.write(&[0x11]).await; // sleep out
// display.write(&[0x20]).await; // inversion off
// display.write(&[0x29]).await; // display on
// display.write(&[0x38]).await; // idle mode off

// display
//     .write(&[
//         0x2a,
//         1 | 256,
//         0x35 | 256,
//         1 | 256,
//         0x3f | 256,
//         // 1 | 256,
//         // 0xd5 | 256,
//         // 1 | 256,
//         // 0xdf | 256,
//     ])
//     .await; // set column

// display
//     .write(&[
//         0x2b,
//         0 | 256,
//         10 | 256,
//         0 | 256,
//         20 | 256,
//         // 1 | 256,
//         // 0x35 | 256,
//         // 1 | 256,
//         // 0x3f | 256,
//     ])
//     .await; // set page

// display.write(&[0x2c]).await; // memory write
// display.write(&[0b1111_1111 | 256; 200]).await; // memory write

// loop {
//     //     // spi.write(&[0, 0b1111_0000]).await.unwrap();
//     //     // for i in 1..=5 {
//     //     //     cs.set_low();
//     //     //     // Timer::after_millis(100).await;
//     //     //     let tx_buf = [i];
//     //     //     let mut rx_buf = [0_u8; 1];
//     //     //     dc.set_low();
//     //     //     spi.transfer(&mut rx_buf, &tx_buf).await.unwrap();
//     //     //     info!("Command {}", i);
//     //     //     dc.set_high();
//     //     //     for j in 0..i {
//     //     //         let tx_buf = [j];
//     //     //         spi.transfer(&mut rx_buf, &tx_buf).await.unwrap();
//     //     //         info!("Data {}", j);
//     //     //     }
//     //     //     cs.set_high();
//     //     //     Timer::after_secs(1).await;
//     //     // }
//     //     // Timer::after_secs(5).await;
// }
// }
