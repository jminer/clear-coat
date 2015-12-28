/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

extern crate clear_coat;

use clear_coat::{Dialog, Position, TitleAttribute};

fn main() {
    let mut dialog = Dialog::new();
    dialog.show_xy(Position::Center, Position::Center)
          .expect("There was a problem showing the window");
    dialog.set_title("Howdy");
    clear_coat::main_loop();
}
