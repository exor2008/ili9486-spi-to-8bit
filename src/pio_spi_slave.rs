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
        mosi: Peri<'a, impl PioPin>,
        dc_in: Peri<'a, impl PioPin>,
        clk: Peri<'a, impl PioPin>,
        cs_in: Peri<'a, impl PioPin>,

        db0: Peri<'a, impl PioPin>,
        db1: Peri<'a, impl PioPin>,
        db2: Peri<'a, impl PioPin>,
        db3: Peri<'a, impl PioPin>,
        db4: Peri<'a, impl PioPin>,
        db5: Peri<'a, impl PioPin>,
        db6: Peri<'a, impl PioPin>,
        db7: Peri<'a, impl PioPin>,
        dc_out: Peri<'a, impl PioPin>,
        cs_out: Peri<'a, impl PioPin>,
        rd: Peri<'a, impl PioPin>,
        wr: Peri<'a, impl PioPin>,
    ) -> SpiIn<'a, P, N> {
        let mosi = pio.make_pio_pin(mosi);
        let dc_in = pio.make_pio_pin(dc_in);
        let clk = pio.make_pio_pin(clk);
        let cs_in = pio.make_pio_pin(cs_in);

        let db0 = pio.make_pio_pin(db0);
        let db1 = pio.make_pio_pin(db1);
        let db2 = pio.make_pio_pin(db2);
        let db3 = pio.make_pio_pin(db3);
        let db4 = pio.make_pio_pin(db4);
        let db5 = pio.make_pio_pin(db5);
        let db6 = pio.make_pio_pin(db6);
        let db7 = pio.make_pio_pin(db7);

        let dc_out = pio.make_pio_pin(dc_out);
        let cs_out = pio.make_pio_pin(cs_out);
        let rd = pio.make_pio_pin(rd);
        let wr = pio.make_pio_pin(wr);

        let prg_command = pio::pio_asm!(
            r#"
            .side_set 3 opt
            drop:
                mov ISR NULL
                mov OSR NULL
                jmp pin drop
            .wrap_target

                set X 7
            first8:
                wait 0 pin 2
                wait 1 pin 2
                jmp pin drop
                jmp X-- first8

                wait 0 pin 2
                wait 1 pin 2
                jmp pin drop
                in pins 2

                set X 6
            last7:
                wait 0 pin 2
                wait 1 pin 2
                in pins 1
                jmp pin drop side 0b010
                jmp X-- last7

                mov pins ISR
                nop
                nop
                nop
                nop side 0b110
            .wrap
            "#,
            // r#"
            // .side_set 3 opt

            // drop:
            //     mov ISR NULL
            //     mov OSR NULL
            //     jmp pin drop
            // .wrap_target
            //     set X 8
            // first8:
            //     wait 0 pin 2
            //     // wait 1 pin 2 side 0b110; wr - OFF rd - OFF cs - ON
            //     wait 1 pin 2
            //     jmp pin drop
            //     jmp X-- first8

            //     wait 0 pin 2
            //     wait 1 pin 2
            //     in pins 2

            //     set X 7
            // last7:
            //     wait 0 pin 2
            //     wait 1 pin 2
            //     in pins 1
            //     // in pins 1 side 0b010; wr - ON rd - OFF cs - ON
            //     jmp pin drop
            //     jmp X-- last7

            //     mov pins ISR side 0b010
            // .wrap
            // "#,
        );

        sm.set_pin_dirs(Direction::In, &[&mosi, &dc_in, &clk, &cs_in]);
        sm.set_pin_dirs(
            Direction::Out,
            &[
                &db0, &db1, &db2, &db3, &db4, &db5, &db6, &db7, &dc_out, &cs_out, &rd, &wr,
            ],
        );
        sm.set_pins(Level::High, &[&cs_out, &wr]);
        sm.set_pins(Level::Low, &[&dc_out, &rd]);

        let mut cfg = Config::default();
        cfg.use_program(
            &pio.load_program(&prg_command.program),
            &[&cs_out, &rd, &wr],
        );
        cfg.set_in_pins(&[&mosi, &dc_in, &clk, &cs_in]);
        cfg.set_jmp_pin(&cs_in);
        cfg.set_out_pins(&[&db0, &db1, &db2, &db3, &db4, &db5, &db6, &db7, &dc_out]);

        cfg.shift_in = ShiftConfig {
            auto_fill: false,
            direction: ShiftDirection::Left,
            threshold: 32,
        };
        cfg.shift_out = ShiftConfig {
            auto_fill: false,
            direction: ShiftDirection::Left,
            threshold: 16,
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
    pub async fn work(&mut self) {
        let mut data = [0u16; 1];
        self.sm
            .rx()
            .dma_pull(self.dma.reborrow(), &mut data, false)
            .await;
    }
}
