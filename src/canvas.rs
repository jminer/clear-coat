/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use std::ptr;
use iup_sys::*;
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
    MenuCommonCallbacks,
    EnterLeaveWindowCallbacks,
    GetKillFocusCallbacks,
    ButtonCallback,
};
use super::handle_rc::HandleRc;

#[derive(Clone)]
pub struct Canvas(HandleRc);

impl Canvas {
    pub fn new() -> Self {
        unsafe {
            ::iup_open();
            let ih = IupCanvas(ptr::null_mut());
            Canvas(HandleRc::new(ih))
        }
    }

}

impl_control_traits!(Canvas);

impl ActiveAttribute for Canvas {}
impl MinMaxSizeAttribute for Canvas {}
impl TipAttribute for Canvas {}
impl VisibleAttribute for Canvas {}

impl MenuCommonCallbacks for Canvas {}
impl GetKillFocusCallbacks for Canvas {}
impl EnterLeaveWindowCallbacks for Canvas {}
impl ButtonCallback for Canvas {}

#[derive(Clone)]
pub struct PaintingArgs {
    clip_rect: (i32, i32, i32, i32),
    //#[cfg(any(feature = "cairo"))]
    //cairo_cr: Cairo,
    //#[cfg(windows)]
    //hdc: winapi::HDC,
}
