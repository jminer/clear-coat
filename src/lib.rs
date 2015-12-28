/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

extern crate libc;
extern crate iup_sys;

mod common_callbacks;
mod dialog;
mod button;

pub use dialog::{Position, Dialog};
pub use button::Button;

use std::borrow::Cow;
use std::ffi::CStr;
use std::ptr;
use libc::{c_char, c_int};
use iup_sys::*;

pub fn main_loop() {
    unsafe {
        IupMainLoop();
    }
}


fn set_str_attribute(handle: *mut Ihandle, name: &str, value: &str) {
    unsafe {
        IupSetStrAttribute(handle,
                           name.as_ptr() as *const c_char,
                           value.as_ptr() as *const c_char);
    }
}

// Unfortunately, the return value has to be copied because its lifetime isn't guarenteed.
// IUP's docs state:
//     "The returned pointer can be used safely even if IupGetGlobal or IupGetAttribute are called
//     several times. But not too many times, because it is an internal buffer and after IUP may
//     reuse it after around 50 calls."
fn get_str_attribute(handle: *const Ihandle, name: &str) -> String {
    unsafe {
        get_str_attribute_slice(handle, name).into_owned()
    }
}

unsafe fn get_str_attribute_slice(handle: *const Ihandle, name: &str) -> Cow<str> {
    let value = IupGetAttribute(handle as *mut Ihandle, name.as_ptr() as *const c_char);
    CStr::from_ptr(value).to_string_lossy()
}

fn iup_open() {
    unsafe { IupOpen(ptr::null_mut(), ptr::null_mut()); }
}

pub trait Wrapper {
    fn handle(&self) -> *const Ihandle;
    fn handle_mut(&mut self) -> *mut Ihandle;
}

pub trait CommonAttributes : Wrapper {
    fn active(&self) -> bool {
        get_str_attribute(self.handle(), "ACTIVE") == "YES"
    }

    fn set_active(&mut self, active: bool) {
        set_str_attribute(self.handle_mut(), "ACTIVE", if active { "YES" } else { "NO" });
    }

    fn tip(&self) -> String {
        get_str_attribute(self.handle(), "TIP")
    }
    unsafe fn tip_slice(&self) -> Cow<str> {
        get_str_attribute_slice(self.handle(), "TIP")
    }

    fn set_tip(&mut self, tip: &str) {
        set_str_attribute(self.handle_mut(), "TIP", tip);
    }

    fn show(&mut self) -> Result<(), ()> {
        unsafe {
            if IupShow(self.handle_mut()) == IUP_NOERROR {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    fn hide(&mut self) -> Result<(), ()> {
        unsafe {
            if IupHide(self.handle_mut()) == IUP_NOERROR {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    fn set_visible(&mut self, visible: bool) -> Result<(), ()> {
        if visible { self.show() } else { self.hide() }
    }
}

pub trait TitleAttribute : Wrapper {
    fn title(&self) -> String {
        get_str_attribute(self.handle(), "TITLE")
    }

    fn set_title(&mut self, title: &str) {
        set_str_attribute(self.handle_mut(), "TITLE", title);
    }
}


