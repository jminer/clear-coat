/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use std::borrow::Cow;
use std::cell::Cell;
use std::ffi::CStr;
use libc::{c_char, c_int};
use iup_sys::*;
use smallvec::SmallVec;
use winapi;
use super::Control;

pub fn str_to_c_vec<'a: 'b, 'b, A: ::smallvec::Array<Item=u8>>(s: &'a str, buf: &'b mut SmallVec<A>) -> *const c_char {
    // `CString` in the std library doesn't check if the &str already ends in a null terminator
    // It allocates and pushes a 0 unconditionally. However, I can add the null to string literals
    // and avoid many allocations.
    if s.as_bytes().last() == Some(&0) && !s.as_bytes()[..s.len() - 1].contains(&b'\0') {
        s.as_bytes().as_ptr() as *const c_char
    } else {
        buf.grow(s.len() + 1);
        buf.extend(s.as_bytes().iter().map(|c| if *c == b'\0' { b'?' } else { *c }));
        buf.push(0);
        (&buf[..]).as_ptr() as *const c_char
    }
}

// These functions are named similiarly to IUP's functions. Summary of IUP functions:
// IupSetAttribute() - stores a pointer as an attribute's value; could be a pointer to constant string or an app's struct
// IupSetStrAttribute() - assumes you pass a null-term string and copies it before it returns
// IupSetAttributeHandle() - same as a IupSetHandle/IupSetAttribute pair; associates a name to an Ihandle then sets an attribute with that name

pub fn set_str_attribute(handle: *mut Ihandle, name: &str, value: &str) {
    unsafe {
        let mut name_buf = SmallVec::<[u8; 64]>::new();
        let c_name = str_to_c_vec(name, &mut name_buf);
        let mut value_buf = SmallVec::<[u8; 64]>::new();
        let c_value = str_to_c_vec(value, &mut value_buf);
        //println!("setting {:?} to {:?}", CStr::from_ptr(c_name).to_string_lossy(), CStr::from_ptr(c_value).to_string_lossy());
        IupSetStrAttribute(handle, c_name, c_value);
    }
}

// This function is unsafe because the caller is required to pass a valid pointer for `value`.
pub unsafe fn set_attribute_handle(ih: *mut Ihandle, name: &str, value: *mut Ihandle) {
    let mut name_buf = SmallVec::<[u8; 64]>::new();
    let c_name = str_to_c_vec(name, &mut name_buf);
    IupSetAttributeHandle(ih, c_name, value);
}

pub unsafe fn set_handle(name: &str, ih: *mut Ihandle) {
    let mut name_buf = SmallVec::<[u8; 64]>::new();
    let c_name = str_to_c_vec(name, &mut name_buf);
    IupSetHandle(c_name, ih);
}

pub fn reset_attribute(ih: *mut Ihandle, name: &str) {
    unsafe {
        let mut name_buf = SmallVec::<[u8; 64]>::new();
        let c_name = str_to_c_vec(name, &mut name_buf);
        IupResetAttribute(ih, c_name);
    }
}

pub fn get_attribute_ptr(handle: *mut Ihandle, name: &str) -> *mut c_char {
    unsafe {
        let mut name_buf = SmallVec::<[u8; 64]>::new();
        let c_name = str_to_c_vec(name, &mut name_buf);
        IupGetAttribute(handle as *mut Ihandle, c_name)
    }
}

// Unfortunately, the return value has to be copied because its lifetime isn't guaranteed.
// IUP's docs state:
//     "The returned pointer can be used safely even if IupGetGlobal or IupGetAttribute are called
//     several times. But not too many times, because it is an internal buffer and after IUP may
//     reuse it after around 50 calls."
pub fn get_str_attribute(handle: *mut Ihandle, name: &str) -> String {
    unsafe {
        get_str_attribute_slice(handle, name).into_owned()
    }
}

// This function isn't very error prone (see above), but isn't completely safe either.
pub unsafe fn get_str_attribute_slice(handle: *mut Ihandle, name: &str) -> Cow<str> {
    let value = get_attribute_ptr(handle, name);
    //println!("getting {:?}: {:?}", name, CStr::from_ptr(value).to_string_lossy());
    // It may be better to return `None` when `value` is null rather than an empty string,
    // but I'll try this for now.
    if value.is_null() {
        "".into()
    } else {
        CStr::from_ptr(value).to_string_lossy()
    }
}

#[cfg(for_future_use)] // silence dead_code warning (probably) the best way
pub fn get_attribute_handle(ih: *mut Ihandle, name: &str) -> *mut Ihandle {
    unsafe {
        let mut name_buf = SmallVec::<[u8; 64]>::new();
        let c_name = str_to_c_vec(name, &mut name_buf);
        IupGetAttributeHandle(ih, c_name)
    }
}

pub fn get_int_int_attribute(handle: *mut Ihandle, name: &str) -> (i32, i32) {
    unsafe {
        let mut name_buf = SmallVec::<[u8; 64]>::new();
        let c_name = str_to_c_vec(name, &mut name_buf);
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        assert!(IupGetIntInt(handle as *mut Ihandle,
                             c_name,
                             &mut x as *mut c_int,
                             &mut y as *mut c_int) == 2);
        (x, y)
    }
}

thread_local!(static UNIQUE_ATTRIBUTE_NAME_COUNTER: Cell<u32> = Cell::new(0));

fn get_unique_attribute_name() -> String {
    UNIQUE_ATTRIBUTE_NAME_COUNTER.with(|cell| {
        let counter = cell.get();
        cell.set(counter + 1);
        format!("CLEAR_COAT_UNIQUE_{}\0", counter)
    })
}


pub trait ActiveAttribute : Control {
    fn active(&self) -> bool {
        get_str_attribute(self.handle(), "ACTIVE") == "YES"
    }

    fn set_active(&self, active: bool) {
        set_str_attribute(self.handle(), "ACTIVE", if active { "YES" } else { "NO" });
    }
}

pub trait CanvasAttributes : Control {
    #[cfg(unix)]
    fn x_window(&self) -> c_long {
        get_attribute_ptr(self.handle(), "XWINDOW\0") as c_long
    }

    #[cfg(windows)]
    fn hwnd(&self) -> winapi::HDC {
        get_attribute_ptr(self.handle(), "HWND\0") as winapi::HDC
    }

    fn dx(&self) -> f32 {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "DX\0");
            s.parse().expect("could not convert DX to a number")
        }
    }

    fn set_dx(&self, dx: f32) -> &Self {
        set_str_attribute(self.handle(), "DX\0", &dx.to_string());
        self
    }

    fn dy(&self) -> f32 {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "DY\0");
            s.parse().expect("could not convert DY to a number")
        }
    }

    fn set_dy(&self, dy: f32) -> &Self {
        set_str_attribute(self.handle(), "DY\0", &dy.to_string());
        self
    }

    fn pos_x(&self) -> f32 {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "POSX\0");
            s.parse().expect("could not convert POSX to a number")
        }
    }

    fn set_pos_x(&self, pos_x: f32) -> &Self {
        set_str_attribute(self.handle(), "POSX\0", &pos_x.to_string());
        self
    }

    fn pos_y(&self) -> f32 {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "POSY\0");
            s.parse().expect("could not convert POSY to a number")
        }
    }

    fn set_pos_y(&self, pos_y: f32) -> &Self {
        set_str_attribute(self.handle(), "POSY\0", &pos_y.to_string());
        self
    }

    fn x_min(&self) -> f32 {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "XMIN\0");
            s.parse().expect("could not convert XMIN to a number")
        }
    }

    fn set_x_min(&self, x_min: f32) -> &Self {
        set_str_attribute(self.handle(), "XMIN\0", &x_min.to_string());
        self
    }

    fn x_max(&self) -> f32 {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "XMAX\0");
            s.parse().expect("could not convert XMAX to a number")
        }
    }

    fn set_x_max(&self, x_max: f32) -> &Self {
        set_str_attribute(self.handle(), "XMAX\0", &x_max.to_string());
        self
    }

    fn y_min(&self) -> f32 {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "YMIN\0");
            s.parse().expect("could not convert YMIN to a number")
        }
    }

    fn set_y_min(&self, y_min: f32) -> &Self {
        set_str_attribute(self.handle(), "YMIN\0", &y_min.to_string());
        self
    }

    fn y_max(&self) -> f32 {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "YMAX\0");
            s.parse().expect("could not convert YMAX to a number")
        }
    }

    fn set_y_max(&self, y_max: f32) -> &Self {
        set_str_attribute(self.handle(), "YMAX\0", &y_max.to_string());
        self
    }

    fn line_x(&self) -> f32 {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "LINEX\0");
            s.parse().expect("could not convert LINEX to a number")
        }
    }

    fn set_line_x(&self, linex: f32) -> &Self {
        set_str_attribute(self.handle(), "LINEX\0", &linex.to_string());
        self
    }

    fn line_y(&self) -> f32 {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "LINEY\0");
            s.parse().expect("could not convert LINEY to a number")
        }
    }

    fn set_line_y(&self, line_y: f32) -> &Self {
        set_str_attribute(self.handle(), "LINEY\0", &line_y.to_string());
        self
    }

    // TODO: XAUTOHIDE, YAUTOHIDE, XHIDDEN, YHIDDEN, and others for ScrollbarAttribute
}

pub trait CanFocusAttribute : Control {
    fn can_focus(&self) -> bool {
        unsafe {
            get_str_attribute_slice(self.handle(), "CANFOCUS\0") == "YES"
        }
    }

    fn set_can_focus(&self, can_focus: bool) -> &Self {
        set_str_attribute(self.handle(), "CANFOCUS\0", if can_focus { "YES\0" } else { "NO\0" });
        self
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Cursor {
    // Loading cursors from application resources is purposefully not supported, as doing that
    // is more platform-specific and is unnecessary when you can create and use an `Image`.
    None,
    Arrow,
    Busy,
    Cross,
    Hand,
    Help,
    Move,
    //Pen,
    ResizeN,
    ResizeS,
    ResizeNS,
    ResizeW,
    ResizeE,
    ResizeWE,
    ResizeNE,
    ResizeSW,
    ResizeNW,
    ResizeSE,
    Text,
    /// Windows only
    AppStarting,
    /// Windows only
    No,
    UpArrow,
    // TODO: once images are wrapped, be able to pass one into set_cursor
    // set_cursor would assign it a random name, and assign that name to the CURSOR attribute
    // It also must add_extra_ref to the image
    //Image(&Image),
    Image,
}

impl Cursor {
    fn from_str(s: &str) -> Cursor {
        match s {
            "NONE" => Cursor::None,
            "ARROW" => Cursor::Arrow,
            "BUSY" => Cursor::Busy,
            "CROSS" => Cursor::Cross,
            "HAND" => Cursor::Hand,
            "HELP" => Cursor::Help,
            "MOVE" => Cursor::Move,
            "RESIZE_N" => Cursor::ResizeN,
            "RESIZE_S" => Cursor::ResizeS,
            "RESIZE_NS" => Cursor::ResizeNS,
            "RESIZE_W" => Cursor::ResizeW,
            "RESIZE_E" => Cursor::ResizeE,
            "RESIZE_WE" => Cursor::ResizeWE,
            "RESIZE_NE" => Cursor::ResizeNE,
            "RESIZE_SW" => Cursor::ResizeSW,
            "RESIZE_NW" => Cursor::ResizeNW,
            "RESIZE_SE" => Cursor::ResizeSE,
            "TEXT" => Cursor::Text,
            "APPSTARTING" => Cursor::AppStarting,
            "NO" => Cursor::No,
            "UPARROW" => Cursor::UpArrow,
            _ => {
                unimplemented!(); // TODO: Image
            },
        }
    }

    fn to_str(self) -> Cow<'static, str> {
        match self {
            Cursor::None => "NONE\0".into(),
            Cursor::Arrow => "ARROW\0".into(),
            Cursor::Busy => "BUSY\0".into(),
            Cursor::Cross => "CROSS\0".into(),
            Cursor::Hand => "HAND\0".into(),
            Cursor::Help => "HELP\0".into(),
            Cursor::Move => "MOVE\0".into(),
            Cursor::ResizeN => "RESIZE_N\0".into(),
            Cursor::ResizeS => "RESIZE_S\0".into(),
            Cursor::ResizeNS => "RESIZE_NS\0".into(),
            Cursor::ResizeW => "RESIZE_W\0".into(),
            Cursor::ResizeE => "RESIZE_E\0".into(),
            Cursor::ResizeWE => "RESIZE_WE\0".into(),
            Cursor::ResizeNE => "RESIZE_NE\0".into(),
            Cursor::ResizeSW => "RESIZE_SW\0".into(),
            Cursor::ResizeNW => "RESIZE_NW\0".into(),
            Cursor::ResizeSE => "RESIZE_SE\0".into(),
            Cursor::Text => "TEXT\0".into(),
            Cursor::AppStarting => "APPSTARTING\0".into(),
            Cursor::No => "NO\0".into(),
            Cursor::UpArrow => "UPARROW\0".into(),
            Cursor::Image => {
                unsafe {
                    let img: *mut Ihandle = ::std::ptr::null_mut(); // TODO:
                    let curr_name = IupGetName(img);
                    if !curr_name.is_null() {
                        CStr::from_ptr(curr_name).to_string_lossy().into_owned().into()
                    } else {
                        let new_name = get_unique_attribute_name();
                        set_handle(&new_name, img);
                        new_name.into()
                    }
                }
            },
        }
    }
}

pub trait CursorAttribute : Control {
    fn cursor(&self) -> Cursor {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "CURSOR\0");
            Cursor::from_str(&s)
        }
    }

    fn set_cursor(&self, cursor: Cursor) -> &Self {
        let s = cursor.to_str();
        set_str_attribute(self.handle(), "CURSOR\0", &s);
        self
    }
}

pub trait MinMaxSizeAttribute : Control {
    fn min_size(&self) -> (i32, i32) {
        get_int_int_attribute(self.handle(), "MINSIZE")
    }

    fn set_min_size(&self, x: i32, y: i32) {
        let s = format!("{}x{}", x, y);
        set_str_attribute(self.handle(), "MINSIZE", &s);
    }

    fn max_size(&self) -> (i32, i32) {
        get_int_int_attribute(self.handle(), "MAXSIZE")
    }

    fn set_max_size(&self, x: i32, y: i32) {
        let s = format!("{}x{}", x, y);
        set_str_attribute(self.handle(), "MAXSIZE", &s);
    }
}

pub trait OrientationAttribute : Control {
    fn orientation(&self) -> ::Orientation {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "ORIENTATION");
            ::Orientation::from_str(s.as_bytes())
        }
    }

    fn set_orientation(&self, orientation: ::Orientation) {
        set_str_attribute(self.handle(), "ORIENTATION", orientation.to_str());
    }
}

pub enum Scrollbar {
    Vertical,
    Horizontal,
    Both,
    None,
}

impl Scrollbar {
    fn from_str(s: &str) -> Scrollbar {
        match s {
            "VERTICAL" => Scrollbar::Vertical,
            "HORIZONTAL" => Scrollbar::Horizontal,
            "YES" => Scrollbar::Both,
            "NO" => Scrollbar::None,
            _ => panic!("unknown Scrollbar"),
        }
    }

    fn to_str(self) -> &'static str {
        match self {
            Scrollbar::Vertical => "VERTICAL\0",
            Scrollbar::Horizontal => "HORIZONTAL\0",
            Scrollbar::Both => "YES\0",
            Scrollbar::None => "NO\0",
        }
    }
}

pub trait ScrollbarAttribute : Control {
    fn scrollbar(&self) -> Scrollbar {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "SCROLLBAR\0");
            Scrollbar::from_str(&s)
        }
    }

    fn set_scrollbar(&self, scrollbar: Scrollbar) -> &Self {
        set_str_attribute(self.handle(), "SCROLLBAR\0", scrollbar.to_str());
        self
    }
}

pub trait TipAttribute : Control {
    fn tip(&self) -> String {
        get_str_attribute(self.handle(), "TIP")
    }
    unsafe fn tip_slice(&self) -> Cow<str> {
        get_str_attribute_slice(self.handle(), "TIP")
    }

    fn set_tip(&self, tip: &str) {
        set_str_attribute(self.handle(), "TIP", tip);
    }
}

pub trait TitleAttribute : Control {
    fn title(&self) -> String {
        get_str_attribute(self.handle(), "TITLE")
    }

    fn set_title(&self, title: &str) {
        set_str_attribute(self.handle(), "TITLE", title);
    }
}

pub trait VisibleAttribute : Control {
    fn show(&self) -> Result<(), ()> {
        unsafe {
            if IupShow(self.handle()) == IUP_NOERROR {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    fn hide(&self) -> Result<(), ()> {
        unsafe {
            if IupHide(self.handle()) == IUP_NOERROR {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    fn visible(&self) -> bool {
        unsafe {
            get_str_attribute_slice(self.handle(), "VISIBLE\0") == "YES"
        }
    }

    fn set_visible(&self, visible: bool) -> Result<(), ()> {
        if visible { self.show() } else { self.hide() }
    }
}
