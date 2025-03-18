// SPDX-License-Identifier: Apache-2.0
//
// SPDX-FileCopyrightText: 2025 Benedikt Spranger <b.spranger@linutronix.de>

#![no_std]
#![no_main]

mod btn;
mod menu;
mod pwm;

use crate::btn::btn_task;
use crate::menu::{dec_dimmer, inc_dimmer, menu, menu_init, menu_update};
use crate::pwm::pwm_task;
use defmt::*;
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{OutputType, Pull};
use embassy_stm32::rcc::{AHBPrescaler, APBPrescaler, Hse, HseMode, Pll, PllPreDiv, PllSource, Sysclk};
use embassy_stm32::time::{Hertz, khz};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::usart::{BufferedInterruptHandler, BufferedUart, Config as UsartConfig};
use embedded_io_async::BufRead;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART1 => BufferedInterruptHandler<peripherals::USART1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll = Some(
            Pll {
                src: PllSource::HSE,
                prediv: PllPreDiv::DIV1,
                mul: embassy_stm32::rcc::PllMul::MUL9
            }
        );
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;
        config.rcc.sys = Sysclk::PLL1_P;
    }
    let p = embassy_stm32::init(config);
    let button = ExtiInput::new(p.PC13, p.EXTI13, Pull::Up);

    let ch1_1_pin = PwmPin::new_ch1(p.PA6, OutputType::PushPull);
    let ch2_1_pin = PwmPin::new_ch2(p.PA7, OutputType::PushPull);
    let ch3_1_pin = PwmPin::new_ch3(p.PB0, OutputType::PushPull);
    let ch4_1_pin = PwmPin::new_ch4(p.PB1, OutputType::PushPull);
    let ch1_2_pin = PwmPin::new_ch1(p.PA0, OutputType::PushPull);
    let ch2_2_pin = PwmPin::new_ch2(p.PA1, OutputType::PushPull);
    let ch3_2_pin = PwmPin::new_ch3(p.PA2, OutputType::PushPull);
    let ch4_2_pin = PwmPin::new_ch4(p.PA3, OutputType::PushPull);
    let pwm1 = SimplePwm::new(p.TIM3, Some(ch1_1_pin), Some(ch2_1_pin), Some(ch3_1_pin), Some(ch4_1_pin), khz(10), Default::default());
    let pwm2 = SimplePwm::new(p.TIM2, Some(ch1_2_pin), Some(ch2_2_pin), Some(ch3_2_pin), Some(ch4_2_pin), khz(10), Default::default());

    let mut tx_buf = [0u8; 32];
    let mut rx_buf = [0u8; 32];
    let buf_usart = BufferedUart::new(p.USART1, Irqs, p.PA10, p.PA9, &mut tx_buf, &mut rx_buf, UsartConfig::default()).unwrap();
    let (mut buf_tx, mut buf_rx) = buf_usart.split();

    let mut act = 0;

    unwrap!(_spawner.spawn(btn_task(button)));
    unwrap!(_spawner.spawn(pwm_task(pwm1, pwm2)));

    info!("DCC Licht - {}", env!("CARGO_PKG_VERSION"));
    menu_init(&mut buf_tx);

    menu(&mut buf_tx, act);
    loop {
        
        let buf = buf_rx.fill_buf().await.unwrap();
        info!("Received: {}", buf);
        let n = buf.len();
        for c in buf {
            match c {
                43 | 100 => {
                    if act > 0 {
                        inc_dimmer(act - 1);
                        menu_update(&mut buf_tx, act, act);
                    }
                },
                45 | 97 => {
                    if act > 0 {
                        dec_dimmer(act - 1);
                        menu_update(&mut buf_tx, act, act);
                    }
                },
                48..=56 => {
                    menu_update(&mut buf_tx, act, 0);
                    act = (c - 48) as usize;
                    menu_update(&mut buf_tx, act, act);
                },
                114 => menu(&mut buf_tx, act),
		115 => {
		    if act < 8 {
		        menu_update(&mut buf_tx, act, 0);
                        act = act + 1;
                        menu_update(&mut buf_tx, act, act);
		    }
		},
		119 => {
		    if act > 0 {
		        menu_update(&mut buf_tx, act, 0);
                        act = act - 1;
                        menu_update(&mut buf_tx, act, act);
		    }
                },
                _ => info!("Invalid command {}", c),
            }
        }
        buf_rx.consume(n);
    }
}
