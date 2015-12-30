/* Copyright 2015 Jordan Miner
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 */

extern crate clear_coat;

use std::ptr;
use clear_coat::{Dialog, NonMenuCommonCallbacks};

fn main() {
    let mut dialog = Dialog(ptr::null_mut());
    dialog.leave_window().remove_callback();
}
