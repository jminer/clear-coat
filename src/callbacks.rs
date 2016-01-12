/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem;
use std::ops::{CoerceUnsized};
use std::panic::{self, RecoverSafe};
use std::rc::Rc;
use std::thread::LocalKey;
use libc::{c_int, c_char};
use iup_sys::*;
use super::{Control, MouseButton, KeyboardMouseStatus};
use super::handle_rc::add_destroy_callback;

pub enum CallbackAction {
    Default,
    // Close is not needed because it is just as easy to call IupExitLoop()
    // and then the API is smaller.
    // Close,
    Ignore,
    Continue,
}

// To avoid letting panics unwind into C stack frames, we need to catch the panic,
// store it away, and check it later when there's no C code on the stack.
thread_local!(
    static PANIC_PAYLOAD: RefCell<Option<Box<Any + Send + 'static>>> = RefCell::new(None)
);

pub fn set_panic_payload(payload: Box<Any + Send + 'static>) {
    PANIC_PAYLOAD.with(|cell|
        *cell.borrow_mut() = Some(payload)
    );
}

pub fn is_panic_pending() -> bool {
    PANIC_PAYLOAD.with(|cell| cell.borrow().is_some())
}

pub fn take_panic_payload() -> Option<Box<Any + Send + 'static>> {
    PANIC_PAYLOAD.with(|cell| cell.borrow_mut().take())
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

pub struct CallbackRegistry<F: ?Sized + 'static, T: Into<Token> + From<Token>> {
    cb_name: &'static str,
    cb_fn: Icallback,
    pub callbacks: Rc<RefCell<HashMap<*mut Ihandle, Vec<(usize, Box<F>)>>>>,
    phantom: PhantomData<*const T>,
}

impl<F: ?Sized, T: Into<Token> + From<Token>> CallbackRegistry<F, T> {
    // Icallback is the most common type of callback, but there are many exceptions. If the
    // callback's signature does not match Icallback, just cast to Icallback.
    pub fn new(cb_name: &'static str, cb_fn: Icallback) -> CallbackRegistry<F, T> {
        let reg = CallbackRegistry {
            cb_name: cb_name,
            cb_fn: cb_fn,
            callbacks: Rc::new(RefCell::new(HashMap::new())),
            phantom: PhantomData,
        };
        // When a control is destroyed, we need to remove all of its callbacks.
        let callbacks = reg.callbacks.clone();
        add_destroy_callback(move |ih| { callbacks.borrow_mut().remove(&ih); });
        reg
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

struct AssertRecoverSafeVal<T: ?Sized>(T);

impl<T: ?Sized> RecoverSafe for AssertRecoverSafeVal<T> {}

pub fn with_callbacks<F, G: ?Sized, T>(ih: *mut Ihandle,
                                       reg: &'static LocalKey<CallbackRegistry<G, T>>, f: F)
                                       -> c_int
                                       where F: FnOnce(&mut [(usize, Box<G>)]) -> c_int,
                                             G: 'static,
                                             T: Into<Token> + From<Token> {
    let h = AssertRecoverSafeVal(f);
    let result = panic::recover(move || {
        reg.with(move |reg| {
            if let Some(cbs) = reg.callbacks.borrow_mut().get_mut(&ih) {
                h.0(cbs)
            } else {
                IUP_DEFAULT
            }
        })
    });
    match result {
        Ok(r) => r,
        Err(err) => {
            set_panic_payload(err);
            unsafe { IupExitLoop(); }
            IUP_DEFAULT
        },
    }
}

pub fn simple_callback<T>(ih: *mut Ihandle,
                      reg: &'static LocalKey<CallbackRegistry<FnMut(), T>>)
                      -> c_int where T: Into<Token> + From<Token> {
    with_callbacks(ih, reg, |cbs| {
        for cb in cbs {
            cb.1();
        }
        IUP_DEFAULT
    })
}

pub struct Event<'a, F: ?Sized + 'static, T: 'static + Into<Token> + From<Token>> {
    control: &'a Control,
    reg: &'static LocalKey<CallbackRegistry<F, T>>,
}

impl<'a, F: ?Sized, T: Into<Token> + From<Token>> Event<'a, F, T> {
    pub fn new(control: &'a Control, reg: &'static LocalKey<CallbackRegistry<F, T>>) -> Event<'a, F, T> {
        Event { control: control, reg: reg }
    }

    pub fn add<G>(&self, cb: G) -> T
    where Box<G>: CoerceUnsized<Box<F>>
    {
        self.reg.with(|reg| reg.add_callback_inner(self.control.handle(), Box::new(cb) as Box<F>))
    }

    pub fn remove(&self, token: T) {
        self.reg.with(|reg| reg.remove_callback(self.control.handle(), token))
    }
}



pub trait MenuCommonCallbacks : Control {
    // fn map_event();
    // fn unmap_event();
    // fn destroy_event();
}


callback_token!(EnterWindowCallbackToken);
thread_local!(
    static ENTER_WINDOW_CALLBACKS: CallbackRegistry<FnMut(), EnterWindowCallbackToken> =
        CallbackRegistry::new("ENTERWINDOW_CB", enter_window_cb)
);
extern fn enter_window_cb(ih: *mut Ihandle) -> c_int {
    simple_callback(ih, &ENTER_WINDOW_CALLBACKS)
}

callback_token!(LeaveWindowCallbackToken);
thread_local!(
    static LEAVE_WINDOW_CALLBACKS: CallbackRegistry<FnMut(), LeaveWindowCallbackToken> =
        CallbackRegistry::new("LEAVEWINDOW_CB", leave_window_cb)
);
extern fn leave_window_cb(ih: *mut Ihandle) -> c_int {
    simple_callback(ih, &LEAVE_WINDOW_CALLBACKS)
}

pub trait NonMenuCommonCallbacks : MenuCommonCallbacks {
    // fn get_focus_event();
    // fn kill_focus_event();

    fn enter_window_event<'a>(&'a self) -> Event<'a, FnMut(), EnterWindowCallbackToken>
    where &'a Self: CoerceUnsized<&'a Control> {
        Event::new(self as &Control, &ENTER_WINDOW_CALLBACKS)
    }

    fn leave_window_event<'a>(&'a self) -> Event<'a, FnMut(), LeaveWindowCallbackToken>
    where &'a Self: CoerceUnsized<&'a Control> {
        Event::new(self as &Control, &LEAVE_WINDOW_CALLBACKS)
    }

    // fn k_any_event();
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
        CallbackRegistry::new("BUTTON_CB",  unsafe { mem::transmute::<_, Icallback>(button_cb) })
);
unsafe extern fn button_cb(ih: *mut Ihandle, button: c_int, pressed: c_int, x: c_int, y: c_int, status: *mut c_char) -> c_int {
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
    fn button_event<'a>(&'a self) -> Event<'a, FnMut(&ButtonArgs), ButtonCallbackToken>
    where &'a Self: CoerceUnsized<&'a Control> {
        Event::new(self as &Control, &BUTTON_CALLBACKS)
    }
}
