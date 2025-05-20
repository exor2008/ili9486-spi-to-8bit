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
    peripherals::{PIO0, PIO1},
    pio::{InterruptHandler, Pio},
};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Duration;
use embassy_time::Timer;
use ili9486display::parallel_out::PioParallel8;
use ili9486display::pio_spi_slave::SpiIn;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    PIO1_IRQ_0 => InterruptHandler<PIO1>;
});

const PACKET_SIZE: usize = 1000;
const QUEUE_SIZE: usize = 2;
static CHANNEL: Channel<ThreadModeRawMutex, [u16; PACKET_SIZE], QUEUE_SIZE> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
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
        p.PIN_16, // miso
        p.PIN_17, // mosi
        p.PIN_18, // dc
        p.PIN_19, // clk
        p.PIN_20, // cs
    );

    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO1, Irqs);

    let display = PioParallel8::new(
        &mut common,
        sm0,
        p.DMA_CH1,
        p.PIN_2,
        p.PIN_3,
        p.PIN_4,
        p.PIN_5,
        p.PIN_6,
        p.PIN_7,
        p.PIN_8,
        p.PIN_9,
        p.PIN_10,
        p.PIN_11,
        p.PIN_12,
        p.PIN_13,
    );

    // let mut reset = Output::new(p.PIN_14, Level::Low);
    // reset.set_low();
    // Timer::after(Duration::from_millis(10)).await;
    // reset.set_high();
    // Timer::after(Duration::from_millis(120)).await;
    info!("Begin");

    unwrap!(spawner.spawn(send_task(display)));

    let mut data = [0; PACKET_SIZE];

    // for _ in 0..1000 {
    loop {
        spi_in.read(&mut data).await;
        CHANNEL.try_send(data).expect("Buffer if full");
        // info!("+");
    }
    // info!("Read 1000");

    // for j in 0..1000 {
    //     let mut d = 0;
    //     d |= data[j] & 0b1;
    //     d |= (data[j] & 0b100) >> 1;
    //     d |= (data[j] & 0b10000) >> 2;
    //     d |= (data[j] & 0b1000000) >> 3;
    //     d |= (data[j] & 0b100000000) >> 4;
    //     d |= (data[j] & 0b10000000000) >> 5;
    //     d |= (data[j] & 0b1000000000000) >> 6;
    //     d |= (data[j] & 0b100000000000000) >> 7;

    //     let mut dc = 0;
    //     dc |= (data[j] & 0b10) >> 1;
    //     dc |= (data[j] & 0b1000) >> 2;
    //     dc |= (data[j] & 0b100000) >> 3;
    //     dc |= (data[j] & 0b10000000) >> 4;
    //     dc |= (data[j] & 0b1000000000) >> 5;
    //     dc |= (data[j] & 0b100000000000) >> 6;
    //     dc |= (data[j] & 0b10000000000000) >> 7;
    //     dc |= (data[j] & 0b1000_0000_0000_0000) >> 8;

    //     let status = if dc == 0 {
    //         "Command"
    //     } else if dc == 0xff {
    //         "Data"
    //     } else {
    //         "Err"
    //     };

    //     //     if dc == 0 {
    //     //         if [0x11, 0x3a, 0xb0, 0x29].contains(&d) {
    //     //             // info!("{:#x} #{}", data[j], j);
    //     info!("{} - {:#x} ({:032b}) #{}", status, d, data[j], j);
    //     //         }
    //     Timer::after_millis(10).await;
    //     //     }
    // }

    // info!("End");
    // loop {}
    // }

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

#[embassy_executor::task]
async fn send_task(mut display: PioParallel8<'static, PIO1, 0>) -> ! {
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
    // info!("Coco");

    loop {
        let mut data = CHANNEL.receive().await;
        convert(&mut data);
        display.write(&data).await;

        if data.iter().any(|d| *d == 0x11) {
            info!("Got sleep out");
        }
        if data.iter().any(|d| *d == 0x13) {
            info!("Got normal mode");
        }
        if data.iter().any(|d| *d == 0x2a) {
            info!("Got set column");
        }

        // for i in 0..PACKET_SIZE {
        // info!("{:032b}", i);
        // Timer::after_millis(10).await;
        // }
        // info!("-");

        // loop {
        // Timer::after_millis(1).await;
    }
}

fn convert(data: &mut [u16]) {
    for i in data.iter_mut() {
        let mut new = 0;
        let orig = *i;
        new |= orig & 0b1;
        new |= (orig & 0b100) >> 1;
        new |= (orig & 0b10000) >> 2;
        new |= (orig & 0b1000000) >> 3;
        new |= (orig & 0b100000000) >> 4;
        new |= (orig & 0b10000000000) >> 5;
        new |= (orig & 0b1000000000000) >> 6;
        new |= (orig & 0b100000000000000) >> 7;
        new |= (orig & 0b010) << 7;
        new = match new {
            0x2a => 0x2b,
            0x2b => 0x2a,
            _ => new,
        };
        *i = new;
    }
}
