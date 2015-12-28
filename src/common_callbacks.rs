/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use std::cell::RefCell;
use std::collections::HashMap;
use libc::{c_int};
use iup_sys::*;

enum CallbackAction {
    Default,
    Close,
    Ignore,
    Continue,
}

type CallbackRegistry<T> = RefCell<HashMap<*mut Ihandle, Vec<(usize, Box<T>)>>>;

// If a callback's documentation does not specify valid return values, then only IUP_DEFAULT is
// supported.

// use LDESTROY_CB instead of DESTROY_CB



thread_local!(
    static LEAVE_WINDOW_CALLBACKS: CallbackRegistry<FnMut()> =
        RefCell::new(HashMap::new())
);

extern fn leave_window(ih: *mut Ihandle) -> c_int {
    use std::ops::DerefMut;
    LEAVE_WINDOW_CALLBACKS.with(|cell| {
        if let Some(cbs) = cell.borrow_mut().get_mut(&ih) {
            for cb in cbs {
                cb.1();
            }
        }
    });
    IUP_DEFAULT
}

pub struct LeaveWindowCallbackToken {
    id: usize,
    ih: *mut Ihandle,
}

fn add_leave_window_callback_inner(ih: *mut Ihandle, cb: Box<FnMut() + 'static>)
-> LeaveWindowCallbackToken {
    LEAVE_WINDOW_CALLBACKS.with(|cell| {
        let mut map = cell.borrow_mut();
        let cbs = map.entry(ih).or_insert_with(|| Vec::with_capacity(4));
        let id = cbs.last().map(|&(id, _)| id + 1).unwrap_or(0);
        cbs.push((id, cb));

        unsafe {
            IupSetCallback(ih, "LEAVEWINDOW_CB".as_ptr() as *const i8, leave_window);
        }

        LeaveWindowCallbackToken { id: id, ih: ih }
    })
}

pub fn add_leave_window_callback<F: FnMut() + 'static>(ih: *mut Ihandle, cb: F)
-> LeaveWindowCallbackToken {
    add_leave_window_callback_inner(ih, Box::new(cb))
}

pub fn remove_leave_window_callback(token: LeaveWindowCallbackToken) {
    LEAVE_WINDOW_CALLBACKS.with(|cell| {
        let mut map = cell.borrow_mut();
        if let Some(cbs) = map.get_mut(&token.ih) {
            if let Some(index) = cbs.iter().position(|&(id, _)| id == token.id) {
                cbs.remove(index);
            }
        }
    })
}
