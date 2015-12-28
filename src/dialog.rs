/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use std::ffi::CStr;
use std::ptr;
use libc::{c_int};
use iup_sys::*;
use super::{CommonAttributes,TitleAttribute,Wrapper};

#[derive(Copy,Clone,PartialEq)]
pub enum Position {
    Left,
    Center,
    Right,
    MousePos,
    CenterParent,
    Current,
}

impl Position {
    //fn from_int(i: c_int) -> Position {
    //    match i {
    //        IUP_LEFT => Position::Left,
    //        IUP_CENTER => Position::Center,
    //        IUP_RIGHT => Position::Right,
    //        IUP_MOUSEPOS => Position::MousePos,
    //        IUP_CENTERPARENT => Position::CenterParent,
    //        IUP_CURRENT => Position::Current,
    //        _ => panic!("unknown position"),
    //    }
    //}

    fn to_int(self) -> c_int {
        match self {
            Position::Left => IUP_LEFT,
            Position::Center => IUP_CENTER,
            Position::Right => IUP_RIGHT,
            Position::MousePos => IUP_MOUSEPOS,
            Position::CenterParent => IUP_CENTERPARENT,
            Position::Current => IUP_CURRENT,
        }
    }
}

pub struct Dialog(*mut Ihandle);

impl Dialog {
    // TODO: must do something to kill this when the control is destroyed
    pub fn new() -> Dialog {
        unsafe {
            super::iup_open();
            let handle = IupDialog(ptr::null_mut());
            Dialog(handle)
        }
    }

    /*pub*/ unsafe fn from_handle(handle: *mut Ihandle) -> Dialog {
        // got to already be IupOpen()ed
        assert!(CStr::from_ptr(IupGetClassName(handle)).to_string_lossy() == "dialog");
        Dialog(handle)
    }

    pub fn show_xy(&mut self, x: Position, y: Position) -> Result<(), ()> {
        unsafe {
            if IupShowXY(self.0, x.to_int(), y.to_int()) == IUP_NOERROR {
                Ok(())
            } else {
                Err(())
            }
        }
    }
}

impl Wrapper for Dialog {
    fn handle(&self) -> *const Ihandle { self.0 }
    fn handle_mut(&mut self) -> *mut Ihandle { self.0 }
}

impl CommonAttributes for Dialog {}

impl TitleAttribute for Dialog {}
