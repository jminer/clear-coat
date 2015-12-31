/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use std::ptr;
use handle_rc::HandleRc;
use super::{Control, Container, UnwrapHandle};
use iup_sys::*;

// Be sure that the Vec is not dropped before the *mut *mut Ihandle is used.
fn wrapper_to_handles(controls: Option<&[&::Control]>)
-> (Option<Vec<*mut Ihandle>>, *mut *mut Ihandle)
{
    let mut controls: Option<Vec<_>> = controls.map(|slice| slice.iter().map(|c| c.handle()).collect());
    let p = controls.as_mut().map(|v| v.as_mut_ptr()).unwrap_or(ptr::null_mut());
    (controls, p)
}


pub struct Hbox(HandleRc);

impl Hbox {
    pub fn new(children: Option<&[&::Control]>) -> Hbox {
        unsafe {
            ::iup_open();
            let (children, children_handles) = wrapper_to_handles(children);
            Hbox::from_handles(children_handles)
        }
    }

    pub unsafe fn from_handles(children: *mut *mut Ihandle) -> Hbox {
        let handle = IupHboxv(children);
        Hbox(HandleRc::new(handle))
    }
}

impl_control_traits!(Hbox);

impl Container for Hbox {}

#[macro_export]
macro_rules! hbox {
    ($($c:expr),*) => {
        unsafe {
            let mut handles = Vec::new();
            $(
                handles.push($c.handle());
            )*
            Hbox::from_handles(handles.as_mut_ptr())
        }
    };
    ($($c:expr,)*) => { hbox!($($c),*) };
}


pub struct Vbox(HandleRc);

impl Vbox {
    pub fn new(children: Option<&[&::Control]>) -> Vbox {
        unsafe {
            ::iup_open();
            let (children, children_handles) = wrapper_to_handles(children);
            Vbox::from_handles(children_handles)
        }
    }

    pub unsafe fn from_handles(children: *mut *mut Ihandle) -> Vbox {
        let handle = IupVboxv(children);
        Vbox(HandleRc::new(handle))
    }
}

impl_control_traits!(Vbox);

impl Container for Vbox {}

#[macro_export]
macro_rules! vbox {
    ($($c:expr),*) => {
        unsafe {
            let mut handles = Vec::new();
            $(
                handles.push($c.handle());
            )*
            Vbox::from_handles(handles.as_mut_ptr())
        }
    };
    ($($c:expr,)*) => { vbox!($($c),*) };
}
