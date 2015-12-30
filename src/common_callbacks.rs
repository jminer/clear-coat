/* Copyright 2015 Jordan Miner
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 */

use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr;
use std::thread::LocalKey;
use super::{Control};

pub type Icallback = extern fn(ih: *mut u32) -> i32;

pub struct CallbackRegistry<F: ?Sized> {
    pub callbacks: RefCell<HashMap<*mut u32, Vec<(usize, Box<F>)>>>,
}

impl<F: ?Sized> CallbackRegistry<F> {
    pub fn remove_callback(&self) {
        let mut map = self.callbacks.borrow_mut();
        if let Some(cbs) = map.get_mut(&ptr::null_mut()) {
            if let Some(index) = cbs.iter().position(|&(id, _)| id == 1) {
                cbs.remove(index);
            } else {
                panic!("failed to remove callback");
            }
        }
    }
}

pub struct Event<F: ?Sized + 'static> {
    pub reg: &'static LocalKey<CallbackRegistry<F>>,
}

impl<F: ?Sized> Event<F> {
    pub fn remove_callback(&mut self) {
        self.reg.with(|reg| reg.remove_callback())
    }
}



thread_local!(
    static LEAVE_WINDOW_CALLBACKS: CallbackRegistry<FnMut()> =
        CallbackRegistry { callbacks: RefCell::new(HashMap::new()), }
);

pub trait NonMenuCommonCallbacks : Control {
    fn leave_window<'a>(&'a mut self) -> Event<FnMut()> {
        Event { reg: &LEAVE_WINDOW_CALLBACKS }
    }
}

