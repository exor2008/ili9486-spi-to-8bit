use embassy_rp::dma::{AnyChannel, Channel};
use embassy_rp::gpio::Level;
use embassy_rp::pio::{
    Common, Config, Direction, FifoJoin, Instance, PioPin, ShiftConfig, ShiftDirection,
    StateMachine,
};
use embassy_rp::Peri;
use fixed::types::U24F8;

pub struct PioParallel8<'a, P: Instance, const N: usize> {
    dma: Peri<'a, AnyChannel>,
    sm: StateMachine<'a, P, N>,
}

impl<'a, P: Instance, const N: usize> PioParallel8<'a, P, N> {
    pub fn new(
        pio: &mut Common<'a, P>,
        mut sm: StateMachine<'a, P, N>,
        dma: Peri<'a, impl Channel>,
        db0: Peri<'a, impl PioPin>,
        db1: Peri<'a, impl PioPin>,
        db2: Peri<'a, impl PioPin>,
        db3: Peri<'a, impl PioPin>,
        db4: Peri<'a, impl PioPin>,
        db5: Peri<'a, impl PioPin>,
        db6: Peri<'a, impl PioPin>,
        db7: Peri<'a, impl PioPin>,
        dc: Peri<'a, impl PioPin>,
        cs: Peri<'a, impl PioPin>,
        rd: Peri<'a, impl PioPin>,
        wr: Peri<'a, impl PioPin>,
    ) -> PioParallel8<'a, P, N> {
        let db0 = pio.make_pio_pin(db0);
        let db1 = pio.make_pio_pin(db1);
        let db2 = pio.make_pio_pin(db2);
        let db3 = pio.make_pio_pin(db3);
        let db4 = pio.make_pio_pin(db4);
        let db5 = pio.make_pio_pin(db5);
        let db6 = pio.make_pio_pin(db6);
        let db7 = pio.make_pio_pin(db7);

        let dc = pio.make_pio_pin(dc);
        let cs = pio.make_pio_pin(cs);
        let rd = pio.make_pio_pin(rd);
        let wr = pio.make_pio_pin(wr);

        let prg_command = pio::pio_asm!(
            r#"
            .side_set 3 opt
            .wrap_target
                nop side 0b010 ; wr - ON rd - OFF cs - ON
                out pins 16
                nop side 0b110 ; wr - OFF rd - OFF cs - ON
            .wrap
            "#,
        );

        sm.set_pin_dirs(
            Direction::Out,
            &[
                &db0, &db1, &db2, &db3, &db4, &db5, &db6, &db7, &dc, &cs, &rd, &wr,
            ],
        );
        sm.set_pins(Level::High, &[&cs, &rd, &wr]);
        sm.set_pins(Level::Low, &[&dc]);

        let mut cfg = Config::default();
        cfg.use_program(&pio.load_program(&prg_command.program), &[&cs, &rd, &wr]);
        cfg.set_out_pins(&[&db0, &db1, &db2, &db3, &db4, &db5, &db6, &db7, &dc]);

        cfg.shift_out = ShiftConfig {
            auto_fill: true,
            direction: ShiftDirection::Left,
            threshold: 16,
        };

        cfg.fifo_join = FifoJoin::TxOnly;
        cfg.clock_divider = U24F8::from_num(4.0);
        sm.set_config(&cfg);
        sm.set_enable(true);

        PioParallel8 {
            dma: dma.into(),
            sm,
        }
    }
}

impl<'a, P: Instance, const N: usize> PioParallel8<'a, P, N> {
    pub async fn write(&mut self, data: &[u16]) {
        self.sm
            .tx()
            .dma_push(self.dma.reborrow(), data, false)
            .await;
    }
}
