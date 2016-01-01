/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

extern crate clear_coat;

use std::ptr;
use clear_coat::*;
use clear_coat::common_attrs_cbs::*;

#[test]
fn test_shared_ownership() {
    let button = Button::new();
    {
        let dialog = Dialog::new(None);
        dialog.append(&button as &Control).unwrap();
    }
    button.handle();
    button.set_title("Hello");
}
