/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem;
use std::ops::{CoerceUnsized};
use std::thread::LocalKey;
use libc::{c_int, c_char};
use iup_sys::*;
use super::{Control, MouseButton, KeyboardMouseStatus};

pub enum CallbackAction {
    Default,
    // Close is not needed because it is just as easy to call IupExitLoop()
    // and then the API is smaller.
    // Close,
    Ignore,
    Continue,
}

// If a callback's documentation does not specify valid return values, then only IUP_DEFAULT is
// supported.

// use LDESTROY_CB instead of DESTROY_CB

// TODO: if Token is public, then I need to make sure that converting from a MapCallbackToken to
// a DestroyToken can't cause unsafety
pub struct Token {
    id: usize,
    ih: *mut Ihandle,
}

macro_rules! callback_token {
    ($name:ident) => {
        pub struct $name(Token);

        impl From<$name> for Token {
            fn from($name(t): $name) -> Token { t }
        }

        impl From<Token> for $name {
            fn from(t: Token) -> $name { $name(t) }
        }
    };
}

pub struct CallbackRegistry<F: ?Sized, T: Into<Token> + From<Token>> {
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

pub fn simple_callback<T>(ih: *mut Ihandle,
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

pub struct Event<'a, F: ?Sized + 'static, T: 'static + Into<Token> + From<Token>> {
    control: &'a Control,
    reg: &'static LocalKey<CallbackRegistry<F, T>>,
}

impl<'a, F: ?Sized, T: Into<Token> + From<Token>> Event<'a, F, T> {
    pub fn new(control: &'a Control, reg: &'static LocalKey<CallbackRegistry<F, T>>) -> Event<'a, F, T> {
        Event { control: control, reg: reg }
    }

    pub fn add_callback<G>(&self, cb: G) -> T
    where Box<G>: CoerceUnsized<Box<F>>
    {
        self.reg.with(|reg| reg.add_callback_inner(self.control.handle(), Box::new(cb) as Box<F>))
    }

    pub fn remove_callback(&self, token: T) {
        self.reg.with(|reg| reg.remove_callback(self.control.handle(), token))
    }
}



pub trait MenuCommonCallbacks : Control {
    // fn map();
    // fn unmap();
    // fn destroy();
}


callback_token!(EnterWindowCallbackToken);
thread_local!(
    static ENTER_WINDOW_CALLBACKS: CallbackRegistry<FnMut(), EnterWindowCallbackToken> =
        CallbackRegistry::new("ENTERWINDOW_CB", enter_window)
);
extern fn enter_window(ih: *mut Ihandle) -> c_int {
    simple_callback(ih, &ENTER_WINDOW_CALLBACKS)
}

callback_token!(LeaveWindowCallbackToken);
thread_local!(
    static LEAVE_WINDOW_CALLBACKS: CallbackRegistry<FnMut(), LeaveWindowCallbackToken> =
        CallbackRegistry::new("LEAVEWINDOW_CB", leave_window)
);
extern fn leave_window(ih: *mut Ihandle) -> c_int {
    simple_callback(ih, &LEAVE_WINDOW_CALLBACKS)
}

pub trait NonMenuCommonCallbacks : MenuCommonCallbacks {
    // fn get_focus();
    // fn kill_focus();

    fn enter_window<'a>(&'a self) -> Event<'a, FnMut(), EnterWindowCallbackToken>
    where &'a Self: CoerceUnsized<&'a Control> {
        Event::new(self as &Control, &ENTER_WINDOW_CALLBACKS)
    }

    fn leave_window<'a>(&'a self) -> Event<'a, FnMut(), LeaveWindowCallbackToken>
    where &'a Self: CoerceUnsized<&'a Control> {
        Event::new(self as &Control, &LEAVE_WINDOW_CALLBACKS)
    }

    // fn k_any();
}


#[derive(Clone)]
pub struct ButtonArgs {
    pub button: MouseButton,
    pub pressed: bool,
    pub x: i32,
    pub y: i32,
    pub status: KeyboardMouseStatus,
    _dummy: (),
}

callback_token!(ButtonCallbackToken);
thread_local!(
    static BUTTON_CALLBACKS: CallbackRegistry<FnMut(&ButtonArgs), ButtonCallbackToken> =
        CallbackRegistry::new("BUTTON_CB",  unsafe { mem::transmute::<_, Icallback>(button) })
);
unsafe extern fn button(ih: *mut Ihandle, button: c_int, pressed: c_int, x: c_int, y: c_int, status: *mut c_char) -> c_int {
    // Maybe the callback should be able to return Ignore (and thus this function return
    // IUP_IGNORE). My main hesitation is that IUP's docs state that it is system
    // dependent: "On some controls if IUP_IGNORE is returned the action is ignored (this is
    // system dependent)." Plus, it doesn't seem really useful and is more verbose.
    BUTTON_CALLBACKS.with(|reg| {
        if let Some(cbs) = reg.callbacks.borrow_mut().get_mut(&ih) {
            let args = ButtonArgs {
                button: MouseButton::from_int(button),
                pressed: pressed != 0,
                x: x as i32,
                y: y as i32,
                status: KeyboardMouseStatus::from_cstr(status),
                _dummy: (),
            };
            for cb in cbs {
                cb.1(&args);
            }
        }
    });
    IUP_DEFAULT
}

pub trait ButtonCallback {
    fn button<'a>(&'a self) -> Event<'a, FnMut(&ButtonArgs), ButtonCallbackToken>
    where &'a Self: CoerceUnsized<&'a Control> {
        Event::new(self as &Control, &BUTTON_CALLBACKS)
    }
}
