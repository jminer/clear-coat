/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */


use std::ptr;
use iup_sys::*;
use smallvec::SmallVec;
use super::{
    Control,
    UnwrapHandle,
};
use super::attributes::{
    str_to_c_vec,
};
use super::handle_rc::HandleRc;

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
            let ih = IupMenuv(ptr::null_mut());
            Menu(HandleRc::new(ih))
        }
    }
}

impl_control_traits!(Menu);



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
            let mut buf = SmallVec::<[u8; 32]>::new(); // TODO: change to 64 after upgrading smallvec
            let c_title = str_to_c_vec(title, &mut buf);
            let ih = IupSubmenu(c_title, menu.handle());
            Submenu(HandleRc::new(ih))
        }
    }
}

impl_control_traits!(Submenu);

impl MenuSubitem for Submenu {
}



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
            let mut buf = SmallVec::<[u8; 32]>::new(); // TODO: change to 64 after upgrading smallvec
            let c_title = str_to_c_vec(title, &mut buf);
            let ih = IupItem(c_title, ptr::null_mut());
            Item(HandleRc::new(ih))
        }
    }
}

impl_control_traits!(Item);

impl MenuSubitem for Item {
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

impl MenuSubitem for Separator {
}
