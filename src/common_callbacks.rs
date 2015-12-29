/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::{DerefMut,CoerceUnsized};
use std::thread::LocalKey;
use libc::{c_int, c_void};
use iup_sys::*;
use super::Control;

enum CallbackAction {
    Default,
    Close,
    Ignore,
    Continue,
}

// If a callback's documentation does not specify valid return values, then only IUP_DEFAULT is
// supported.

// use LDESTROY_CB instead of DESTROY_CB

struct Token {
    id: usize,
    ih: *mut Ihandle,
}

struct CallbackRegistry<F: ?Sized, T: Into<Token> + From<Token>> {
    cb_name: &'static str,
    cb_fn: Icallback,
    callbacks: RefCell<HashMap<*mut Ihandle, Vec<(usize, Box<F>)>>>,
    phantom: PhantomData<*const T>,
}

impl<F: ?Sized, T: Into<Token> + From<Token>> CallbackRegistry<F, T> {
    // Icallback is the most common type of callback, but there are many exceptions. If the
    // callback's signature does not match Icallback, just cast to Icallback.
    pub fn new(cb_name: &'static str, cb_fn: Icallback) -> CallbackRegistry<F, T> {
        CallbackRegistry {
            cb_name: cb_name,
            cb_fn: cb_fn,
            callbacks: RefCell::new(HashMap::new()),
            phantom: PhantomData,
        }
    }

    fn add_callback_inner(&self, ih: *mut Ihandle, cb: Box<F>) -> T {
        let mut map = self.callbacks.borrow_mut();
        let cbs = map.entry(ih).or_insert_with(|| Vec::with_capacity(4));
        let id = cbs.last().map(|&(id, _)| id + 1).unwrap_or(0);
        cbs.push((id, cb));

        unsafe {
            IupSetCallback(ih, self.cb_name.as_ptr() as *const i8, self.cb_fn);
        }

        Token { id: id, ih: ih }.into()
    }

    pub fn add_callback<G>(&self, ih: *mut Ihandle, cb: G) -> T
    where Box<G>: CoerceUnsized<Box<F>>
    {
        self.add_callback_inner(ih, Box::new(cb) as Box<F>)
    }

    pub fn remove_callback(&self, ih: *mut Ihandle, token: T) {
        let token: Token = token.into();
        assert!(ih == token.ih, "token used with wrong control");
        let mut map = self.callbacks.borrow_mut();
        if let Some(cbs) = map.get_mut(&token.ih) {
            if let Some(index) = cbs.iter().position(|&(id, _)| id == token.id) {
                cbs.remove(index);
            } else {
                panic!("failed to remove callback");
            }
        }
    }
}

fn simple_callback<T>(ih: *mut Ihandle,
                      reg: &'static LocalKey<CallbackRegistry<FnMut(), T>>)
                      -> c_int where T: Into<Token> + From<Token> {
    reg.with(|reg| {
        if let Some(cbs) = reg.callbacks.borrow_mut().get_mut(&ih) {
            for cb in cbs {
                cb.1();
            }
        }
    });
    IUP_DEFAULT
}



// need to add a test that converting between LeaveWindowCallbackToken and Token either fails to
// compile or cannot cause unsafety (maybe just leave Token private)
pub struct LeaveWindowCallbackToken(Token);

impl From<LeaveWindowCallbackToken> for Token {
    fn from(LeaveWindowCallbackToken(t): LeaveWindowCallbackToken) -> Token { t }
}

impl From<Token> for LeaveWindowCallbackToken {
    fn from(t: Token) -> LeaveWindowCallbackToken { LeaveWindowCallbackToken(t) }
}

thread_local!(
    static LEAVE_WINDOW_CALLBACKS: CallbackRegistry<FnMut(), LeaveWindowCallbackToken> =
        CallbackRegistry::new("LEAVEWINDOW_CB", leave_window)
);

extern fn leave_window(ih: *mut Ihandle) -> c_int {
    simple_callback(ih, &LEAVE_WINDOW_CALLBACKS)
}



struct Event<'a, F: ?Sized + 'static, T: 'static + Into<Token> + From<Token>> {
    control: &'a mut Control,
    reg: &'static LocalKey<CallbackRegistry<F, T>>,
}

impl<'a, F: ?Sized, T: Into<Token> + From<Token>> Event<'a, F, T> {
    pub fn new(control: &'a mut Control, reg: &'static LocalKey<CallbackRegistry<F, T>>) -> Event<'a, F, T> {
        Event { control: control, reg: reg }
    }

    pub fn add_callback<G>(&mut self, cb: G) -> T
    where Box<G>: CoerceUnsized<Box<F>>
    {
        self.reg.with(|reg| reg.add_callback_inner(self.control.handle_mut(), Box::new(cb) as Box<F>))
    }

    pub fn remove_callback(&mut self, token: T) {
        self.reg.with(|reg| reg.remove_callback(self.control.handle_mut(), token))
    }
}

pub trait CommonCallbacks : Control {
    fn leave_window<'a>(&'a mut self) -> Event<'a, FnMut(), LeaveWindowCallbackToken>
    where &'a mut Self: CoerceUnsized<&'a mut Control> {
        Event::new(self as &mut Control, &LEAVE_WINDOW_CALLBACKS)
    }
}
