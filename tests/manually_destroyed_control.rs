/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

extern crate clear_coat;
extern crate iup_sys;

use std::ptr;
use clear_coat::*;
use clear_coat::common_attrs_cbs::*;
use iup_sys::*;

#[test]
#[should_panic(expected="attempted to use destroyed control")]
fn test_manually_destroyed_control() {
    let button = Button::new();
    assert!(button.handle() != ptr::null_mut());
    unsafe { IupDestroy(button.handle()); }
    button.set_title("Hello"); // should panic since control is destroyed (pointer should be zeroed)
}
