/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

#[macro_use]
extern crate clear_coat;

use clear_coat::*;

// Tests that dropping/destroying a control inside a callback works.

#[test]
fn test_drop_control_in_callback() {
    let dialog = Dialog::new();
    dialog.show_event().add(|_| {
        let _ = Button::new();
        exit_loop();
    });
    dialog.show_xy(ScreenPosition::Center, ScreenPosition::Center).expect("could not show dialog");
    main_loop();
}
