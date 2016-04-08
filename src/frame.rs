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
pub struct Frame(HandleRc);

impl Frame {
    pub fn new() -> Self {
        unsafe {
            ::iup_open();
            let ih = IupFrame(ptr::null_mut());
            let f = Frame(HandleRc::new(ih));
            f.set_title("");
            f
        }
    }

    pub fn with_child(child: &Control) -> Self {
        unsafe {
            ::iup_open();
            let ih = IupFrame(child.handle());
            let f = Frame(HandleRc::new(ih));
            f.set_title("");
            f
        }
    }
}

impl_control_traits!(Frame);

impl Container for Frame {}
impl NonDialogContainer for Frame {}

impl ActiveAttribute for Frame {}
impl MinMaxSizeAttribute for Frame {}
impl TitleAttribute for Frame {}
impl VisibleAttribute for Frame {}

impl MenuCommonCallbacks for Frame {}
