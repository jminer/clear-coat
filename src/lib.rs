/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

#![feature(coerce_unsized)]

extern crate libc;
extern crate iup_sys;
extern crate kernel32;

#[cfg(windows)]
fn get_thread_id() -> isize {
    use kernel32::GetCurrentThreadId;
    unsafe { GetCurrentThreadId() as isize }
}

macro_rules! impl_control_traits {
    ($control:ident) => {
        unsafe impl Control for ::$control {
            fn handle(&self) -> *mut Ihandle {
                assert!(self.0.get() != ptr::null_mut(), "attempted to use destroyed control");
                ::check_thread();
                self.0.get()
            }
        }

        unsafe impl UnwrapHandle for ::$control {
            fn try_unwrap_handle(self) -> Result<*mut Ihandle, Self> {
                assert!(self.0.get() != ptr::null_mut(), "attempted to use destroyed control");
                self.0.try_unwrap().map_err(|handle_rc| ::$control(handle_rc))
            }
        }
    };
}

#[macro_use]
mod common_callbacks;
mod dialog;
mod button;
mod handle_rc;

pub use dialog::{Position, Dialog};
pub use button::Button;
pub use common_callbacks::{NonMenuCommonCallbacks, MenuCommonCallbacks, ButtonCallback, Event};

use std::borrow::Cow;
use std::ffi::CStr;
use std::ptr;
use std::sync::atomic::{AtomicIsize, Ordering, ATOMIC_ISIZE_INIT};
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
    check_thread();
    unsafe { IupOpen(ptr::null_mut(), ptr::null_mut()); }
}

static THREAD_ID: AtomicIsize = ATOMIC_ISIZE_INIT;

fn check_thread() {
    let thread_id = get_thread_id();
    let prev = THREAD_ID.compare_and_swap(0, thread_id, Ordering::SeqCst);
    assert!(prev == 0 || prev == thread_id, "IUP/Clear Coat functions must be called from a single thread");
}

// Part of the contract of implementing this trait is that no invalid handle
// is returned. Either the handle will stay valid for the life of the object or
// the method will panic.
pub unsafe trait Control {
    fn handle(&self) -> *mut Ihandle;

    fn detach(&self) {
        unsafe { IupDetach(self.handle()); }
    }

    fn reparent(&self, new_parent: &Container, ref_child: Option<&Control>) -> Result<(), ()> {
        unsafe {
            let ref_child = ref_child.map(|c| c.handle()).unwrap_or(ptr::null_mut());
            if IupReparent(self.handle(), new_parent.handle(), ref_child) == IUP_NOERROR {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    fn get_dialog(&self) -> Option<Dialog> {
        unsafe {
            let handle = IupGetDialog(self.handle());
            if handle == ptr::null_mut() {
                None
            } else {
                Some(Dialog::from_handle(handle))
            }
        }
    }
}

// If this wrapper has the only reference, it gives up shared ownership of the *mut Ihandle.
pub unsafe trait UnwrapHandle : Sized {
    fn try_unwrap_handle(self) -> Result<*mut Ihandle, Self>;
}

pub trait Container : Control {
    fn append(&self, new_child: &Control) -> Result<(), ()> {
        unsafe {
            if IupAppend(self.handle(), new_child.handle()) == ptr::null_mut() {
                Err(())
            } else {
                Ok(())
            }
        }
    }

    fn insert(&self, ref_child: Option<&Control>, new_child: &Control) -> Result<(), ()> {
        unsafe {
            let ref_child = ref_child.map(|c| c.handle()).unwrap_or(ptr::null_mut());
            if IupInsert(self.handle(), ref_child, new_child.handle()) == ptr::null_mut() {
                Err(())
            } else {
                Ok(())
            }
        }
    }
}

pub trait NonDialogContainer : Container {
    fn refresh_children(&self) {
        unsafe {
            IupRefreshChildren(self.handle());
        }
    }
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
        set_str_attribute(self.handle(), "ACTIVE", if active { "YES" } else { "NO" });
    }

    fn tip(&self) -> String {
        get_str_attribute(self.handle(), "TIP")
    }
    unsafe fn tip_slice(&self) -> Cow<str> {
        get_str_attribute_slice(self.handle(), "TIP")
    }

    fn set_tip(&mut self, tip: &str) {
        set_str_attribute(self.handle(), "TIP", tip);
    }

    fn show(&mut self) -> Result<(), ()> {
        unsafe {
            if IupShow(self.handle()) == IUP_NOERROR {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    fn hide(&mut self) -> Result<(), ()> {
        unsafe {
            if IupHide(self.handle()) == IUP_NOERROR {
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
        set_str_attribute(self.handle(), "TITLE", title);
    }
}
