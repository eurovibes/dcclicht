// SPDX-License-Identifier: Apache-2.0
//
// SPDX-FileCopyrightText: 2025 Benedikt Spranger <b.spranger@linutronix.de>

use defmt::*;
use embassy_stm32::exti::ExtiInput;

#[embassy_executor::task]
pub async fn btn_task(mut button: ExtiInput<'static>) {
    info!("Press the USER button...");
    loop {
        button.wait_for_falling_edge().await;
        info!("Pressed!");
        button.wait_for_rising_edge().await;
        info!("Released!");
    }
}
