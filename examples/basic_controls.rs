/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

extern crate clear_coat;

use clear_coat::{Dialog, Position, TitleAttribute, CommonCallbacks};

fn main() {
    let mut dialog = Dialog::new(None);
    dialog.show_xy(Position::Center, Position::Center)
          .expect("There was a problem showing the window");
    dialog.set_title("Howdy");
    dialog.leave_window().add_callback(|| println!("left window 1"));
    dialog.leave_window().add_callback(|| println!("left window 2"));
    //dialog.leave_window().remove_callback(t);
    clear_coat::main_loop();
}
