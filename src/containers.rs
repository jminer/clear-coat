/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use std::ptr;
use iup_sys::*;
use super::{Control, UnwrapHandle};
use super::handle_rc::HandleRc;

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


// Be sure that the Vec is not dropped before the *mut *mut Ihandle is used.
fn wrapper_to_handles(controls: Option<&[&::Control]>)
-> (Option<Vec<*mut Ihandle>>, *mut *mut Ihandle)
{
    let mut controls: Option<Vec<_>> = controls.map(|slice| {
        let mut v: Vec<*mut Ihandle> = slice.iter().map(|c| c.handle()).collect();
        v.push(ptr::null_mut()); // array has to be null terminated
        v
    });
    let p = controls.as_mut().map(|v| v.as_mut_ptr()).unwrap_or(ptr::null_mut());
    (controls, p)
}


#[derive(Clone)]
pub struct Fill(HandleRc);

impl Fill {
    pub fn new() -> Fill {
        unsafe {
            ::iup_open();
            let handle = IupFill();
            Fill(HandleRc::new(handle))
        }
    }
}

impl_control_traits!(Fill);

#[macro_export]
macro_rules! fill { // This is a macro for consistency, even though it could just be a function.
    () => { Fill::new() };
}


#[derive(Clone)]
pub struct Hbox(HandleRc);

impl Hbox {
    pub fn new(children: Option<&[&::Control]>) -> Hbox {
        unsafe {
            ::iup_open();
            let (_children, children_handles) = wrapper_to_handles(children);
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
            use std::ptr;
            let mut handles = Vec::new();
            $(
                // The control has to be stored in a binding to ensure it isn't dropped before
                // it is added as a child of the container. (Otherwise, the control is destroyed.)
                let c = $c;
                handles.push(c.handle());
            )*
            handles.push(ptr::null_mut());
            Hbox::from_handles(handles.as_mut_ptr())
        }
    };
    ($($c:expr,)*) => { hbox!($($c),*) };
}


#[derive(Clone)]
pub struct Vbox(HandleRc);

impl Vbox {
    pub fn new(children: Option<&[&::Control]>) -> Vbox {
        unsafe {
            ::iup_open();
            let (_children, children_handles) = wrapper_to_handles(children);
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
            use std::ptr;
            let mut handles = Vec::new();
            $(
                // The control has to be stored in a binding to ensure it isn't dropped before
                // it is added as a child of the container. (Otherwise, the control is destroyed.)
                let c = $c;
                handles.push(c.handle());
            )*
            handles.push(ptr::null_mut());
            Vbox::from_handles(handles.as_mut_ptr())
        }
    };
    ($($c:expr,)*) => { vbox!($($c),*) };
}
