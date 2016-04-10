/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use super::control_prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use super::{
    Button,
    Container,
    Dialog,
    Fill,
    Hbox,
    Label,
    Popup,
    ScreenPosition,
    Vbox,
};

#[derive(Copy, Clone, Debug)]
pub enum AlarmResult {
    Button1,
    Button2,
    Button3,
}

#[derive(Clone, Debug)]
pub struct AlarmBuilder<'t, 'm, 'b> {
    title: &'t str,
    message: &'m str,
    buttons: Vec<&'b str>,
}

// This started as a wrapper for IupAlarm(); however, when using it, the buttons are too tall on
// Windows, and they aren't right aligned. A bonus of implementing it myself is the ability to show
// more than three buttons. Usually, you want <= 3, but there's a couple places Qt Creator uses
// 4 or 5 buttons, and I've never found it hard to use.
impl<'t, 'm, 'b> AlarmBuilder<'t, 'm, 'b> {
    pub fn new(title: &'t str, message: &'m str, button1: &'b str) -> AlarmBuilder<'t, 'm, 'b> {
        AlarmBuilder {
            title: title,
            message: message,
            buttons: vec![button1],
        }
    }

    pub fn add_button(&mut self, text: &'b str) -> &mut Self {
        self.buttons.push(text);
        self
    }

    pub fn popup(&self) -> i32 {
        ::iup_open();
        let result = Rc::new(RefCell::new(0));

        let dialog = Dialog::new();

        let button_box = hbox!(fill!());
        for (i, b) in self.buttons.iter().enumerate() {
            let button = Button::with_title(b);
            let (result_cap, dialog_cap) = (result.clone(), dialog.clone());
            button.action_event().add(move || {
                *result_cap.borrow_mut() = i as i32;
                dialog_cap.hide().expect("failed to hide dialog");
            });
            button_box.append(&button).expect("failed to build alarm button box");
        }
        set_str_attribute(button_box.handle(), "GAP\0", "5");

        dialog.append(vbox!(
                Label::with_title(self.message),
                fill!(),
                button_box,
            ).set_top_level_margin_and_gap()).expect("failed to build alarm dialog");
        dialog.set_title(self.title);
        dialog.popup(ScreenPosition::CenterParent, ScreenPosition::CenterParent)
                .expect("failed to show alarm dialog");

        let result = *result.borrow(); // it should be possible to write this without a variable
        result
    }
}
