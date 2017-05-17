/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use super::control_prelude::*;
use super::extra_refs::{add_extra_ref, remove_extra_ref, ExtraRefKey};

const EXTRA_REF_RUNNING_TIMER: ExtraRefKey = ExtraRefKey(1);

#[derive(Clone)]
pub struct Timer(HandleRc);

impl Timer {
    pub fn new() -> Self {
        unsafe {
            ::iup_open();
            let ih = IupTimer();
            Timer(HandleRc::new(ih))
        }
    }

    pub fn time(&self) -> u32 {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "TIME\0");
            s.parse().expect("could not convert TIME to an integer")
        }
    }

    pub fn set_time(&self, time: u32) -> &Self {
        set_str_attribute(self.handle(), "TIME\0", &format!("{}\0", time));
        self
    }

    pub fn is_running(&self) -> bool {
        unsafe {
            get_str_attribute_slice(self.handle(), "RUN\0") == "YES"
        }
    }

    pub fn set_running(&self, running: bool) -> &Self {
        let s = if running {
            // Add an extra ref (just of the timer to itself) so that it doesn't get destroyed
            // while it is running, even if there's no handle to it. Windows Forms and Swing
            // timers work the same way.
            add_extra_ref(self.handle(), EXTRA_REF_RUNNING_TIMER, self.0.clone());
            "YES\0"
        } else {
            remove_extra_ref(self.handle(), EXTRA_REF_RUNNING_TIMER);
            "NO\0"
        };
        set_str_attribute(self.handle(), "RUN\0", s);
        self
    }
}

impl_control_traits!(Timer);

impl_callbacks! {
    Timer {
        "ACTION_CB\0" => action_event {
            ACTION_CALLBACKS<FnMut(), TimerActionCallbackToken>
        }
        unsafe extern fn timer_action_cb(ih: *mut Ihandle) -> c_int {
            simple_callback(ih, &ACTION_CALLBACKS)
        }
    }
}
