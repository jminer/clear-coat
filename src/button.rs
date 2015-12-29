/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use std::ptr;
use iup_sys::*;
use super::{CommonAttributes,TitleAttribute,Control};

pub struct Button(*mut Ihandle);

impl Button {
    pub fn new() -> Button {
        unsafe {
            super::iup_open();
            let handle = IupButton(ptr::null_mut(), ptr::null_mut());
            Button(handle)
        }
    }
}

impl_control_traits!(Button);

impl CommonAttributes for Button {}

impl TitleAttribute for Button {}
