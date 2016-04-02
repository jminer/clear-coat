/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use libc::c_float;
#[cfg(windows)]
use winapi;
use super::control_prelude::*;

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
    #[cfg(unix)]
    xwindow: libc::c_ulong,
    #[cfg(windows)]
    hdc: winapi::HDC,
}
