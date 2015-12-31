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
    button1.action().add_callback(|| println!("you pushed it!"));

    let mut dialog = Dialog::new(Some(&vbox!(button1, button2)));

    dialog.show_xy(Position::Center, Position::Center)
          .expect("There was a problem showing the window");
    dialog.set_title("Howdy");
    let t = dialog.leave_window().add_callback(|| println!("left window 1"));
    dialog.leave_window().add_callback(|| println!("left window 2"));
    dialog.leave_window().remove_callback(t);
    clear_coat::main_loop();
}
