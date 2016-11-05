/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use super::control_prelude::*;
use std::borrow::Cow;
use std::ffi::CStr;
use std::path::{PathBuf, Path};
use super::Popup;

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
            FileDialogType::Open => "OPEN\0",
            FileDialogType::Save => "SAVE\0",
            FileDialogType::Dir => "DIR\0",
        }
    }
}

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
            let val = get_str_attribute_slice(self.handle(), "DIALOGTYPE\0");
            FileDialogType::from_str(&*val)
        }
    }

    pub fn set_dialog_type(&self, ty: FileDialogType) {
        set_str_attribute(self.handle(), "DIALOGTYPE\0", ty.to_str());
    }

    pub fn directory(&self) -> String {
        get_str_attribute(self.handle(), "DIRECTORY\0")
    }

    pub fn set_directory(&self, s: &str) {
        set_str_attribute(self.handle(), "DIRECTORY\0", s);
    }

    pub fn ext_filter(&self) -> Vec<FileExtFilter> {
        unsafe {
            let val = get_str_attribute_slice(self.handle(), "EXTFILTER\0");
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

    /// Sets file filters. It is recommended to always include an "All Files" filter.
    ///
    /// # Examples
    /// ```
    /// # use clear_coat::{FileDlg, FileExtFilter};
    /// let f = FileDlg::new();
    /// f.set_ext_filter(&[
    ///     FileExtFilter::from_borrowed("All Files",
    ///                                  &["*.*".into()]),
    ///     FileExtFilter::from_borrowed("All Images",
    ///                                  &["*.jpg".into(), "*.jpeg".into(), "*.png".into()]),
    ///     FileExtFilter::from_borrowed("JPEG Images",
    ///                                  &["*.jpg".into(), "*.jpeg".into()]),
    ///     FileExtFilter::from_borrowed("PNG Images",
    ///                                  &["*.png".into()]),
    /// ]);
    /// f.set_filter_used(1);
    /// ```
    pub fn set_ext_filter(&self, ext_filter: &[FileExtFilter]) {
        let mut s = String::with_capacity(ext_filter.len() * 25);
        for f in ext_filter.iter() {
            let semi_joined_filter = f.filter.join(";");
            s.push_str(&f.description);
            s.push_str(" (");
            s.push_str(&semi_joined_filter);
            s.push_str(")");
            s.push_str("|");

            s.push_str(&semi_joined_filter);
            s.push_str("|");
        }
        s.push_str("\0");
        set_str_attribute(self.handle(), "EXTFILTER\0", &s);
    }

    /// Sets the index of the filter to use starting at 0.
    pub fn set_filter_used(&self, i: u32) -> &Self {
        set_str_attribute(self.handle(), "FILTERUSED\0", &format!("{}\0", i + 1));
        self
    }

    /// Gets the index of the filter to use. It returns the selection made by the user.
    pub fn filter_used(&self) -> u32 {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "FILTERUSED\0");
            s.parse::<u32>().expect("could not convert FILTERUSED to an integer") - 1
        }
    }

    pub fn multiple_files(&self) -> bool {
        unsafe {
            get_str_attribute_slice(self.handle(), "MULTIPLEFILES\0") == "YES"
        }
    }

    pub fn set_multiple_files(&self, multiple: bool) -> &Self {
        set_str_attribute(self.handle(), "MULTIPLEFILES\0", if multiple { "YES\0" } else { "NO\0" });
        self
    }

    pub fn value_single(&self) -> Option<PathBuf> {
        assert!(!self.multiple_files());
        unsafe {
            let val = get_attribute_ptr(self.handle(), "VALUE\0");
            if val.is_null() {
                None
            } else {
                Some(PathBuf::from(&*CStr::from_ptr(val).to_string_lossy()))
            }
        }
    }

    pub fn value_multiple(&self) -> Option<Vec<PathBuf>> {
        assert!(self.multiple_files());
        unsafe {
            let val = get_attribute_ptr(self.handle(), "VALUE\0");
            if val.is_null() {
                None
            } else {
                const PIPE: &'static [char] = &['|'];
                let val = CStr::from_ptr(val).to_string_lossy();
                let mut parts = val.split(PIPE);
                let last_part = parts.next_back().expect("failed removing last part");
                // if multiple files were selected, the string will end in a pipe
                if last_part.is_empty() {
                    let dir = parts.next().expect("failed to get directory in value in \
                                                FileDlg when multiple_files == true");
                    Some(parts.map(|p| Path::new(dir).join(p)).collect())
                } else {
                    assert_eq!(parts.next(), None);
                    Some(vec![PathBuf::from(last_part)])
                }
            }
        }
    }
}

impl_control_traits!(FileDlg);

impl Popup for FileDlg {}

impl TitleAttribute for FileDlg {}