/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use std::any::Any;
use std::cell::RefCell;
use std::collections::{hash_map, HashMap};
use std::marker::PhantomData;
use std::mem;
use std::ops::{CoerceUnsized};
use std::panic::{self, RecoverSafe};
use std::rc::Rc;
use std::thread::LocalKey;
use libc::{c_int, c_char};
use iup_sys::*;
use smallvec::SmallVec;
use super::attributes::str_to_c_vec;
use super::{Control, MouseButton, KeyboardMouseStatus};
use super::handle_rc::{add_ldestroy_callback, remove_ldestroy_callback};

pub enum CallbackAction {
    Default,
    // Close is not needed because it is just as easy to call IupExitLoop()
    // and then the API is smaller.
    Close,
    Ignore,
    Continue,
}

impl CallbackAction {
    #[allow(dead_code)]
    pub fn from_int(action: c_int) -> CallbackAction {
        match action {
            IUP_DEFAULT => CallbackAction::Default,
            IUP_IGNORE => CallbackAction::Ignore,
            IUP_CONTINUE => CallbackAction::Continue,
            _ => panic!("can't convert callback action"),
        }
    }

    pub fn to_int(&self) -> c_int {
        match *self {
            CallbackAction::Default => IUP_DEFAULT,
            CallbackAction::Close => IUP_CLOSE,
            CallbackAction::Ignore => IUP_IGNORE,
            CallbackAction::Continue => IUP_CONTINUE,
        }
    }
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
#[derive(Debug)]
pub struct Token {
    pub id: usize,
    pub ih: *mut Ihandle,
}

macro_rules! callback_token {
    ($name:ident) => {
        #[derive(Debug)]
        pub struct $name(Token);

        impl From<$name> for Token {
            fn from($name(t): $name) -> Token { t }
        }

        impl From<Token> for $name {
            fn from(t: Token) -> $name { $name(t) }
        }
    };
}

struct ControlCallbacks<F: ?Sized + 'static> {
    ldestroy_token: Token,
    // Copy-on-write is used on the vector in the `Rc` so that if a callback is added or removed
    // inside a callback, it can make the change to a copy of the vector. The in-progress
    // notification can continue iterating over the original vector. To make the vector
    // `Clone`, each function is wrapped in an `Rc`. Since each function is in a `RefCell`, there
    // is no need to have the vector in a RefCell. Iterating over the vector only requires read
    // access (even to reentrantly iterate over the vector because you can have multiple
    // immutable references), and for write access in add/remove_callback, `make_mut` is used.
    vec: Rc<Vec<(usize, Rc<RefCell<F>>)>>,
}

pub struct CallbackRegistry<F: ?Sized + 'static, T: Into<Token> + From<Token>> {
    cb_name: &'static str,
    cb_fn: Icallback,
    callbacks: Rc<RefCell<HashMap<*mut Ihandle, ControlCallbacks<F>>>>,
    phantom: PhantomData<*const T>,
}

impl<F: ?Sized, T: Into<Token> + From<Token>> CallbackRegistry<F, T> {
    // Icallback is the most common type of callback, but there are many exceptions. If the
    // callback's signature does not match Icallback, just cast to Icallback.
    pub fn new(cb_name: &'static str, cb_fn: Icallback) -> CallbackRegistry<F, T> {
        CallbackRegistry {
            cb_name: cb_name,
            cb_fn: cb_fn,
            callbacks: Rc::new(RefCell::new(HashMap::new())),
            phantom: PhantomData,
        }
    }

    // `add_callback` and `remove_callback` do not try to borrow the `RefCell` until they have
    // called `Rc::make_mut` (the `RefCell` could already be borrowed by `with_callbacks`). After
    // `Rc::make_mut`, it is guaranteed safe to borrow.
    fn add_callback_inner(&self, ih: *mut Ihandle, cb: Rc<RefCell<F>>) -> T {
        let mut map = self.callbacks.borrow_mut();
        let cc = map.entry(ih).or_insert_with(|| {
            let callbacks2 = self.callbacks.clone();
            let t = add_ldestroy_callback(ih, move |ih| { callbacks2.borrow_mut().remove(&ih); });
            ControlCallbacks { ldestroy_token: t, vec: Rc::new(Vec::with_capacity(4)) }
        });
        let cbs = Rc::make_mut(&mut cc.vec);
        let id = cbs.last().map(|&(id, _)| id + 1).unwrap_or(0);
        cbs.push((id, cb));

        unsafe {
            let mut buf = SmallVec::<[u8; 64]>::new();
            IupSetCallback(ih, str_to_c_vec(self.cb_name, &mut buf) as *const i8, self.cb_fn);
        }

        Token { id: id, ih: ih }.into()
    }

    pub fn add_callback<G>(&self, ih: *mut Ihandle, cb: G) -> T
    where Rc<RefCell<G>>: CoerceUnsized<Rc<RefCell<F>>>
    {
        self.add_callback_inner(ih, Rc::new(RefCell::new(cb)) as Rc<RefCell<F>>)
    }

    pub fn remove_callback(&self, ih: *mut Ihandle, token: T) {
        let token: Token = token.into();
        assert!(ih == token.ih, "token used with wrong control");
        let mut map = self.callbacks.borrow_mut();
        if let hash_map::Entry::Occupied(mut entry) = map.entry(token.ih) {
            let is_empty = {
                // Use make_mut() in case the vector is being iterated over.
                let cbs = Rc::make_mut(&mut entry.get_mut().vec);
                let index = cbs.iter().position(|&(id, _)| id == token.id).expect("failed to remove callback");
                cbs.remove(index);

                cbs.is_empty()
            };
            if is_empty {
                let ControlCallbacks { ldestroy_token, .. } = entry.remove();
                remove_ldestroy_callback(ldestroy_token);
            }

            // I could use the below code with non-lexical borrows.
            // Use make_mut() in case the vector is being iterated over.
            //let cbs = Rc::make_mut(&mut entry.get_mut().vec);
            //let index = cbs.iter().position(|&(id, _)| id == token.id).expect("failed to remove callback");
            //cbs.remove(index);

            //if cbs.is_empty() {
            //    let ControlCallbacks { ldestroy_token, .. } = entry.remove();
            //    remove_ldestroy_callback(ldestroy_token);
            //}
        }
    }
}

struct AssertRecoverSafeVal<T: ?Sized>(T);

impl<T: ?Sized> RecoverSafe for AssertRecoverSafeVal<T> {}

// Takes a function that takes one parameter that is a slice of (id, callback) tuples.
pub fn with_callbacks<F, G: ?Sized, T>(ih: *mut Ihandle,
                                       reg: &'static LocalKey<CallbackRegistry<G, T>>, f: F)
                                       -> c_int
                                       where F: FnOnce(&[(usize, Rc<RefCell<G>>)]) -> c_int,
                                             G: 'static,
                                             T: Into<Token> + From<Token> {
    let h = AssertRecoverSafeVal(f);
    let result = panic::recover(move || {
        reg.with(move |reg| {
            let cbs_rc = reg.callbacks.borrow().get(&ih).map(|cc| cc.vec.clone());
            if let Some(cbs) = cbs_rc {
                h.0(&*cbs)
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
            (&mut *cb.1.borrow_mut())();
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
    where Rc<RefCell<G>>: CoerceUnsized<Rc<RefCell<F>>>
    {
        self.reg.with(|reg| reg.add_callback_inner(self.control.handle(), Rc::new(RefCell::new(cb)) as Rc<RefCell<F>>))
    }

    pub fn remove(&self, token: T) {
        self.reg.with(|reg| reg.remove_callback(self.control.handle(), token))
    }
}



callback_token!(DestroyCallbackToken);
thread_local!(
    static DESTROY_CALLBACKS: CallbackRegistry<FnMut(), DestroyCallbackToken> =
        CallbackRegistry::new("DESTROY_CB", destroy_cb)
);
extern fn destroy_cb(ih: *mut Ihandle) -> c_int {
    simple_callback(ih, &DESTROY_CALLBACKS)
}

pub trait MenuCommonCallbacks : Control {
    // fn map_event();
    // fn unmap_event();

    fn destroy_event<'a>(&'a self) -> Event<'a, FnMut(), DestroyCallbackToken>
    where &'a Self: CoerceUnsized<&'a Control> {
        Event::new(self as &Control, &DESTROY_CALLBACKS)
    }
}


callback_token!(GetFocusCallbackToken);
thread_local!(
    static GET_FOCUS_CALLBACKS: CallbackRegistry<FnMut(), GetFocusCallbackToken> =
        CallbackRegistry::new("GETFOCUS_CB", get_focus_cb)
);
extern fn get_focus_cb(ih: *mut Ihandle) -> c_int {
    simple_callback(ih, &GET_FOCUS_CALLBACKS)
}

callback_token!(KillFocusCallbackToken);
thread_local!(
    static KILL_FOCUS_CALLBACKS: CallbackRegistry<FnMut(), KillFocusCallbackToken> =
        CallbackRegistry::new("KILLFOCUS_CB", kill_focus_cb)
);
extern fn kill_focus_cb(ih: *mut Ihandle) -> c_int {
    simple_callback(ih, &KILL_FOCUS_CALLBACKS)
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

pub trait GetKillFocusCallbacks : Control {
    fn get_focus_event<'a>(&'a self) -> Event<'a, FnMut(), GetFocusCallbackToken>
    where &'a Self: CoerceUnsized<&'a Control> {
        Event::new(self as &Control, &GET_FOCUS_CALLBACKS)
    }

    fn kill_focus_event<'a>(&'a self) -> Event<'a, FnMut(), KillFocusCallbackToken>
    where &'a Self: CoerceUnsized<&'a Control> {
        Event::new(self as &Control, &KILL_FOCUS_CALLBACKS)
    }
}

pub trait EnterLeaveWindowCallbacks : Control {
    fn enter_window_event<'a>(&'a self) -> Event<'a, FnMut(), EnterWindowCallbackToken>
    where &'a Self: CoerceUnsized<&'a Control> {
        Event::new(self as &Control, &ENTER_WINDOW_CALLBACKS)
    }

    fn leave_window_event<'a>(&'a self) -> Event<'a, FnMut(), LeaveWindowCallbackToken>
    where &'a Self: CoerceUnsized<&'a Control> {
        Event::new(self as &Control, &LEAVE_WINDOW_CALLBACKS)
    }
}

pub trait KAnyCallback : Control {
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
    static BUTTON_CALLBACKS: CallbackRegistry<FnMut(&ButtonArgs) -> CallbackAction, ButtonCallbackToken> =
        CallbackRegistry::new("BUTTON_CB",  unsafe { mem::transmute::<_, Icallback>(button_cb) })
);
unsafe extern fn button_cb(ih: *mut Ihandle, button: c_int, pressed: c_int, x: c_int, y: c_int, status: *mut c_char) -> c_int {
    // Maybe the callback should be able to return Ignore (and thus this function return
    // IUP_IGNORE). My main hesitation is that IUP's docs state that it is system
    // dependent: "On some controls if IUP_IGNORE is returned the action is ignored (this is
    // system dependent)." Plus, it doesn't seem really useful and is more verbose.
    with_callbacks(ih, &BUTTON_CALLBACKS, |cbs| {
        let args = ButtonArgs {
            button: MouseButton::from_int(button),
            pressed: pressed != 0,
            x: x as i32,
            y: y as i32,
            status: KeyboardMouseStatus::from_cstr(status),
            _dummy: (),
        };
        let mut action = CallbackAction::Default;
        for cb in cbs {
            match (&mut *cb.1.borrow_mut())(&args) {
                CallbackAction::Default => {},
                cb_action => action = cb_action,
            }
        }
        action.to_int()
    })
}

pub trait ButtonCallback {
    fn button_event<'a>(&'a self) -> Event<'a, FnMut(&ButtonArgs) -> CallbackAction, ButtonCallbackToken>
    where &'a Self: CoerceUnsized<&'a Control> {
        Event::new(self as &Control, &BUTTON_CALLBACKS)
    }
}
