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
    MinMaxSizeAttribute,
    TipAttribute,
    VisibleAttribute,
};
use super::callbacks::{
    CallbackRegistry,
    with_callbacks,
    MenuCommonCallbacks,
    EnterLeaveWindowCallbacks,
    GetKillFocusCallbacks,
    ValueChangedCallback,
    Event,
    Token,
};
use super::handle_rc::HandleRc;

#[derive(Clone)]
pub struct Text(HandleRc);

impl Text {
    pub fn new() -> Self {
        unsafe {
            ::iup_open();
            let ih = IupText(ptr::null_mut());
            Text(HandleRc::new(ih))
        }
    }
}

impl_control_traits!(Text);

impl ActiveAttribute for Text {}
impl MinMaxSizeAttribute for Text {}
impl TipAttribute for Text {}
impl VisibleAttribute for Text {}

impl MenuCommonCallbacks for Text {}
impl GetKillFocusCallbacks for Text {}
impl EnterLeaveWindowCallbacks for Text {}
impl ValueChangedCallback for Text {}

impl_callbacks! {
    Text {
        "CARET_CB\0" => caret_event {
            CARET_CALLBACKS<FnMut(usize, usize, usize), CaretCallbackToken>
        }
        unsafe extern fn caret_cb(ih: *mut Ihandle, lin: c_int, col: c_int, pos: c_int) -> c_int {
            with_callbacks(ih, &CARET_CALLBACKS, |cbs| {
                for cb in cbs {
                    (&mut *cb.1.borrow_mut())(lin as usize, col as usize, pos as usize);
                }
                IUP_DEFAULT
            })
        }
    }
}
