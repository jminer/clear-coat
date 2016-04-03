/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use std::ops::CoerceUnsized;
use std::ptr;
use iup_sys::*;
use libc::c_int;
use super::{
    Control,
    UnwrapHandle,
};
use super::attributes::{
    ActiveAttribute,
    CanFocusAttribute,
    MinMaxSizeAttribute,
    TipAttribute,
    TitleAttribute,
    VisibleAttribute,
};
use super::callbacks::{
    CallbackRegistry,
    with_callbacks,
    MenuCommonCallbacks,
    EnterLeaveWindowCallbacks,
    Event,
    Token,
};
use super::handle_rc::HandleRc;

#[derive(Clone)]
pub struct Toggle(HandleRc);

impl Toggle {
    pub fn new() -> Self {
        unsafe {
            ::iup_open();
            let ih = IupToggle(ptr::null_mut(), ptr::null_mut());
            Toggle(HandleRc::new(ih))
        }
    }

}

impl_control_traits!(Toggle);

impl ActiveAttribute for Toggle {}
impl CanFocusAttribute for Toggle {}
impl MinMaxSizeAttribute for Toggle {}
impl TipAttribute for Toggle {}
impl TitleAttribute for Toggle {}
impl VisibleAttribute for Toggle {}

impl MenuCommonCallbacks for Toggle {}
impl EnterLeaveWindowCallbacks for Toggle {}

impl_callbacks! {
    Toggle {
        "ACTION\0" => action_event {
            TOGGLE_ACTION_CALLBACKS<FnMut(bool), ToggleActionCallbackToken>
        }
        unsafe extern fn toggle_action_cb(ih: *mut Ihandle, state: c_int) -> c_int {
            let checked = state == 1;
            with_callbacks(ih, &TOGGLE_ACTION_CALLBACKS, |cbs| {
                for cb in cbs {
                    (&mut *cb.1.borrow_mut())(checked);
                }
                IUP_DEFAULT
            })
        }
    }
}
