/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

#![feature(coerce_unsized)]

extern crate libc;
extern crate iup_sys;

macro_rules! impl_control_traits {
    ($control:ident) => {
        impl Drop for ::$control {
            fn drop(&mut self) {
                unsafe {
                    if IupGetParent(self.handle_mut()) == ptr::null_mut() {
                        IupDestroy(self.handle_mut());
                    }
                }
            }
        }

        unsafe impl Control for ::$control {
            fn handle(&self) -> *const Ihandle { self.0 }
            fn handle_mut(&mut self) -> *mut Ihandle { self.0 }
        }
    };
}

#[macro_use]
mod common_callbacks;
mod dialog;
mod button;

pub use dialog::{Position, Dialog};
pub use button::Button;
pub use common_callbacks::{NonMenuCommonCallbacks, MenuCommonCallbacks, ButtonCallback, Event};

use std::borrow::Cow;
use std::ffi::CStr;
use std::ptr;
use libc::{c_char, c_int};
use iup_sys::*;

pub fn main_loop() {
    unsafe {
        iup_open();
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

// Part of the contract of implementing this trait is that no invalid handle
// is returned. Either the handle will stay valid for the life of the object or
// the method will panic.
pub unsafe trait Control {
    fn handle(&self) -> *const Ihandle;
    fn handle_mut(&mut self) -> *mut Ihandle;
}

#[derive(Copy,Clone)]
pub enum MouseButton {
    Button1,
    Button2,
    Button3,
    Button4,
    Button5,
}

impl MouseButton {
    fn from_int(i: c_int) -> MouseButton {
        match i {
            IUP_BUTTON1 => MouseButton::Button1,
            IUP_BUTTON2 => MouseButton::Button2,
            IUP_BUTTON3 => MouseButton::Button3,
            IUP_BUTTON4 => MouseButton::Button4,
            IUP_BUTTON5 => MouseButton::Button5,
            _ => panic!("unknown mouse button"),
        }
    }

    // fn to_int(self) -> c_int {
    //     match self {
    //         MouseButton::Button1 => IUP_BUTTON1,
    //         MouseButton::Button2 => IUP_BUTTON2,
    //         MouseButton::Button3 => IUP_BUTTON3,
    //         MouseButton::Button4 => IUP_BUTTON4,
    //         MouseButton::Button5 => IUP_BUTTON5,
    //     }
    // }
}

#[derive(Clone)]
pub struct KeyboardMouseStatus {
    shift_pressed: bool,
    control_pressed: bool,
    alt_pressed: bool,
    sys_pressed: bool,
    button1_pressed: bool,
    button2_pressed: bool,
    button3_pressed: bool,
    button4_pressed: bool,
    button5_pressed: bool,
}

impl KeyboardMouseStatus {
    unsafe fn from_cstr(s: *const c_char) -> KeyboardMouseStatus {
        KeyboardMouseStatus {
            shift_pressed: iup_isshift(s),
            control_pressed: iup_iscontrol(s),
            alt_pressed: iup_isalt(s),
            sys_pressed: iup_issys(s),
            button1_pressed: iup_isbutton1(s),
            button2_pressed: iup_isbutton2(s),
            button3_pressed: iup_isbutton3(s),
            button4_pressed: iup_isbutton4(s),
            button5_pressed: iup_isbutton5(s),
        }
    }
}

pub trait CommonAttributes : Control {
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

pub trait TitleAttribute : Control {
    fn title(&self) -> String {
        get_str_attribute(self.handle(), "TITLE")
    }

    fn set_title(&mut self, title: &str) {
        set_str_attribute(self.handle_mut(), "TITLE", title);
    }
}


#[test]
#[should_panic]
fn test_destroyed_control() {
    let dialog = Dialog::new();
    let button = Button::new();
    dialog.append(button);
    button.set_title("Hello");
}

#[test]
#[should_panic]
fn test_destroyed_control_with_normalizer() {
    panic!("TODO");
}
