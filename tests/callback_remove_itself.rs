/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

#[macro_use]
extern crate clear_coat;

use std::cell::RefCell;
use std::rc::Rc;
use clear_coat::*;

// Tests that a callback can remove itself.

#[test]
fn test_callback_remove_itself() {
    let dialog = Dialog::new();
    let dialog2 = dialog.clone();
    let token = Rc::new(RefCell::new(None));
    let token2: Rc<RefCell<Option<ShowCallbackToken>>> = token.clone();
    *token.borrow_mut() = Some(dialog.show_event().add(move |_| {
        dialog2.show_event().remove(token2.borrow_mut().take().unwrap());
        CallbackAction::Close
    }));
    drop(token);
    dialog.show_xy(ScreenPosition::Center, ScreenPosition::Center).expect("could not show dialog");
    main_loop();
}
