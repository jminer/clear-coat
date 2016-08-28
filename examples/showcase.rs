/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

#[macro_use]
extern crate clear_coat;
extern crate iup_sys;
extern crate smallvec;

use std::rc::Rc;
use iup_sys::*;
use smallvec::SmallVec;

use clear_coat::*;
use clear_coat::common_attrs_cbs::*;

struct CursorsCanvas {
    canvas: Canvas,
}

impl CursorsCanvas {
    pub fn new() -> Self {
        CursorsCanvas {
            canvas: Canvas::new(),
        }
    }
}

impl CursorAttribute for CursorsCanvas {}

unsafe impl Control for CursorsCanvas {
    fn handle(&self) -> *mut Ihandle {
        self.canvas.handle()
    }
}

fn create_cursors_page() -> Box<Control> {
    let cursors_canvas = Rc::new(CursorsCanvas::new());

    let radios_info = [
        ("None", Cursor::None),
        ("Arrow", Cursor::Arrow),
        ("Busy", Cursor::Busy),
        ("Cross", Cursor::Cross),
        ("Hand", Cursor::Hand),
        ("Help", Cursor::Help),
        ("Move", Cursor::Move),
        ("ResizeN", Cursor::ResizeN),
        ("ResizeS", Cursor::ResizeS),
        ("ResizeNS", Cursor::ResizeNS),
        ("ResizeW", Cursor::ResizeW),
        ("ResizeE", Cursor::ResizeE),
        ("ResizeWE", Cursor::ResizeWE),
        ("ResizeNE", Cursor::ResizeNE),
        ("ResizeSW", Cursor::ResizeSW),
        ("ResizeNW", Cursor::ResizeNW),
        ("ResizeSE", Cursor::ResizeSE),
        ("Text", Cursor::Text),
    ];

    let mut radios = SmallVec::<[Toggle; 32]>::new();
    for &(text, cur) in radios_info.iter() {
        let toggle = Toggle::new();
        toggle.set_title(text);
        let cursors_canvas2 = cursors_canvas.clone();
        toggle.action_event().add(move |checked| {
            if checked { cursors_canvas2.set_cursor(cur); }
        });
        radios.push(toggle);
    }

    let grid = grid_box!(
        &radios[0],
        &radios[1],
        &radios[2],
        &radios[3],
        &radios[4],
        &radios[5],
        &radios[6],
        &radios[7],
        &radios[8],
        &radios[9],
        &radios[10],
        &radios[11],
        &radios[12],
        &radios[13],
        &radios[14],
        &radios[15],
        &radios[16],
        &radios[17],
    );
    grid.set_num_div(NumDiv::Fixed(2));
    grid.fit_all_to_children();
    let page = vbox!(
        &cursors_canvas,
        hbox!(
            fill!(),
            Radio::with_child(&grid),
            fill!(),
        ),
    );
    Box::new(page)
}

fn create_file_dialog_page() -> Box<Control> {

    let type_check_box = Toggle::new();
    type_check_box.set_title("Dialog Type:");

    let open_radio = Toggle::new();
    open_radio.set_title("Open");
    let save_radio = Toggle::new();
    save_radio.set_title("Save");
    let dir_radio = Toggle::new();
    dir_radio.set_title("Directory");
    let type_radio = Radio::with_child(&vbox!(
        &open_radio,
        &save_radio,
        &dir_radio,
    ));

    let dir_check_box = Toggle::new();
    dir_check_box.set_title("Directory:");

    let dir_text_box = Text::new();

    let show_dialog = Button::with_title("Show Dialog");
    let type_check_box_capt = type_check_box.clone();
    let open_radio_capt = open_radio.clone();
    let save_radio_capt = save_radio.clone();
    let dir_check_box_capt = dir_check_box.clone();
    let dir_text_box_capt = dir_text_box.clone();
    show_dialog.action_event().add(move || {
        let dialog = FileDlg::new();
        if type_check_box_capt.is_on() {
            dialog.set_dialog_type(if open_radio_capt.is_on() {
                FileDialogType::Open
            } else if save_radio_capt.is_on() {
                FileDialogType::Save
            } else {
                FileDialogType::Dir
            })
        }
        if dir_check_box_capt.is_on() {
            dialog.set_directory(&dir_text_box_capt.value());
        }
        dialog.popup(ScreenPosition::CenterParent, ScreenPosition::CenterParent)
              .expect("couldn't show file dialog");
    });

    let grid = grid_box!(
        type_check_box, type_radio,
        dir_check_box, dir_text_box,
        fill!(), show_dialog,
    );
    grid.set_alignment_lin_all(VAlignment::Top);
    grid.set_num_div(NumDiv::Fixed(2));
    grid.fit_all_to_children();
    Box::new(grid)
}

fn main() {

    let dialog = Dialog::new();

    let tabs = Tabs::new();

    tabs.append_tabs(&[
        TabInfo::new(&*create_cursors_page()).title("Cursors"),
        TabInfo::new(&*create_file_dialog_page()).title("File Dialog"),
    ]);

    dialog.append(&tabs).expect("failed to build the window");
    dialog.set_title("Showcase");

    dialog.show_xy(ScreenPosition::Center, ScreenPosition::Center)
          .expect("failed to show the window");
    main_loop();
}
