/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, hash_map};
use std::ops::Deref;
use std::rc::{Rc, Weak};
use std::ptr;
use libc::c_int;
use iup_sys::*;
use callbacks::Token;

thread_local!(
    static LDESTROY_CALLBACKS: RefCell<HashMap<*mut Ihandle, Vec<(usize, Box<FnMut(*mut Ihandle)>)>>> =
        RefCell::new(HashMap::new())
);


// simplified version of CallbackRegistry::add_callback
// An ldestroy callback should hold a reference to a control, so that when it is dropped a control
// is dropped. Doing so will cause the LDESTROY_CALLBACKS HashMap to be accessed, causing a panic
// due to being borrowed by remove_ldestroy_callback.
pub fn add_ldestroy_callback_inner(ih: *mut Ihandle, cb: Box<FnMut(*mut Ihandle) + 'static>) -> Token {
    LDESTROY_CALLBACKS.with(|reg| {
        let mut map = reg.borrow_mut();
        let cbs = map.entry(ih).or_insert_with(|| { Vec::with_capacity(4) });
        let id = cbs.last().map(|&(id, _)| id + 1).unwrap_or(0);
        cbs.push((id, cb));

        Token { id: id, ih: ih }
    })
}

pub fn add_ldestroy_callback<F: 'static + FnMut(*mut Ihandle)>(ih: *mut Ihandle, cb: F) -> Token {
    add_ldestroy_callback_inner(ih, Box::new(cb))
}

// simplified version of CallbackRegistry::remove_callback
pub fn remove_ldestroy_callback(token: Token) {
    LDESTROY_CALLBACKS.with(|reg| {
        let mut map = reg.borrow_mut();
        if let hash_map::Entry::Occupied(mut entry) = map.entry(token.ih) {
            let is_empty = {
                let cbs = entry.get_mut();
                let index = cbs.iter().position(|&(id, _)| id == token.id).expect("failed to remove callback");
                // TODO: if this causes ldestroy_cb() to be called, it will panic
                cbs.remove(index);

                cbs.is_empty()
            };
            if is_empty {
                entry.remove();
            }

            // I could use the below with non-lexical borrows.
            //let cbs = entry.get_mut();
            //let index = cbs.iter().position(|&(id, _)| id == token.id).expect("failed to remove callback");
            // TODO: if this causes ldestroy_cb() to be called, it will panic
            //cbs.remove(index);

            //if cbs.is_empty() {
            //    entry.remove();
            //}
        }
    });
}

extern fn ldestroy_cb(ih: *mut Ihandle) -> c_int {
    handle_rc_destroy_cb(ih);
    LDESTROY_CALLBACKS.with(|cell| {
        if let Some(mut cbs) = cell.borrow_mut().remove(&ih) {
            for cb in cbs.iter_mut() {
                cb.1(ih);
            }
        }
        IUP_DEFAULT
    })
}

thread_local!(
    static EXISTING_HANDLES: RefCell<HashMap<*mut Ihandle, Weak<HandleBox>>> = RefCell::new(HashMap::new())
);

pub fn handle_rc_destroy_cb(ih: *mut Ihandle) {
    EXISTING_HANDLES.with(|cell| {
        let mut map = cell.borrow_mut();
        if let Some(weak) = map.remove(&ih) {
            // It should be removed when the last strong ref was dropped, so it should always be upgradable.
            let handle = weak.upgrade().expect("could not upgrade Weak in handle map");
            // Since the control is destroyed, zero out to prevent wrapper structs from
            // using after free.
            handle.set(ptr::null_mut());
        }
        // else: No wrapper handles currently exist. Don't need to do anything.


        // Detach any child that still has a reference from a wrapper so that it doesn't
        // get destroyed.
        unsafe {
            // Removing a child shifts the index of the following children by one as you'd expect.
            // The easiest way would be to iterate backward removing children. However,
            // IUP stores children as a singly-linked list starting with the first child.
            // So iterating reverse would be O(n^2) best case. Iterating forward would be O(n)
            // best case and O(n^2) worst case. However, using IupGetNextChild and IupGetBrother
            // is always O(n).
            let mut child = IupGetNextChild(ih, ptr::null_mut());
            while !child.is_null() {
                let brother = IupGetBrother(child);
                if map.contains_key(&child) {
                    IupDetach(child);
                }
                child = brother;
            }
        }
    });
}

/// A `HandleBox` is the wrapper's one reference to a control. If there are multiple `HandleRc`s,
/// they all refer to the same `HandleBox`. Since there will always be at most one `HandleBox` for
/// a certain `*mut Ihandle`, then when the `HandleBox` is dropped, there are no more references
/// from the wrapper to the `*mut Ihandle`.
struct HandleBox(Cell<*mut Ihandle>);

impl Deref for HandleBox {
    type Target = Cell<*mut Ihandle>;
    fn deref(&self) -> &Cell<*mut Ihandle> {
        &self.0
    }
}

impl Drop for HandleBox {
    fn drop(&mut self) {
        if self.get() == ptr::null_mut() {
            return;
        }
        EXISTING_HANDLES.with(|map| {
            map.borrow_mut().remove(&self.get()).expect("handle not found in map");
        });
        unsafe {
            if IupGetParent(self.get()) == ptr::null_mut()
            {
                //println!("deleting control with no parent: {:?}", self.get());
                IupDestroy(self.get());
            }
        }
    }
}

// As a future optimization, the implementation could be changed to not use Rc, since support
// for weak ptrs is not used. It would only get rid of one word of memory though. Not worth
// the time.
#[derive(Clone)]
pub struct HandleRc(Rc<HandleBox>);

impl HandleRc {
    pub unsafe fn new(ih: *mut Ihandle) -> HandleRc {
        assert!(ih != ptr::null_mut(), "handle must be non-null");
        let rc = EXISTING_HANDLES.with(|map|
            match map.borrow_mut().entry(ih) {
                hash_map::Entry::Occupied(entry) =>
                    entry.get().upgrade().expect("could not upgrade Weak in handle map"),
                hash_map::Entry::Vacant(entry) => {
                    IupSetCallback(ih, "LDESTROY_CB".as_ptr() as *const i8, ldestroy_cb);
                    let rc = Rc::new(HandleBox(Cell::new(ih)));
                    entry.insert(Rc::downgrade(&rc));
                    rc
                },
            }
        );
        HandleRc(rc)
    }

    pub fn get(&self) -> *mut Ihandle {
        self.0.get()
    }

    pub fn try_unwrap(self) -> Result<*mut Ihandle, HandleRc> {
        Rc::try_unwrap(self.0).map(|handle_box| {
            let handle = handle_box.get();
            // null out so that `HandleBox` doesn't IupDestroy() it and so that no other
            // `HandleRc` can return it
            handle_box.set(ptr::null_mut());
            handle
        }).map_err(|rc| HandleRc(rc))
    }
}
