use embassy_rp::dma::{AnyChannel, Channel};
use embassy_rp::gpio::Level;
use embassy_rp::pio::{
    Common, Config, Direction, FifoJoin, Instance, PioPin, ShiftConfig, ShiftDirection,
    StateMachine,
};
use embassy_rp::Peri;
use fixed::types::U24F8;

pub struct SpiIn<'a, P: Instance, const N: usize> {
    dma: Peri<'a, AnyChannel>,
    sm: StateMachine<'a, P, N>,
}

impl<'a, P: Instance, const N: usize> SpiIn<'a, P, N> {
    pub fn new(
        pio: &mut Common<'a, P>,
        mut sm: StateMachine<'a, P, N>,
        dma: Peri<'a, impl Channel>,
        miso: Peri<'a, impl PioPin>,
        mosi: Peri<'a, impl PioPin>,
        dc: Peri<'a, impl PioPin>,
        clk: Peri<'a, impl PioPin>,
        cs: Peri<'a, impl PioPin>,
    ) -> SpiIn<'a, P, N> {
        let miso = pio.make_pio_pin(miso);
        let mosi = pio.make_pio_pin(mosi);
        let dc = pio.make_pio_pin(dc);
        let clk = pio.make_pio_pin(clk);
        let cs = pio.make_pio_pin(cs);

        let prg_command = pio::pio_asm!(
            r#"
            .side_set 1 opt

            drop:
                mov ISR, NULL;
                jmp check_cs;
            .wrap_target
                wait 0 pin 2;
                wait 1 pin 2;
                in pins 2;
            check_cs:
                jmp pin drop;
            .wrap
            "#,
        );

        sm.set_pin_dirs(Direction::In, &[&mosi, &dc, &clk, &cs]);
        sm.set_pin_dirs(Direction::Out, &[&miso]);

        sm.set_pins(Level::Low, &[&miso]);

        let mut cfg = Config::default();
        cfg.use_program(&pio.load_program(&prg_command.program), &[&miso]);
        cfg.set_in_pins(&[&mosi, &dc, &clk, &cs]);
        cfg.set_out_pins(&[&miso]);
        cfg.set_jmp_pin(&cs);

        cfg.shift_in = ShiftConfig {
            auto_fill: true,
            direction: ShiftDirection::Left,
            threshold: 32,
        };

        cfg.fifo_join = FifoJoin::Duplex;
        cfg.clock_divider = U24F8::from_num(1.0);

        sm.set_config(&cfg);
        sm.set_enable(true);

        SpiIn {
            dma: dma.into(),
            sm,
        }
    }
}

impl<'a, P: Instance, const N: usize> SpiIn<'a, P, N> {
    pub async fn read(&mut self, data: &mut [u16]) {
        self.sm
            .rx()
            .dma_pull(self.dma.reborrow(), data, false)
            .await;
    }
}
