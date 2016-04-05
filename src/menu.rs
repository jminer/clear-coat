/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use super::control_prelude::*;
use super::containers::{
    Container,
    wrapper_to_handle_vec,
};

/// Implemented by controls that can be added as children of a `Menu`.
pub trait MenuSubitem : Control {
}

#[derive(Clone)]
pub struct Menu(HandleRc);

impl Menu {
    pub fn new() -> Self {
        unsafe {
            ::iup_open();
            // You can pass NULL for the array, even though the docs don't explicitly mention it.
            // (They only say to use NULL as the last element of the array.)
            let handle = IupMenuv(ptr::null_mut());
            Menu(HandleRc::new(handle))
        }
    }

    pub fn with_children(children: &[&MenuSubitem]) -> Self {
        unsafe {
            // got to already be IupOpen()ed
            let mut handles = wrapper_to_handle_vec(children);
            Menu::from_handles(handles.as_mut_ptr())
        }
    }

    pub unsafe fn from_handles(children: *mut *mut Ihandle) -> Self {
        let handle = IupMenuv(children);
        Menu(HandleRc::new(handle))
    }
}

impl_control_traits!(Menu);

impl Container for Menu {}

impl MenuCommonCallbacks for Menu {}



#[derive(Clone)]
pub struct Submenu(HandleRc);

impl Submenu {
    pub fn with_menu(menu: &Menu) -> Self {
        unsafe {
            ::iup_open();
            let ih = IupSubmenu(ptr::null_mut(), menu.handle());
            Submenu(HandleRc::new(ih))
        }
    }

    pub fn with_title_and_menu(title: &str, menu: &Menu) -> Self {
        unsafe {
            ::iup_open();
            let mut buf = SmallVec::<[u8; 64]>::new();
            let c_title = str_to_c_vec(title, &mut buf);
            let ih = IupSubmenu(c_title, menu.handle());
            Submenu(HandleRc::new(ih))
        }
    }
}

impl_control_traits!(Submenu);

impl MenuSubitem for Submenu {}



#[derive(Clone)]
pub struct Item(HandleRc);

impl Item {
    pub fn new() -> Self {
        unsafe {
            ::iup_open();
            let ih = IupItem(ptr::null_mut(), ptr::null_mut());
            Item(HandleRc::new(ih))
        }
    }

    pub fn with_title(title: &str) -> Self {
        unsafe {
            ::iup_open();
            let mut buf = SmallVec::<[u8; 64]>::new();
            let c_title = str_to_c_vec(title, &mut buf);
            let ih = IupItem(c_title, ptr::null_mut());
            Item(HandleRc::new(ih))
        }
    }

    pub fn action_event<'a>(&'a self) -> Event<'a, FnMut(), ItemActionCallbackToken>
    where &'a Self: CoerceUnsized<&'a Control> {
        Event::new(self as &'a Control, &ITEM_ACTION_CALLBACKS)
    }
}

impl_control_traits!(Item);

impl MenuSubitem for Item {}

impl ActiveAttribute for Item {}
impl TitleAttribute for Item {}


callback_token!(ItemActionCallbackToken);
thread_local!(
    static ITEM_ACTION_CALLBACKS: CallbackRegistry<FnMut(), ItemActionCallbackToken> =
        CallbackRegistry::new("ACTION", item_action_cb)
);
extern fn item_action_cb(ih: *mut Ihandle) -> c_int {
    simple_callback(ih, &ITEM_ACTION_CALLBACKS)
}



#[derive(Clone)]
pub struct Separator(HandleRc);

impl Separator {
    pub fn new() -> Self {
        unsafe {
            ::iup_open();
            let ih = IupSeparator();
            Separator(HandleRc::new(ih))
        }
    }
}

impl_control_traits!(Separator);

impl MenuSubitem for Separator {}
