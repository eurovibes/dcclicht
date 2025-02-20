// SPDX-License-Identifier: Apache-2.0
//
// SPDX-FileCopyrightText: 2025 Benedikt Spranger <b.spranger@linutronix.de>

use crate::menu::{get_dimmer, wait_dimmer};
use defmt::*;

use embassy_stm32::peripherals::{TIM2, TIM3};
use embassy_stm32::timer::{simple_pwm::SimplePwm, Channel};

static PWMSTEPS: [u16; 16] = [0, 2, 3, 4, 6, 8, 11, 16, 23, 32, 45, 64, 90, 128, 181, 255];

#[embassy_executor::task]
pub async fn pwm_task(mut pwm1: SimplePwm<'static, TIM3>, mut pwm2: SimplePwm<'static, TIM2>) {
    info!("PWM task started.");

    for ch in [Channel::Ch1, Channel::Ch2, Channel::Ch3, Channel::Ch4] {
        pwm1.channel(ch).enable();
	    pwm2.channel(ch).enable();
    }

    info!("PWMs initialized");
    info!("PWMs max duty {}", pwm2.ch1().max_duty_cycle());

    loop {
        info!("PWM duty change");
        pwm2.ch1().set_duty_cycle_fraction(PWMSTEPS[get_dimmer(7) as usize], 256);
        pwm2.ch2().set_duty_cycle_fraction(PWMSTEPS[get_dimmer(6) as usize], 256);
        pwm2.ch3().set_duty_cycle_fraction(PWMSTEPS[get_dimmer(5) as usize], 256);
        pwm2.ch4().set_duty_cycle_fraction(PWMSTEPS[get_dimmer(4) as usize], 256);
        pwm1.ch1().set_duty_cycle_fraction(PWMSTEPS[get_dimmer(3) as usize], 256);
        pwm1.ch2().set_duty_cycle_fraction(PWMSTEPS[get_dimmer(2) as usize], 256);
        pwm1.ch3().set_duty_cycle_fraction(PWMSTEPS[get_dimmer(1) as usize], 256);
        pwm1.ch4().set_duty_cycle_fraction(PWMSTEPS[get_dimmer(0) as usize], 256);

        wait_dimmer().await;
    }
}
