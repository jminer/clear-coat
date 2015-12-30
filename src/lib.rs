/* Copyright 2015 Jordan Miner
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 */

mod common_callbacks;
pub use common_callbacks::{NonMenuCommonCallbacks};


pub unsafe trait Control {
    fn handle(&self) -> *const u32;
    fn handle_mut(&mut self) -> *mut u32;
}

pub struct Dialog(pub *mut u32);

unsafe impl Control for Dialog {
    fn handle(&self) -> *const u32 { self.0 }
    fn handle_mut(&mut self) -> *mut u32 { self.0 }
}

impl NonMenuCommonCallbacks for Dialog {}

