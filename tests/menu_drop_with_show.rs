/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */
#![feature(const_fn)]

extern crate clear_coat;
extern crate iup_sys;

use std::ptr;
use std::sync::atomic::{self, AtomicUsize};
use clear_coat::*;
use clear_coat::common_attrs_cbs::*;
use iup_sys::*;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[test]
fn test_menu_drop_with_show() {
    let dialog = Dialog::with_child(&Fill::new());

    let menu = Menu::new();
    menu.destroy_event().add(move || {
        COUNTER.fetch_add(1, atomic::Ordering::SeqCst);
    });
    dialog.set_menu(Some(&menu));

    dialog.show_event().add(|_| CallbackAction::Close );
    dialog.show_xy(ScreenPosition::Center, ScreenPosition::Center).expect("could not show dialog");
    main_loop();

    let menu2 = Menu::new();
    menu2.destroy_event().add(move || {
        COUNTER.fetch_add(2, atomic::Ordering::SeqCst);
    });
    dialog.set_menu(Some(&menu2));
    // Test that the first menu is no longer a child.
    unsafe { assert_eq!(IupGetParent(menu.handle()), ptr::null_mut()); }
    unsafe { assert_eq!(IupGetChildPos(dialog.handle(), menu.handle()), -1); }
    drop(menu2);
    assert_eq!(COUNTER.load(atomic::Ordering::Acquire), 0);
    drop(dialog);
    assert_eq!(COUNTER.load(atomic::Ordering::Acquire), 2);
    drop(menu);
    assert_eq!(COUNTER.load(atomic::Ordering::Acquire), 3);
}
