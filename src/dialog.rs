/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use super::control_prelude::*;
use std::ffi::CStr;
use std::mem;
use super::{
    ScreenPosition,
    Menu,
    Popup,
};
use super::containers::Container;
use super::extra_refs::{
    ExtraRefKey,
    add_extra_ref,
    remove_extra_ref,
};

#[derive(Clone)]
pub struct Dialog(HandleRc);

const EXTRA_REF_MENU: ExtraRefKey = ExtraRefKey(0);

impl Dialog {
    pub fn new() -> Dialog {
        unsafe {
            ::iup_open();
            let ih = IupDialog(ptr::null_mut());
            let d = Dialog(HandleRc::new(ih));
            d.set_min_size(150, 0);
            d
        }
    }

    pub fn with_child(child: &Control) -> Dialog {
        unsafe {
            ::iup_open();
            let ih = IupDialog(child.handle());
            let d = Dialog(HandleRc::new(ih));
            d.set_min_size(150, 0);
            d
        }
    }

    pub unsafe fn from_handle(handle: *mut Ihandle) -> Dialog {
        // got to already be IupOpen()ed
        assert!(CStr::from_ptr(IupGetClassName(handle)).to_string_lossy() == "dialog");
        Dialog(HandleRc::new(handle))
    }

    pub fn show_xy(&self, x: ScreenPosition, y: ScreenPosition) -> Result<(), ()> {
        unsafe {
            if IupShowXY(self.handle(), x.to_int(), y.to_int()) == IUP_NOERROR {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    pub fn refresh(&self) {
        unsafe {
            IupRefresh(self.handle());
        }
    }

    pub fn set_menu(&self, menu: Option<&Menu>) {
        unsafe {
            //let old_menu_ih = get_attribute_handle(self.handle(), "MENU\0");
            let new_menu_ih = menu.map_or(ptr::null_mut(), |m| m.handle());
            if new_menu_ih.is_null() {
                reset_attribute(self.handle(), "MENU\0");
                remove_extra_ref(self.handle(), EXTRA_REF_MENU);
            } else {
                set_attribute_handle(self.handle(), "MENU\0", new_menu_ih);
                remove_extra_ref(self.handle(), EXTRA_REF_MENU);
                add_extra_ref(self.handle(), EXTRA_REF_MENU, HandleRc::new(new_menu_ih));
            }
            // From testing, I've found that a menu does get set as the dialog's child, but only
            // when the dialog is mapped.
            // (The menu's parent is set in iDialogSetMenuAttrib() in IUP.)
        }
    }

    pub fn show_event<'a>(&'a self) -> Event<'a, FnMut(ShowState) -> CallbackAction, ShowCallbackToken>
    where &'a Self: CoerceUnsized<&'a Control> {
        Event::new(self as &Control, &SHOW_CALLBACKS)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ShowState {
    Hide,
    Show,
    Restore,
    Minimize,
    Maximize,
}

impl ShowState {
    fn from_int(state: c_int) -> ShowState {
        match state {
            IUP_HIDE => ShowState::Hide,
            IUP_SHOW => ShowState::Show,
            IUP_RESTORE => ShowState::Restore,
            IUP_MINIMIZE => ShowState::Minimize,
            IUP_MAXIMIZE => ShowState::Maximize,
            _ => panic!("unknown ShowState"),
        }
    }

    fn to_int(state: ShowState) -> c_int {
        match state {
            ShowState::Hide => IUP_HIDE,
            ShowState::Show => IUP_SHOW,
            ShowState::Restore => IUP_RESTORE,
            ShowState::Minimize => IUP_MINIMIZE,
            ShowState::Maximize => IUP_MAXIMIZE,
        }
    }
}

callback_token!(ShowCallbackToken);
thread_local!(
    static SHOW_CALLBACKS: CallbackRegistry<FnMut(ShowState) -> CallbackAction, ShowCallbackToken> =
        CallbackRegistry::new("SHOW_CB", unsafe { mem::transmute::<_, Icallback>(show_cb) })
);
extern fn show_cb(ih: *mut Ihandle, state: c_int) -> c_int {
    with_callbacks(ih, &SHOW_CALLBACKS, |cbs| {
        let state = ShowState::from_int(state);

        let mut action = CallbackAction::Default;
        for cb in cbs {
            match (&mut *cb.1.borrow_mut())(state) {
                CallbackAction::Default => {},
                cb_action => action = cb_action,
            }
        }
        action.to_int()
    })
}

impl_control_traits!(Dialog);

impl Container for Dialog {}
impl Popup for Dialog {}

impl ActiveAttribute for Dialog {}
impl CursorAttribute for Dialog {}
impl MinMaxSizeAttribute for Dialog {}
impl TipAttribute for Dialog {}
impl TitleAttribute for Dialog {}
impl VisibleAttribute for Dialog {}

impl MenuCommonCallbacks for Dialog {}
impl GetKillFocusCallbacks for Dialog {}
impl EnterLeaveWindowCallbacks for Dialog {}
impl ResizeCallback for Dialog {}
