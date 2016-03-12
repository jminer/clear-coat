/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use std::cell::{RefCell};
use std::collections::HashMap;
use iup_sys::*;
use smallvec::SmallVec;
use super::handle_rc::{
    add_ldestroy_callback,
    HandleRc,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ExtraRefKey(pub i32);
#[derive(Debug)]
struct ExtraRef(ExtraRefKey, HandleRc);

// on ldestroy, remove all entries from this hashmap
thread_local!(
    static EXTRA_REFS: RefCell<HashMap<*mut Ihandle, SmallVec<[ExtraRef; 2]>>> =
        RefCell::new(HashMap::new())
);

pub fn add_extra_ref(ih: *mut Ihandle, key: ExtraRefKey, r: HandleRc) {
    EXTRA_REFS.with(|map| {
        let mut map = map.borrow_mut();
        let mut vec = map.entry(ih).or_insert_with(|| {
            add_ldestroy_callback(ih, |ih| {
                EXTRA_REFS.with(|map| {
                    map.borrow_mut().remove(&ih);
                });
            });
            SmallVec::new()
        });
        vec.push(ExtraRef(key, r));
    })
}

// Returns true if the key was found and the element was removed and false otherwise.
pub fn remove_extra_ref(ih: *mut Ihandle, key: ExtraRefKey) -> bool {
    EXTRA_REFS.with(|map| {
        if let Some(ref mut vec) = map.borrow_mut().get_mut(&ih) {
            let orig_len = vec.len();
            // SmallVec doesn't have retain :(
            //vec.retain(|ExtraRef(ref k, _)| k != key);
            for i in (0..vec.len()).rev() {
                if vec[i].0 == key {
                    vec.remove(i);
                }
            }
            return vec.len() != orig_len;
        }
        false
    })
}
