/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use std::borrow::Cow;
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
    str_to_c_vec,
    get_str_attribute,
    get_str_attribute_slice,
    set_str_attribute,
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

    pub fn value(&self) -> String {
        get_str_attribute(self.handle(), "VALUE\0")
    }
    pub unsafe fn value_slice(&self) -> Cow<str> {
        get_str_attribute_slice(self.handle(), "VALUE\0")
    }

    pub fn set_value(&self, value: &str) -> &Self {
        set_str_attribute(self.handle(), "VALUE\0", value);
        self
    }

    pub fn append(&self, text: &str) -> &Self {
        set_str_attribute(self.handle(), "APPEND\0", text);
        self
    }

    pub fn append_newline(&self) -> bool {
        unsafe {
            get_str_attribute_slice(self.handle(), "APPENDNEWLINE\0") == "YES"
        }
    }

    pub fn set_append_newline(&self, enabled: bool) -> &Self {
        set_str_attribute(self.handle(), "APPENDNEWLINE\0", if enabled { "YES\0" } else { "NO\0" });
        self
    }

    pub fn multiline(&self) -> bool {
        unsafe {
            get_str_attribute_slice(self.handle(), "MULTILINE\0") == "YES"
        }
    }

    pub fn set_multiline(&self, multiline: bool) -> &Self {
        set_str_attribute(self.handle(), "MULTILINE\0", if multiline { "YES\0" } else { "NO\0" });
        self
    }

    pub fn visible_lines(&self) -> i32 {
        unsafe {
            let val = get_str_attribute_slice(self.handle(), "VISIBLELINES\0");
            val.parse().expect("could not convert VISIBLELINES to an integer")
        }
    }

    pub fn set_visible_lines(&self, lines: i32) -> &Self {
        set_str_attribute(self.handle(), "VISIBLELINES\0", &lines.to_string());
        self
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
