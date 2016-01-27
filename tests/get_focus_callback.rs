/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

#[macro_use]
extern crate clear_coat;

use std::sync::atomic::{AtomicIsize, ATOMIC_ISIZE_INIT, Ordering};
use clear_coat::*;
use clear_coat::common_attrs_cbs::*;

// Tests that the get_focus callback works (which is most likely a test that the `simple_callback` function works).

static COUNTER: AtomicIsize = ATOMIC_ISIZE_INIT;

#[test]
fn test_get_focus_callback() {
    let button = Button::new();
    button.get_focus_event().add(move || { COUNTER.store(2, Ordering::Release); exit_loop(); });
    let dialog = Dialog::with_child(&vbox!(&button));
    //dialog.show_event().add(|_| CallbackAction::Close);

    dialog.show_xy(ScreenPosition::Center, ScreenPosition::Center).expect("could not show dialog");
    main_loop();
    assert_eq!(COUNTER.load(Ordering::Acquire), 2);
}
