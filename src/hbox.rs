/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

pub struct Hbox(*mut Ihandle);

impl Hbox {
    // TODO: must do something to kill this when the control is destroyed
    pub fn new() -> Dialog {
        unsafe {
            super::iup_open();
            let handle = IupHbox(ptr::null_mut());
            Dialog(handle)
        }
    }
}

