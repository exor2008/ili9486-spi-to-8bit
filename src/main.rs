#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::clocks::ClockConfig;
use embassy_rp::config::Config;
use embassy_rp::{
    bind_interrupts,
    peripherals::{PIO0, PIO1},
    pio::{InterruptHandler, Pio},
};
use ili9486_spi_to_8bit::pio_converter::PioConverter;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    PIO1_IRQ_0 => InterruptHandler<PIO1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Started");
    let config = ClockConfig::system_freq(200_000_000);
    let cfg = Config::new(config);
    let p = embassy_rp::init(cfg);

    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0, Irqs);

    let mut pio_converter = PioConverter::new(
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
        pio_converter.work().await;
    }
}
