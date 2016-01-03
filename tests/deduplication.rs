/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

extern crate clear_coat;

use std::rc::Rc;
use clear_coat::*;
use clear_coat::common_attrs_cbs::*;

// Tests that creating multiple control wrappers from the same *mut Ihandle will share
// a reference count.

#[test]
fn test_deduplication() {
    let x = Rc::new(5);
    let x2 = x.clone();
    let dialog = Dialog::new();
    dialog.enter_window().add_callback(move || println!("{}", *x2));
    let handle = dialog.handle();
    let dialog2 = unsafe { Dialog::from_handle(handle) };
    let dialog3 = unsafe { Dialog::from_handle(handle) };
    let x = Rc::try_unwrap(x).unwrap_err();
    drop(dialog);
    let x = Rc::try_unwrap(x).unwrap_err();
    drop(dialog3);
    let x = Rc::try_unwrap(x).unwrap_err();
    drop(dialog2);
    let _ = Rc::try_unwrap(x).unwrap();
}
