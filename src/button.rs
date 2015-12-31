/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use std::ptr;
use std::ops::CoerceUnsized;
use iup_sys::*;
use libc::c_int;
use super::{CommonAttributes, TitleAttribute, Control, MenuCommonCallbacks, NonMenuCommonCallbacks, ButtonCallback, UnwrapHandle};
use super::handle_rc::HandleRc;
use super::common_callbacks::{Event, CallbackRegistry, simple_callback, Token};

#[derive(Clone)]
pub struct Button(HandleRc);

impl Button {
    pub fn new() -> Button {
        unsafe {
            ::iup_open();
            let handle = IupButton(ptr::null_mut(), ptr::null_mut());
            assert!(handle != ptr::null_mut());
            let b = Button(HandleRc::new(handle));
            if cfg!(windows) {
                b.set_min_size(75, 0);
            }
            b
        }
    }

    fn action<'a>(&'a mut self) -> Event<'a, FnMut(), ButtonActionCallbackToken>
    where &'a mut Self: CoerceUnsized<&'a mut Control> {
        Event::new(self as &mut Control, &BUTTON_ACTION_CALLBACKS)
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
