/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use super::control_prelude::*;
use super::containers::{
    Container,
    NonDialogContainer,
};

#[derive(Clone)]
pub struct Radio(HandleRc);

impl Radio {
    pub fn new() -> Self {
        unsafe {
            ::iup_open();
            let ih = IupRadio(ptr::null_mut());
            Radio(HandleRc::new(ih))
        }
    }

    pub fn with_child(child: &Control) -> Self {
        unsafe {
            ::iup_open();
            let ih = IupRadio(child.handle());
            Radio(HandleRc::new(ih))
        }
    }
}

impl_control_traits!(Radio);

impl Container for Radio {}
impl NonDialogContainer for Radio {}

impl ExpandAttribute for Radio {}
impl MinMaxSizeAttribute for Radio {}
impl VisibleAttribute for Radio {}

impl MenuCommonCallbacks for Radio {}
