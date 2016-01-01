/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

#[macro_use]
extern crate clear_coat;

use std::rc::Rc;
use clear_coat::*;
use clear_coat::common_attrs_cbs::*;

// Tests that the correct children are removed and not destroyed when a container is.

#[test]
fn test_destroy_with_multiple_children() {
    let (x, y, z, w) = (Rc::new(1), Rc::new(5), Rc::new(10), Rc::new(15));
    let (x2, y2, z2, w2) = (x.clone(), y.clone(), z.clone(), w.clone());

    let button_x = Button::new();
    button_x.action().add_callback(move || println!("{}", *x2));
    let button_y = Button::new();
    button_y.action().add_callback(move || println!("{}", *y2));
    let button_z = Button::new();
    button_z.action().add_callback(move || println!("{}", *z2));
    let button_w = Button::new();
    button_w.action().add_callback(move || println!("{}", *w2));
    let dialog = Dialog::new(Some(&vbox!(&button_x, &button_y, &button_z, &button_w)));
    drop(button_y);
    let (x, y, z, w) = (Rc::try_unwrap(x).unwrap_err(), Rc::try_unwrap(y).unwrap_err(),
                        Rc::try_unwrap(z).unwrap_err(), Rc::try_unwrap(w).unwrap_err());
    drop(dialog);
    let (_x, _y, _z, _w) = (Rc::try_unwrap(x).unwrap_err(), Rc::try_unwrap(y).unwrap(),
                            Rc::try_unwrap(z).unwrap_err(), Rc::try_unwrap(w).unwrap_err());
    button_x.set_title("Hello");
    button_z.set_title("Hello");
    button_w.set_title("Hello");
}
