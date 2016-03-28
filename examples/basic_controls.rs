/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

#[macro_use]
extern crate clear_coat;
extern crate iup_sys;
use iup_sys::*;
use std::ptr;

use clear_coat::*;
use clear_coat::common_attrs_cbs::*;

fn main() {
    let button1 = Button::new();
    button1.set_title("Push Me");
    button1.action_event().add(|| println!("you pushed it!"));

    let button2 = Button::new();
    button2.set_title("Print text");

    let button3 = Button::new();
    button3.set_title("Hi");
    let button4 = Button::new();
    button4.set_title("Hi");
    let text_box = Text::new();
    text_box.set_multiline(true);
    text_box.set_visible_lines(5);
    let text_box2 = text_box.clone();
    button2.action_event().add(move || println!("Text box text: \"{}\"", text_box2.value()));
    let check_box = Toggle::new();
    check_box.set_title("Check me");

    let new_item = Item::with_title("New");
    let open_item = Item::with_title("Open");
    let file_menu = Menu::with_children(&[
        &new_item,
        &open_item,
    ]);

    // TODO: there is handle::is_null()
    // TODO: have setters return self, so that hbox and vbox, etc. can be configured without being stored in a variable?
    let dialog = Dialog::with_child(&vbox!(
        button1, fill!(),
        hbox!(fill!(), button3, button4),
        text_box,
        check_box,
        button2));
    // unsafe {
    //     let mut subsubmenu = vec![IupItem("Foo\0".as_ptr() as *const i8, ptr::null_mut()), ptr::null_mut()];
    //     let mut submenu = vec![
    //         IupSubmenu("Edit\0".as_ptr() as *const i8, IupMenuv(subsubmenu.as_mut_ptr())),
    //         IupMenuv(ptr::null_mut()), ptr::null_mut()
    //     ];
    //     IupSetAttributeHandle(dialog.handle(), "MENU\0".as_ptr() as *const i8, IupMenuv(submenu.as_mut_ptr()));

    // }
    dialog.set_menu(Some(&Menu::with_children(&[
        &Submenu::with_title_and_menu("File", &file_menu),
    ])));

    dialog.show_xy(ScreenPosition::Center, ScreenPosition::Center)
          .expect("There was a problem showing the window");
    dialog.set_title("Howdy");
    let t = dialog.leave_window_event().add(|| println!("left window 1"));
    dialog.leave_window_event().add(|| println!("left window 2"));
    dialog.leave_window_event().remove(t);
    clear_coat::main_loop();
}
