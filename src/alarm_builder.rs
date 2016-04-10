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
    default_enter: Option<i32>,
    default_esc: Option<i32>,
}

// This started as a wrapper for IupAlarm(); however, when using it, the buttons are too tall on
// Windows, and they aren't right aligned.
// Advantages of reimplementing it:
// - The ability to show more than three buttons. Usually, you want <= 3, but there's a couple
//   places Qt Creator uses 4 or 5 buttons, and I've never found it hard to use.
// - The ability to explicitly set the default and esc buttons.
impl<'t, 'm, 'b> AlarmBuilder<'t, 'm, 'b> {
    pub fn new(title: &'t str, message: &'m str, button1: &'b str) -> AlarmBuilder<'t, 'm, 'b> {
        AlarmBuilder {
            title: title,
            message: message,
            buttons: vec![button1],
            default_enter: None,
            default_esc: None,
        }
    }

    pub fn add_button(&mut self, text: &'b str) -> &mut Self {
        self.buttons.push(text);
        self
    }

    pub fn default_enter(&mut self, default_enter: i32) -> &mut Self {
        self.default_enter = Some(default_enter);
        self
    }

    pub fn default_esc(&mut self, default_esc: i32) -> &mut Self {
        self.default_esc = Some(default_esc);
        self
    }

    pub fn popup(&self) -> i32 {
        if let Some(default_enter) = self.default_enter {
            assert!(default_enter >= 1 && default_enter <= self.buttons.len() as i32);
        }
        if let Some(default_esc) = self.default_esc {
            assert!(default_esc >= 1 && default_esc <= self.buttons.len() as i32);
        }

        ::iup_open();
        let result = Rc::new(RefCell::new(0));

        let dialog = Dialog::new();

        let mut default_button = None;
        let mut esc_button = None;
        let button_box = hbox!(fill!());
        for (i, b) in self.buttons.iter().enumerate() {
            let button = Button::with_title(b);
            let (result_cap, dialog_cap) = (result.clone(), dialog.clone());
            button.action_event().add(move || {
                *result_cap.borrow_mut() = i as i32;
                dialog_cap.hide().expect("failed to hide dialog");
            });
            button_box.append(&button).expect("failed to build alarm button box");

            let j = i as i32 + 1;
            if Some(j) == self.default_enter {
                default_button = Some(button);
            } else if Some(j) == self.default_esc {
                esc_button = Some(button);
            }
        }

        dialog.append(vbox!(
                Label::with_title(self.message),
                fill!(),
                button_box,
            ).set_top_level_margin_and_gap()).expect("failed to build alarm dialog");
        dialog.set_title(self.title);
        if let Some(b) = default_button {
            // set_default_enter isn't the right method to use here. If a button is focused,
            // then the default button is ignored. And setting the default button doesn't set what
            // is focused. So calling set_default_enter actually has no effect when you only have
            // buttons. If you had something besides buttons, you'd also want to call
            // set_default_enter here.
            dialog.set_start_focus(&b);
        }
        if let Some(b) = esc_button {
            dialog.set_default_esc(&b);
        }
        dialog.popup(ScreenPosition::CenterParent, ScreenPosition::CenterParent)
                .expect("failed to show alarm dialog");

        let result = *result.borrow() + 1; // it should be possible to write this without a variable
        result
    }
}
