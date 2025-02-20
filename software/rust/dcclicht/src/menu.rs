// SPDX-License-Identifier: Apache-2.0
//
// SPDX-FileCopyrightText: 2025 Benedikt Spranger <b.spranger@linutronix.de>

use core::cell::RefCell;
use core::fmt::Write;
use cortex_m::interrupt;
use cortex_m::prelude::_embedded_hal_blocking_serial_Write;
use defmt::*;
use embassy_stm32::usart::BufferedUartTx;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;
use heapless::String;
use {defmt_rtt as _, panic_probe as _};

const BEL: u8 = 7;
const DINGDINGDING: [u8; 3] = [BEL, BEL, BEL];

static DIMMER: interrupt::Mutex<RefCell<[u8; 8]>> = interrupt::Mutex::new(RefCell::new([0; 8]));
static UPDATE: Signal<ThreadModeRawMutex, u32> = Signal::new();

pub fn get_dimmer(idx: usize) -> u8 {
    if idx > 7 {
        warn!("get_dimmer: idx {} out of range - Return 0", idx);
        return 0;
    }
    let mut a: u8 = 0;
    interrupt::free(|cs| {a = DIMMER.borrow(cs).borrow_mut() [idx];});
    return a;
}

pub fn set_dimmer(idx: usize, val: u8) {
    if idx > 7 {
        warn!("set_dimmer: idx {} out of range - Ignoring", idx);
        return;
    }
    interrupt::free(|cs| {DIMMER.borrow(cs).borrow_mut() [idx] = val});
    UPDATE.signal(0);
}

pub fn dec_dimmer(idx: usize) {
    if idx > 7 {
        warn!("dec_dimmer: idx {} out of range - Ignoring", idx);
        return;
    }
    interrupt::free(|cs| {
        let a = DIMMER.borrow(cs).borrow_mut() [idx];
        if a >0 {
            DIMMER.borrow(cs).borrow_mut() [idx] = a - 1;
            UPDATE.signal(0);
        }
    });
}

pub fn inc_dimmer(idx: usize) {
    if idx > 7 {
        warn!("inc_dimmer: idx {} out of range - Ignoring", idx);
        return;
    }
    interrupt::free(|cs| {
        let a = DIMMER.borrow(cs).borrow_mut() [idx];
        if a < 15 {
            DIMMER.borrow(cs).borrow_mut() [idx] = a + 1;
            UPDATE.signal(0);
        }
    });
}

pub async fn wait_dimmer() {
    UPDATE.wait().await;
}

pub fn menu_init(buf_tx: &mut BufferedUartTx) {
    buf_tx.bwrite_all(&[27, 99]).unwrap();
    buf_tx.bwrite_all(&DINGDINGDING).unwrap();

    for n in 1..=8 {
        set_dimmer(n, 0);
    }
}

pub fn menu_update(buf_tx: &mut BufferedUartTx, idx: usize, sel: usize) {
    let mut msg: String<127> = String::new();

    if idx < 1 || idx > 8 {return;}

    if idx == sel {
        core::writeln!(&mut msg, "\x1b[{};10HLicht {}: \x1b[7m{:3}\x1b[0m",
                       idx + 4, idx, get_dimmer(idx - 1)).unwrap();
    } else {
        core::writeln!(&mut msg, "\x1b[{};10HLicht {}: {:3}",
                       idx + 4, idx, get_dimmer(idx - 1)).unwrap();
    }
    buf_tx.bwrite_all(msg.as_bytes()).unwrap();
    msg.clear();
    buf_tx.bwrite_all(b"\x1b[H").unwrap();   
}

pub fn menu(buf_tx: &mut BufferedUartTx, sel: usize) {
    let mut msg: String<127> = String::new();
    let version = env!("CARGO_PKG_VERSION");

    buf_tx.bwrite_all(b"\x1b[2J\x1b[1;20H").unwrap();
    core::writeln!(&mut msg, "DCC Licht - {}", version).unwrap();
    buf_tx.bwrite_all(msg.as_bytes()).unwrap();
    msg.clear();

    for n in 1..=8 {
        menu_update(buf_tx, n, sel);
    }

    buf_tx.bwrite_all(b"\x1b[H").unwrap();   
}
