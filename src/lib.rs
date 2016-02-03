/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

#![feature(coerce_unsized)]
#![feature(std_panic)]
#![feature(recover)]

extern crate libc;
extern crate iup_sys;
extern crate kernel32;
extern crate smallvec;

#[cfg(windows)]
fn get_thread_id() -> isize {
    use kernel32::GetCurrentThreadId;
    unsafe { GetCurrentThreadId() as isize }
}

macro_rules! impl_control_traits {
    ($control:ident) => {
        unsafe impl Control for ::$control {
            fn handle(&self) -> *mut Ihandle {
                assert!(!self.0.get().is_null(), "attempted to use destroyed control");
                ::check_thread();
                self.0.get()
            }
        }

        unsafe impl UnwrapHandle for ::$control {
            fn try_unwrap_handle(self) -> Result<*mut Ihandle, Self> {
                assert!(!self.0.get().is_null(), "attempted to use destroyed control");
                self.0.try_unwrap().map_err(|handle_rc| ::$control(handle_rc))
            }
        }
    };
}

#[macro_use]
mod callbacks;
#[macro_use]
mod containers;

mod attributes;
mod button;
mod dialog;
mod file_dlg;
mod handle_rc;

pub use dialog::{Dialog, ShowCallbackToken, ShowState};
pub use file_dlg::{FileDlg};
pub use button::Button;
pub use containers::{Container, Fill, Hbox, Vbox, NumDiv, GridBox};
pub use callbacks::{CallbackAction, Event};

// With this layout, you can glob import this module's contents but selectively import the
// above types if you want.
pub mod common_attrs_cbs {
    pub use attributes::{CommonAttributes, TitleAttribute};
    pub use callbacks::{MenuCommonCallbacks, GetKillFocusCallbacks, EnterLeaveWindowCallbacks, ButtonCallback};
}

use std::ptr;
use std::sync::atomic::{AtomicIsize, Ordering, ATOMIC_ISIZE_INIT};
use libc::{c_char, c_int};
use iup_sys::*;

// returns false if a panic is pending, but cannot be propagated because the main loop level is too high
fn propagate_panic() -> bool {
    if callbacks::is_panic_pending() && main_loop_level() > 0 {
        exit_loop();
        return false;
    }
    if let Some(payload) = callbacks::take_panic_payload() {
        // TODO: once a new nightly is released, use this
        //::std::panic::propagate(payload);
        panic!(payload.downcast_ref::<String>().unwrap_or(&String::new()).clone());
    }
    true
}

/// Executes the user interaction until `exit_loop` is called or the last visible dialog is hidden.
///
/// If you cascade many calls to `main_loop`, there must be a `exit_loop` call for each cascade
/// level, as hiding all dialogs will close only one level. Call `main_loop_level` to obtain the
/// current level.
///
/// If `main_loop` is called without any visible dialogs and no active timers, the application will
/// hang, and it will not be possible to close the main loop. The process will have to be
/// interrupted by the system.
///
/// When the last visible dialog is hidden, the `exit_loop` function is automatically called,
/// causing the `main_loop` to return. To avoid that, call `set_lock_loop(true)` before hiding the
/// last dialog.
pub fn main_loop() {
    unsafe {
        iup_open();

        // Callbacks can be called before main_loop() ever is. If they have been and have
        // panicked, we need to panic without calling IupMainLoop(). (IupMainLoopLevel() will
        // be 0 in a callback if it is running before IupMainLoop() has been called.)
        if !propagate_panic() { return; }
        IupMainLoop();
        propagate_panic();
    }
}

pub fn main_loop_level() -> i32 {
    unsafe {
        IupMainLoopLevel()
    }
}

pub fn exit_loop() {
    unsafe {
        IupExitLoop();
    }
}


fn iup_open() {
    check_thread();
    unsafe {
        if IupOpen(ptr::null_mut(), ptr::null_mut()) == IUP_OPENED {
            return;
        }
        attributes::set_str_attribute(ptr::null_mut(), "UTF8MODE", "YES");
        attributes::set_str_attribute(ptr::null_mut(), "UTF8MODE_FILE", "YES");
    }
}

static THREAD_ID: AtomicIsize = ATOMIC_ISIZE_INIT;

fn check_thread() {
    let thread_id = get_thread_id();
    let prev = THREAD_ID.compare_and_swap(0, thread_id, Ordering::SeqCst);
    assert!(prev == 0 || prev == thread_id, "IUP/Clear Coat functions must be called from a single thread");
}

// Part of the contract of implementing this trait is that no invalid handle
// is returned. Either the handle will stay valid for the life of the object or
// the method will panic.
pub unsafe trait Control {
    fn handle(&self) -> *mut Ihandle;

    fn detach(&self) {
        unsafe { IupDetach(self.handle()); }
    }

    fn reparent(&self, new_parent: &Container, ref_child: Option<&Control>) -> Result<(), ()> {
        unsafe {
            let ref_child = ref_child.map(|c| c.handle()).unwrap_or(ptr::null_mut());
            if IupReparent(self.handle(), new_parent.handle(), ref_child) == IUP_NOERROR {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    fn get_dialog(&self) -> Option<Dialog> {
        unsafe {
            let handle = IupGetDialog(self.handle());
            if handle == ptr::null_mut() {
                None
            } else {
                Some(Dialog::from_handle(handle))
            }
        }
    }
}

// If this wrapper has the only reference, it gives up shared ownership of the *mut Ihandle.
pub unsafe trait UnwrapHandle : Sized {
    fn try_unwrap_handle(self) -> Result<*mut Ihandle, Self>;
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ScreenPosition { // This is the name IUP uses: SCREENPOSITION attribute
    Absolute(i32),
    Left,
    Right,
    Top,
    Bottom,
    Center,
    MousePos,
    CenterParent,
    Current,
}

impl ScreenPosition {
    #[allow(dead_code)]
    fn from_int_x(i: c_int) -> ScreenPosition {
       match i {
           IUP_LEFT => ScreenPosition::Left,
           IUP_RIGHT => ScreenPosition::Right,
           IUP_CENTER => ScreenPosition::Center,
           IUP_MOUSEPOS => ScreenPosition::MousePos,
           IUP_CENTERPARENT => ScreenPosition::CenterParent,
           IUP_CURRENT => ScreenPosition::Current,
           _ => ScreenPosition::Absolute(i),
       }
    }

    #[allow(dead_code)]
    fn from_int_y(i: c_int) -> ScreenPosition {
       match i {
           IUP_TOP => ScreenPosition::Top,
           IUP_BOTTOM => ScreenPosition::Bottom,
           _ => Self::from_int_x(i),
       }
    }

    fn to_int(self) -> c_int {
        match self {
            ScreenPosition::Absolute(i) => i,
            ScreenPosition::Left => IUP_LEFT,
            ScreenPosition::Right => IUP_RIGHT,
            ScreenPosition::Top => IUP_TOP,
            ScreenPosition::Bottom => IUP_BOTTOM,
            ScreenPosition::Center => IUP_CENTER,
            ScreenPosition::MousePos => IUP_MOUSEPOS,
            ScreenPosition::CenterParent => IUP_CENTERPARENT,
            ScreenPosition::Current => IUP_CURRENT,
        }
    }
}

pub trait Popup : Control {
    fn popup(&self, x: ScreenPosition, y: ScreenPosition) -> Result<(), ()> {
        unsafe {
            if IupPopup(self.handle(), x.to_int(), y.to_int()) == IUP_NOERROR {
                Ok(())
            } else {
                Err(())
            }
        }
    }
}

#[derive(Copy,Clone)]
pub enum MouseButton {
    Button1,
    Button2,
    Button3,
    Button4,
    Button5,
}

impl MouseButton {
    fn from_int(i: c_int) -> MouseButton {
        match i {
            IUP_BUTTON1 => MouseButton::Button1,
            IUP_BUTTON2 => MouseButton::Button2,
            IUP_BUTTON3 => MouseButton::Button3,
            IUP_BUTTON4 => MouseButton::Button4,
            IUP_BUTTON5 => MouseButton::Button5,
            _ => panic!("unknown mouse button"),
        }
    }

    #[allow(dead_code)]
    fn to_int(self) -> c_int {
        match self {
            MouseButton::Button1 => IUP_BUTTON1,
            MouseButton::Button2 => IUP_BUTTON2,
            MouseButton::Button3 => IUP_BUTTON3,
            MouseButton::Button4 => IUP_BUTTON4,
            MouseButton::Button5 => IUP_BUTTON5,
        }
    }
}

#[derive(Clone)]
pub struct KeyboardMouseStatus {
    shift_pressed: bool,
    control_pressed: bool,
    alt_pressed: bool,
    sys_pressed: bool,
    button1_pressed: bool,
    button2_pressed: bool,
    button3_pressed: bool,
    button4_pressed: bool,
    button5_pressed: bool,
}

impl KeyboardMouseStatus {
    unsafe fn from_cstr(s: *const c_char) -> KeyboardMouseStatus {
        KeyboardMouseStatus {
            shift_pressed: iup_isshift(s),
            control_pressed: iup_iscontrol(s),
            alt_pressed: iup_isalt(s),
            sys_pressed: iup_issys(s),
            button1_pressed: iup_isbutton1(s),
            button2_pressed: iup_isbutton2(s),
            button3_pressed: iup_isbutton3(s),
            button4_pressed: iup_isbutton4(s),
            button5_pressed: iup_isbutton5(s),
        }
    }
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum VAlignment {
    Top,
    Center,
    Bottom,
}

impl VAlignment {
    fn from_str(s: &[u8]) -> VAlignment {
        match s {
            b"ATOP" => VAlignment::Top,
            b"ACENTER" => VAlignment::Center,
            b"ABOTTOM" => VAlignment::Bottom,
            _ => panic!("unknown VAlignment"),
        }
    }

    fn to_str(self) -> &'static str {
        match self {
            VAlignment::Top => "ATOP",
            VAlignment::Center => "ACENTER",
            VAlignment::Bottom => "ABOTTOM",
        }
    }
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HAlignment {
    Left,
    Center,
    Right,
}

impl HAlignment {
    fn from_str(s: &[u8]) -> HAlignment {
        match s {
            b"ALEFT" => HAlignment::Left,
            b"ACENTER" => HAlignment::Center,
            b"ARIGHT" => HAlignment::Right,
            _ => panic!("unknown HAlignment"),
        }
    }

    fn to_str(self) -> &'static str {
        match self {
            HAlignment::Left => "ALEFT",
            HAlignment::Center => "ACENTER",
            HAlignment::Right => "ARIGHT",
        }
    }
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Orientation {
    fn from_str(s: &[u8]) -> Orientation {
        match s {
            b"HORIZONTAL" => Orientation::Horizontal,
            b"VERTICAL" => Orientation::Vertical,
            _ => panic!("unknown Orientation"),
        }
    }

    fn to_str(self) -> &'static str {
        match self {
            Orientation::Horizontal => "HORIZONTAL",
            Orientation::Vertical => "VERTICAL",
        }
    }
}
