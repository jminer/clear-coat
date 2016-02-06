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
use smallvec::SmallVec;
use super::Control;

pub fn str_to_c_vec<'a: 'b, 'b, A: ::smallvec::Array<Item=u8>>(s: &'a str, buf: &'b mut SmallVec<A>) -> *const c_char {
    // `CString` in the std library doesn't check if the &str already ends in a null terminator
    // It allocates and pushes a 0 unconditionally. However, I can add the null to string literals
    // and avoid many allocations.
    if s.as_bytes().last() == Some(&0) && !s.as_bytes()[..s.len() - 1].contains(&b'\0') {
        s.as_bytes().as_ptr() as *const c_char
    } else {
        buf.grow(s.len() + 1);
        buf.extend(s.as_bytes().iter().map(|c| if *c == b'\0' { b'?' } else { *c }));
        buf.push(0);
        (&buf[..]).as_ptr() as *const c_char
    }
}

pub fn set_str_attribute(handle: *mut Ihandle, name: &str, value: &str) {
    unsafe {
        let mut name_buf = SmallVec::<[u8; 32]>::new();
        let c_name = str_to_c_vec(name, &mut name_buf);
        let mut value_buf = SmallVec::<[u8; 32]>::new(); // TODO: change to 64 after upgrading smallvec
        let c_value = str_to_c_vec(value, &mut value_buf);
        //println!("setting {:?} to {:?}", CStr::from_ptr(c_name).to_string_lossy(), CStr::from_ptr(c_value).to_string_lossy());
        IupSetStrAttribute(handle, c_name, c_value);
    }
}

pub fn get_attribute_ptr(handle: *mut Ihandle, name: &str) -> *mut c_char {
    unsafe {
        let mut name_buf = SmallVec::<[u8; 32]>::new();
        let c_name = str_to_c_vec(name, &mut name_buf);
        IupGetAttribute(handle as *mut Ihandle, c_name)
    }
}

// Unfortunately, the return value has to be copied because its lifetime isn't guaranteed.
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
    let value = get_attribute_ptr(handle, name);
    //println!("getting {:?}: {:?}", name, CStr::from_ptr(value).to_string_lossy());
    CStr::from_ptr(value).to_string_lossy()
}

pub fn get_int_int_attribute(handle: *mut Ihandle, name: &str) -> (i32, i32) {
    unsafe {
        let mut name_buf = SmallVec::<[u8; 32]>::new();
        let c_name = str_to_c_vec(name, &mut name_buf);
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        assert!(IupGetIntInt(handle as *mut Ihandle,
                             c_name,
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
