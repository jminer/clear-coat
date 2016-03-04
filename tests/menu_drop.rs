/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */
#![feature(const_fn)]

extern crate clear_coat;
extern crate iup_sys;

use std::sync::atomic::{self, AtomicUsize};
use clear_coat::*;
use clear_coat::common_attrs_cbs::*;
use iup_sys::*;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[test]
fn test_menu_drop() {
    let dialog = Dialog::new();
    let menu = Menu::new();
    menu.destroy_event().add(move || {
        COUNTER.fetch_add(1, atomic::Ordering::SeqCst);
    });
    dialog.set_menu(&menu);
    unsafe { assert_eq!(IupGetParent(menu.handle()), dialog.handle()); }
    drop(menu);
    // Test that the menu hasn't been destroyed.
    assert_eq!(COUNTER.load(atomic::Ordering::Acquire), 0);

    let menu2 = Menu::new();
    menu2.destroy_event().add(move || {
        COUNTER.fetch_add(2, atomic::Ordering::SeqCst);
    });
    dialog.set_menu(&menu2);
    // Test that adding another menu destroyed the first one.
    assert_eq!(COUNTER.load(atomic::Ordering::Acquire), 1);
    unsafe { assert_eq!(IupGetParent(menu2.handle()), dialog.handle()); }
    drop(menu2);
    assert_eq!(COUNTER.load(atomic::Ordering::Acquire), 1);

    drop(dialog);
    // Test that the menu is destroyed when the dialog is.
    assert_eq!(COUNTER.load(atomic::Ordering::Acquire), 3);
}
