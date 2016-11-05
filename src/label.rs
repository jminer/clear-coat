/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use super::control_prelude::*;

#[derive(Clone)]
pub struct Label(HandleRc);

impl Label {
    // Creates an empty label.
    pub fn new() -> Self {
        unsafe {
            ::iup_open();
            let ih = IupLabel(ptr::null_mut());
            Label(HandleRc::new(ih))
        }
    }

    /// Creates a label with text to be shown on it.
    pub fn with_title(title: &str) -> Self {
        unsafe {
            ::iup_open();
            let mut buf = SmallVec::<[u8; 64]>::new();
            let c_title = str_to_c_vec(title, &mut buf);
            let ih = IupLabel(c_title);
            Label(HandleRc::new(ih))
        }
    }

    /// Gets the horizontal alignment of the contents of the label.
    pub fn halignment(&self) -> ::HAlignment {
        unsafe {
            let slice = get_str_attribute_slice(self.handle(), "ALIGNMENT\0");
            ::HAlignment::from_str(slice.as_bytes().split(|c| *c == b':').next().unwrap())
        }
    }

    /// Sets the horizontal alignment of the contents of the label.
    pub fn set_halignment(&self, alignment: ::HAlignment) -> &Self {
        set_str_attribute(self.handle(), "ALIGNMENT\0", alignment.to_str());
        self
    }
}

impl_control_traits!(Label);

impl ActiveAttribute for Label {}
impl ExpandAttribute for Label {}
impl MinMaxSizeAttribute for Label {}
impl TipAttribute for Label {}
impl TitleAttribute for Label {}
impl VisibleAttribute for Label {}

impl MenuCommonCallbacks for Label {}

impl EnterLeaveWindowCallbacks for Label {}
