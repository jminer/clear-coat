/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use std::borrow::Cow;
use std::ffi::CStr;
use libc::{c_char, c_int};
use iup_sys::*;
use super::Control;

pub fn set_str_attribute(handle: *mut Ihandle, name: &str, value: &str) {
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
pub fn get_str_attribute(handle: *mut Ihandle, name: &str) -> String {
    unsafe {
        get_str_attribute_slice(handle, name).into_owned()
    }
}

// This function isn't very error prone (see above), but isn't completely safe either.
pub unsafe fn get_str_attribute_slice(handle: *mut Ihandle, name: &str) -> Cow<str> {
    let value = IupGetAttribute(handle as *mut Ihandle, name.as_ptr() as *const c_char);
    CStr::from_ptr(value).to_string_lossy()
}

pub fn get_int_int_attribute(handle: *mut Ihandle, name: &str) -> (i32, i32) {
    unsafe {
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        assert!(IupGetIntInt(handle as *mut Ihandle,
                            name.as_ptr() as *const c_char,
                            &mut x as *mut c_int,
                            &mut y as *mut c_int) == 2);
        (x, y)
    }
}


pub trait CommonAttributes : Control {
    fn active(&self) -> bool {
        get_str_attribute(self.handle(), "ACTIVE") == "YES"
    }

    fn set_active(&self, active: bool) {
        set_str_attribute(self.handle(), "ACTIVE", if active { "YES" } else { "NO" });
    }

    fn tip(&self) -> String {
        get_str_attribute(self.handle(), "TIP")
    }
    unsafe fn tip_slice(&self) -> Cow<str> {
        get_str_attribute_slice(self.handle(), "TIP")
    }

    fn set_tip(&self, tip: &str) {
        set_str_attribute(self.handle(), "TIP", tip);
    }

    fn min_size(&self) -> (i32, i32) {
        get_int_int_attribute(self.handle(), "MINSIZE")
    }

    fn set_min_size(&self, x: i32, y: i32) {
        let s = format!("{}x{}", x, y);
        set_str_attribute(self.handle(), "MINSIZE", &s);
    }

    fn max_size(&self) -> (i32, i32) {
        get_int_int_attribute(self.handle(), "MAXSIZE")
    }

    fn set_max_size(&self, x: i32, y: i32) {
        let s = format!("{}x{}", x, y);
        set_str_attribute(self.handle(), "MAXSIZE", &s);
    }

    fn show(&self) -> Result<(), ()> {
        unsafe {
            if IupShow(self.handle()) == IUP_NOERROR {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    fn hide(&self) -> Result<(), ()> {
        unsafe {
            if IupHide(self.handle()) == IUP_NOERROR {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    fn set_visible(&self, visible: bool) -> Result<(), ()> {
        if visible { self.show() } else { self.hide() }
    }
}

pub trait TitleAttribute : Control {
    fn title(&self) -> String {
        get_str_attribute(self.handle(), "TITLE")
    }

    fn set_title(&self, title: &str) {
        set_str_attribute(self.handle(), "TITLE", title);
    }
}

pub trait OrientationAttribute : Control {
    fn orientation(&self) -> ::Orientation {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "ORIENTATION");
            ::Orientation::from_str(s.as_bytes())
        }
    }

    fn set_orientation(&self, orientation: ::Orientation) {
        set_str_attribute(self.handle(), "ORIENTATION", orientation.to_str());
    }
}
