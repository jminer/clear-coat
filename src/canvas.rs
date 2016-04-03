/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

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

    pub fn draw_size(&self) -> (i32, i32) {
        get_int_int_attribute(self.handle(), "DRAWSIZE\0")
    }
}

impl_control_traits!(Canvas);

impl ActiveAttribute for Canvas {}
impl CanFocusAttribute for Canvas {}
impl CanvasAttributes for Canvas {}
impl CursorAttribute for Canvas {}
impl MinMaxSizeAttribute for Canvas {}
impl TipAttribute for Canvas {}
impl VisibleAttribute for Canvas {}

impl MenuCommonCallbacks for Canvas {}
impl GetKillFocusCallbacks for Canvas {}
impl EnterLeaveWindowCallbacks for Canvas {}
impl ButtonCallback for Canvas {}
impl CanvasCallbacks for Canvas {}
impl ResizeCallback for Canvas {}
