/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use std::borrow::Cow;
use iup_sys::*;
use super::{
    Control,
    UnwrapHandle,
    ScreenPosition,
    Popup,
};
use super::attributes::{
    TitleAttribute,
    get_str_attribute,
    get_str_attribute_slice,
    set_str_attribute,
};
use super::handle_rc::HandleRc;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FileDialogType {
    Open,
    Save,
    Dir
}

impl FileDialogType {
    fn from_str(s: &str) -> FileDialogType {
        match s {
            "OPEN" => FileDialogType::Open,
            "SAVE" => FileDialogType::Save,
            "DIR" => FileDialogType::Dir,
            _ => panic!("unknown FileDialogType string"),
        }
    }

    fn to_str(&self) -> &'static str {
        match *self {
            FileDialogType::Open => "OPEN",
            FileDialogType::Save => "SAVE",
            FileDialogType::Dir => "DIR",
        }
    }
}

/*
set_ext_filter(&[
    FileExtFilter::from_borrowed("JPEG Images",
                                 ["*.jpg".into(), "*.jpeg".into()]),
    FileExtFilter::from_borrowed("JPEG Images",
                                 ["*.jpg".into(), "*.jpeg".into()]),
])

 */
pub struct FileExtFilter<'a, 'b, 'c: 'b> {
    pub description: Cow<'a, str>,
    pub filter: Cow<'b, [Cow<'c, str>]>,
}

impl<'a, 'b, 'c> FileExtFilter<'a, 'b, 'c> {
    pub fn from_borrowed(desc: &'a str, filter: &'b [Cow<'c, str>]) -> FileExtFilter<'a, 'b, 'c> {
        FileExtFilter {
            description: Cow::Borrowed(desc),
            filter: Cow::Borrowed(filter),
        }
    }

    pub fn from_owned(desc: String, filter: Vec<String>) -> FileExtFilter<'a, 'b, 'c> {
        FileExtFilter {
            description: Cow::Owned(desc),
            filter: Cow::Owned(filter.into_iter().map(|s| Cow::Owned(s)).collect()),
        }
    }
}

#[derive(Clone)]
pub struct FileDlg(HandleRc);

impl FileDlg {
    pub fn new() -> FileDlg {
        unsafe {
            ::iup_open();
            let ih = IupFileDlg();
            FileDlg(HandleRc::new(ih))
        }
    }

    pub fn dialog_type(&self) -> FileDialogType {
        unsafe {
            let val = get_str_attribute_slice(self.handle(), "DIALOGTYPE");
            FileDialogType::from_str(&*val)
        }
    }

    pub fn set_dialog_type(&self, ty: FileDialogType) {
        set_str_attribute(self.handle(), "DIALOGTYPE", ty.to_str());
    }

    pub fn directory(&self) -> String {
        get_str_attribute(self.handle(), "DIRECTORY")
    }

    pub fn set_directory(&self, s: &str) {
        set_str_attribute(self.handle(), "DIRECTORY", s);
    }

    pub fn ext_filter(&self) -> Vec<FileExtFilter> {
        unsafe {
            let val = get_str_attribute_slice(self.handle(), "EXTFILTER");
            let mut filters = vec![];
            let mut parts = val.split('|');
            loop {
                let desc = match parts.next() {
                    Some(p) => p,
                    None => break,
                };
                let filter = match parts.next() {
                    Some(p) => p,
                    None => break,
                };
                let filter = filter.split(';').map(|s| Cow::Owned(s.to_owned())).collect();
                filters.push(FileExtFilter {
                    description: Cow::Owned(desc.to_owned()),
                    filter: Cow::Owned(filter),
                });
            }
            filters
        }
    }

    pub fn set_ext_filter(&self, ext_filter: &[FileExtFilter]) {
        let s = ext_filter.iter().map(|f|
            format!("{}|{}", f.description, f.filter.join(";"))
        ).fold(String::new(), |mut s, f| {
            if s.is_empty() {
                s.push('|');
            }
            s.push_str(&f);
            s
        });
        set_str_attribute(self.handle(), "EXTFILTER", &s);
    }
}

impl_control_traits!(FileDlg);

impl Popup for FileDlg {}

impl TitleAttribute for FileDlg {}