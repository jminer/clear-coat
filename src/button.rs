/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use std::ptr;
use std::ops::CoerceUnsized;
use iup_sys::*;
use libc::{c_char, c_int};
use super::{
    Control,
    UnwrapHandle,
};
use super::attributes::{
    CommonAttributes,
    TitleAttribute,
};
use super::callbacks::{
    Event,
    Token,
    CallbackRegistry,
    simple_callback,
    MenuCommonCallbacks,
    NonMenuCommonCallbacks,
    ButtonCallback,
};
use super::handle_rc::HandleRc;

#[derive(Clone)]
pub struct Button(HandleRc);

impl Button {
    pub fn new() -> Button {
        unsafe {
            ::iup_open();
            let ih = IupButton(ptr::null_mut(), ptr::null_mut());
            let b = Button(HandleRc::new(ih));
            if cfg!(windows) {
                b.set_min_size(75, 0);
            }
            b
        }
    }

    pub fn with_title(title: &str) -> Button {
        unsafe {
            ::iup_open();
            let ih = IupButton(title.as_ptr() as *const c_char, ptr::null_mut());
            let b = Button(HandleRc::new(ih));
            if cfg!(windows) {
                b.set_min_size(75, 0);
            }
            b
        }
    }

    pub fn action<'a>(&'a self) -> Event<'a, FnMut(), ButtonActionCallbackToken>
    where &'a Self: CoerceUnsized<&'a Control> {
        Event::new(self as &'a Control, &BUTTON_ACTION_CALLBACKS)
    }
}

impl_control_traits!(Button);

impl CommonAttributes for Button {}

impl TitleAttribute for Button {}

impl MenuCommonCallbacks for Button {}
impl NonMenuCommonCallbacks for Button {}

impl ButtonCallback for Button {}


callback_token!(ButtonActionCallbackToken);
thread_local!(
    static BUTTON_ACTION_CALLBACKS: CallbackRegistry<FnMut(), ButtonActionCallbackToken> =
        CallbackRegistry::new("ACTION", button_action)
);
extern fn button_action(ih: *mut Ihandle) -> c_int {
    simple_callback(ih, &BUTTON_ACTION_CALLBACKS)
}
