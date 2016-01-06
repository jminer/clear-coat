/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

#[macro_use]
extern crate clear_coat;

use clear_coat::*;
use clear_coat::common_attrs_cbs::*;

fn main() {
    let button1 = Button::new();
    button1.set_title("Push Me");
    let button2 = Button::new();
    button2.set_title("Hi");
    button1.action_event().add(|| println!("you pushed it!"));
    let button3 = Button::new();
    button3.set_title("Hi");
    let button4 = Button::new();
    button4.set_title("Hi");

    // TODO: there is handle::is_null()
    // TODO: have setters return self, so that hbox and vbox, etc. can be configured without being stored in a variable?
    let dialog = Dialog::with_child(&vbox!(
        button1, fill!(),
        hbox!(fill!(), button3, button4),
        button2));

    dialog.show_xy(ScreenPosition::Center, ScreenPosition::Center)
          .expect("There was a problem showing the window");
    dialog.set_title("Howdy");
    let t = dialog.leave_window_event().add(|| println!("left window 1"));
    dialog.leave_window_event().add(|| println!("left window 2"));
    dialog.leave_window_event().remove(t);
    clear_coat::main_loop();
}
