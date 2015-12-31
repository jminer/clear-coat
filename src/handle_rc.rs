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

thread_local!(
    static EXISTING_HANDLES: RefCell<HashMap<*mut Ihandle, Weak<HandleBox>>> = RefCell::new(HashMap::new())
);

pub fn ihandle_destroyed(ih: *mut Ihandle) {
    EXISTING_HANDLES.with(|map|
        if let Some(weak) = map.borrow_mut().remove(&ih) {
            // It should be removed when the last strong ref was dropped, so it should always be upgradable.
            let handle = weak.upgrade().expect("could not upgrade Weak in handle map");
            // Since the control is destroyed, zero out to prevent wrapper structs from
            // using after free.
            handle.set(ptr::null_mut());
        }
        // else: No wrapper handles currently exist. Don't need to do anything.
    );
}

extern fn ldestroy_cb(ih: *mut Ihandle) -> c_int {
    ihandle_destroyed(ih);
    EXISTING_HANDLES.with(|map| {
        unsafe {
            let count = IupGetChildCount(ih);
            for i in 0..count {
                let child = IupGetChild(ih, i);
                if map.borrow().contains_key(&child) {
                    IupDetach(child);
                }
            }
        }
    });
    IUP_DEFAULT
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
                IupDestroy(self.get());
            }
        }
    }
}

// As a future optimization, the implementation could be changed to not use Rc, since support
// for weak ptrs is not used. Then HandleRc would only be a single indirection instead of two,
// and would get rid of half the allocations. Probably not worth the time though.
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
