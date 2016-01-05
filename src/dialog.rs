/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use std::ffi::CStr;
use std::ptr;
use iup_sys::*;
use super::{
    Control,
    UnwrapHandle,
    ScreenPosition,
    Popup,
};
use super::attributes::{
    CommonAttributes,
    TitleAttribute,
};
use super::callbacks::{
    MenuCommonCallbacks,
    NonMenuCommonCallbacks,
};
use super::containers::Container;
use super::handle_rc::HandleRc;

#[derive(Clone)]
pub struct Dialog(HandleRc);

impl Dialog {
    pub fn new() -> Dialog {
        unsafe {
            ::iup_open();
            let ih = IupDialog(ptr::null_mut());
            let d = Dialog(HandleRc::new(ih));
            d.set_min_size(150, 0);
            d
        }
    }

    pub fn with_child(child: &Control) -> Dialog {
        unsafe {
            ::iup_open();
            let ih = IupDialog(child.handle());
            let d = Dialog(HandleRc::new(ih));
            d.set_min_size(150, 0);
            d
        }
    }

    pub unsafe fn from_handle(handle: *mut Ihandle) -> Dialog {
        // got to already be IupOpen()ed
        assert!(CStr::from_ptr(IupGetClassName(handle)).to_string_lossy() == "dialog");
        Dialog(HandleRc::new(handle))
    }

    pub fn show_xy(&self, x: ScreenPosition, y: ScreenPosition) -> Result<(), ()> {
        unsafe {
            if IupShowXY(self.handle(), x.to_int(), y.to_int()) == IUP_NOERROR {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    pub fn refresh(&self) {
        unsafe {
            IupRefresh(self.handle());
        }
    }
}

impl_control_traits!(Dialog);

impl Container for Dialog {}
impl Popup for Dialog {}

impl CommonAttributes for Dialog {}
impl TitleAttribute for Dialog {}

impl MenuCommonCallbacks for Dialog {}
impl NonMenuCommonCallbacks for Dialog {}
