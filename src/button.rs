/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use iup_sys::*;
use super::{CommonAttributes,TitleAttribute,Wrapper};

pub struct Button(*mut Ihandle);

impl Button {
}

impl Wrapper for Button {
    fn handle(&self) -> *const Ihandle { self.0 }
    fn handle_mut(&mut self) -> *mut Ihandle { self.0 }
}

impl CommonAttributes for Button {}

impl TitleAttribute for Button {}
