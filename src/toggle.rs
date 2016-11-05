/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use super::control_prelude::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ToggleState {
    On,
    Off,
    NotDef,
}

impl ToggleState {
    fn from_str(s: &[u8]) -> Self {
        match s {
            b"ON" => ToggleState::On,
            b"OFF" => ToggleState::Off,
            b"NOTDEF" => ToggleState::NotDef,
            _ => panic!("unknown ToggleState"),
        }
    }

    fn to_str(self) -> &'static str {
        match self {
            ToggleState::On => "ON",
            ToggleState::Off => "OFF",
            ToggleState::NotDef => "NOTDEF",
        }
    }
}

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

    pub fn value(&self) -> ToggleState {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "VALUE\0");
            ToggleState::from_str(s.as_bytes())
        }
    }

    pub fn set_value(&self, value: ToggleState) -> &Self {
        set_str_attribute(self.handle(), "VALUE\0", value.to_str());
        self
    }

    pub fn is_on(&self) -> bool {
        self.value() == ToggleState::On
    }

    pub fn set_on(&self, on: bool) -> &Self {
        self.set_value(if on { ToggleState::On } else { ToggleState::Off });
        self
    }

    pub fn three_state(&self) -> bool {
        unsafe {
            get_str_attribute_slice(self.handle(), "3STATE\0") == "YES"
        }
    }

    pub fn set_three_state(&self, enabled: bool) -> &Self {
        set_str_attribute(self.handle(), "3STATE\0", if enabled { "YES\0" } else { "NO\0" });
        self
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
