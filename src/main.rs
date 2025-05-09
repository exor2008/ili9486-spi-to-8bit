#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Pull};
use embassy_rp::{
    bind_interrupts,
    gpio::Input,
    peripherals::PIO0,
    pio::{InterruptHandler, Pio},
};
use heapless::Vec;
use ili9486display::spi_in::SpiIn;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Started");
    let p = embassy_rp::init(Default::default());

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
        info!("Read: {:#x}", data);
    }

    // let clk = Input::new(p.PIN_12, Pull::Down);
    // let cs = Input::new(p.PIN_13, Pull::Down);

    // let mut acc = false;
    // let mut prev_clocl = false;

    // loop {
    //     if cs.get_level() == Level::Low {
    //         acc = true;

    //     } else if acc {
    //         acc = false;
    //     }
    // }

    // let s = Spi
}
