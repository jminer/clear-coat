/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

pub use std::ops::CoerceUnsized;
pub use std::ptr;
pub use libc::c_int;
pub use iup_sys::*;
pub use smallvec::SmallVec;
pub use super::{
    Control,
    UnwrapHandle,
};
pub use super::attributes::{
    str_to_c_vec,
    get_attribute_ptr,
    get_str_attribute,
    get_str_attribute_slice,
    get_int_int_attribute,
    set_str_attribute,
    set_attribute_handle,
    reset_attribute,
    ActiveAttribute,
    CanFocusAttribute,
    CanvasAttributes,
    CursorAttribute,
    MinMaxSizeAttribute,
    OrientationAttribute,
    ScrollbarAttribute,
    TipAttribute,
    TitleAttribute,
    VisibleAttribute,
};
pub use super::callbacks::{
    CallbackAction,
    CallbackRegistry,
    simple_callback,
    with_callbacks,
    MenuCommonCallbacks,
    EnterLeaveWindowCallbacks,
    GetKillFocusCallbacks,
    ButtonCallback,
    ValueChangedCallback,
    CanvasCallbacks,
    ResizeCallback,
    Event,
    Token,
};
pub use super::handle_rc::HandleRc;
