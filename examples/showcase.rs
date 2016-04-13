/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

#[macro_use]
extern crate clear_coat;
extern crate iup_sys;
use iup_sys::*;

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

unsafe impl Control for CursorsCanvas {
    fn handle(&self) -> *mut Ihandle {
        self.canvas.handle()
    }
}

fn create_cursors_page() -> Box<Control> {
    let page = vbox!(
        &CursorsCanvas::new()
    );
    Box::new(page)
}

fn main() {

    let dialog = Dialog::new();

    let tabs = Tabs::new();

    tabs.append_tabs(&[
        TabInfo::new(&*create_cursors_page()).title("Cursors"),
        TabInfo::new(&Fill::new()).title("File Dialog"),
    ]);

    dialog.append(&tabs).expect("failed to build the window");
    dialog.set_title("Showcase");

    dialog.show_xy(ScreenPosition::Center, ScreenPosition::Center)
          .expect("failed to show the window");
    main_loop();
}
