/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

extern crate clear_coat;

use std::ptr;
use clear_coat::*;

#[test]
fn test_shared_ownership() {
    let mut button = Button::new();
    {
        let mut dialog = Dialog::new(None);
        dialog.append(&button as &Control);
    }
    // if the next line panics, it is in handle(), not the assert!()
    assert!(button.handle() != ptr::null_mut());
    button.set_title("Hello");
}
