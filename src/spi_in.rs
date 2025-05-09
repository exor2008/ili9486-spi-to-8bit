use embassy_rp::dma::{AnyChannel, Channel};
use embassy_rp::gpio::Level;
use embassy_rp::pio::{
    Common, Config, Direction, FifoJoin, Instance, PioPin, ShiftConfig, ShiftDirection,
    StateMachine,
};
use embassy_rp::{into_ref, Peripheral, PeripheralRef};
use fixed::types::U24F8;

pub struct SpiIn<'a, P: Instance, const N: usize> {
    dma: PeripheralRef<'a, AnyChannel>,
    sm: StateMachine<'a, P, N>,
}

impl<'a, P: Instance, const N: usize> SpiIn<'a, P, N> {
    pub fn new(
        pio: &mut Common<'a, P>,
        mut sm: StateMachine<'a, P, N>,
        dma: impl Peripheral<P = impl Channel> + 'a,
        miso: impl PioPin,
        mosi: impl PioPin,
        clk: impl PioPin,
        cs: impl PioPin,
        dc: impl PioPin,
    ) -> SpiIn<'a, P, N> {
        into_ref!(dma);

        let mosi = pio.make_pio_pin(mosi);
        let miso = pio.make_pio_pin(miso);
        let clk = pio.make_pio_pin(clk);
        let cs = pio.make_pio_pin(cs);
        let dc = pio.make_pio_pin(dc);

        let prg_command = pio::pio_asm!(
            r#"
            .side_set 1 opt
            .wrap_target
                wait 0 pin 2; // waiting for falling edge on Chip Select
            loop:
                wait 0 pin 1; // waiting for the rising edge on clock
                wait 1 pin 1; // waiting for the falling edge on clock
                in pins 1 side 0; // read data from master
                jmp pin end;
                jmp loop;
            end:
            .wrap
            "#,
        );

        sm.set_pin_dirs(Direction::In, &[&mosi, &clk, &cs, &dc]);
        sm.set_pin_dirs(Direction::Out, &[&miso]);

        sm.set_pins(Level::Low, &[&miso]);

        let mut cfg = Config::default();
        cfg.use_program(&pio.load_program(&prg_command.program), &[&miso]);
        cfg.set_in_pins(&[&mosi, &clk, &cs, &dc]);
        cfg.set_out_pins(&[&miso]);
        cfg.set_jmp_pin(&cs);

        cfg.shift_in = ShiftConfig {
            auto_fill: true,
            direction: ShiftDirection::Left,
            threshold: 8,
        };

        cfg.fifo_join = FifoJoin::Duplex;
        cfg.clock_divider = U24F8::from_num(1.0);

        sm.set_config(&cfg);
        sm.set_enable(true);

        SpiIn {
            dma: dma.map_into(),
            sm,
        }
    }
}

impl<'a, P: Instance, const N: usize> SpiIn<'a, P, N> {
    pub async fn read(&mut self, data: &mut [u32]) {
        self.sm
            .rx()
            .dma_pull(self.dma.reborrow(), data, false)
            .await;
    }
}
