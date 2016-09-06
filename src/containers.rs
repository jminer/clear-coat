/* Copyright 2015 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use super::control_prelude::*;

pub trait Container : Control {
    /// Warning: Since children are stored as a linked list, appending a control is O(n) where
    /// n is the number of children.
    fn append(&self, new_child: &Control) -> Result<(), ()> {
        unsafe {
            if IupAppend(self.handle(), new_child.handle()) == ptr::null_mut() {
                Err(())
            } else {
                Ok(())
            }
        }
    }

    /// Warning: Since children are stored as a linked list, inserting a control is O(n) where
    /// n is the number of children before `ref_child`.
    fn insert(&self, ref_child: Option<&Control>, new_child: &Control) -> Result<(), ()> {
        unsafe {
            let ref_child = ref_child.map(|c| c.handle()).unwrap_or(ptr::null_mut());
            if IupInsert(self.handle(), ref_child, new_child.handle()) == ptr::null_mut() {
                Err(())
            } else {
                Ok(())
            }
        }
    }

    /// Returns the number of children of the specified control.
    ///
    /// Warning: Since children are stored as a linked list, getting the count is O(n) where
    /// n is the number of children.
    fn child_count(&self) -> usize {
        unsafe {
            IupGetChildCount(self.handle()) as usize
        }
    }
}

pub trait NonDialogContainer : Container {
    fn refresh_children(&self) {
        unsafe {
            IupRefreshChildren(self.handle());
        }
    }
}

const DEFAULT_GAP: &'static str = "6\0";

fn set_top_level_margin_and_gap(ih: *mut Ihandle) {
    set_str_attribute(ih, "NMARGIN\0", "6x6\0");
    set_str_attribute(ih, "GAP\0", DEFAULT_GAP);
}


pub fn wrapper_to_handle_vec<T: ::Control + ?Sized>(controls: &[&T]) -> Vec<*mut Ihandle>
{
    let mut v: Vec<*mut Ihandle> = controls.iter().map(|c| c.handle()).collect();
    v.push(ptr::null_mut()); // array has to be null terminated
    v
}


#[derive(Clone)]
pub struct Fill(HandleRc);

impl Fill {
    pub fn new() -> Fill {
        unsafe {
            ::iup_open();
            let handle = IupFill();
            Fill(HandleRc::new(handle))
        }
    }
}

impl_control_traits!(Fill);

#[macro_export]
macro_rules! fill { // This is a macro for consistency, even though it could just be a function.
    () => { Fill::new() };
}


#[derive(Clone)]
pub struct Hbox(HandleRc);

impl Hbox {
    pub fn new() -> Self {
        unsafe {
            ::iup_open();
            let handle = IupHboxv(ptr::null_mut());
            Hbox(HandleRc::new(handle))
        }
    }

    pub fn with_children(children: &[&::Control]) -> Self {
        unsafe {
            // got to already be IupOpen()ed
            let mut handles = wrapper_to_handle_vec(children);
            Hbox::from_handles(handles.as_mut_ptr())
        }
    }

    pub unsafe fn from_handles(children: *mut *mut Ihandle) -> Hbox {
        let handle = IupHboxv(children);
        Hbox(HandleRc::new(handle))
    }

    pub fn set_top_level_margin_and_gap(&self) -> &Self {
        set_top_level_margin_and_gap(self.handle());
        self
    }
}

impl_control_traits!(Hbox);

impl Container for Hbox {}
impl NonDialogContainer for Hbox {}

#[macro_export]
macro_rules! hbox {
    ($($c:expr),*) => {
        {
            use std::ptr;
            let mut handles = Vec::new();
            $(
                // The control has to be stored in a binding to ensure it isn't dropped before
                // it is added as a child of the container. (Otherwise, the control is destroyed.)
                let c = $c;
                handles.push(c.handle());
            )*
            handles.push(ptr::null_mut());
            unsafe { Hbox::from_handles(handles.as_mut_ptr()) }
        }
    };
    ($($c:expr,)*) => { hbox!($($c),*) };
}


#[derive(Clone)]
pub struct Vbox(HandleRc);

impl Vbox {
    pub fn new() -> Self {
        unsafe {
            ::iup_open();
            let handle = IupVboxv(ptr::null_mut());
            Vbox(HandleRc::new(handle))
        }
    }

    pub fn with_children(children: &[&::Control]) -> Self {
        unsafe {
            // got to already be IupOpen()ed
            let mut handles = wrapper_to_handle_vec(children);
            Vbox::from_handles(handles.as_mut_ptr())
        }
    }

    pub unsafe fn from_handles(children: *mut *mut Ihandle) -> Vbox {
        let handle = IupVboxv(children);
        Vbox(HandleRc::new(handle))
    }

    pub fn set_top_level_margin_and_gap(&self) -> &Self {
        set_top_level_margin_and_gap(self.handle());
        self
    }
}

impl_control_traits!(Vbox);

impl Container for Vbox {}
impl NonDialogContainer for Vbox {}

#[macro_export]
macro_rules! vbox {
    ($($c:expr),*) => {
        {
            use std::ptr;
            let mut handles = Vec::new();
            $(
                // The control has to be stored in a binding to ensure it isn't dropped before
                // it is added as a child of the container. (Otherwise, the control is destroyed.)
                let c = $c;
                handles.push(c.handle());
            )*
            handles.push(ptr::null_mut());
            unsafe { Vbox::from_handles(handles.as_mut_ptr()) }
        }
    };
    ($($c:expr,)*) => { vbox!($($c),*) };
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NumDiv {
    Fixed(u32),
    Auto,
}

#[derive(Clone)]
pub struct GridBox(HandleRc);

impl GridBox {
    pub fn new() -> Self {
        unsafe {
            ::iup_open();
            let handle = IupGridBoxv(ptr::null_mut());
            GridBox(HandleRc::new(handle))
        }
    }

    pub fn with_children(children: &[&::Control]) -> Self {
        unsafe {
            // got to already be IupOpen()ed
            let mut handles = wrapper_to_handle_vec(children);
            GridBox::from_handles(handles.as_mut_ptr())
        }
    }

    pub unsafe fn from_handles(children: *mut *mut Ihandle) -> GridBox {
        let handle = IupGridBoxv(children);
        GridBox(HandleRc::new(handle))
    }

    pub fn set_top_level_margin_and_gap(&self) -> &Self {
        set_str_attribute(self.handle(), "GAPLIN\0", DEFAULT_GAP);
        set_str_attribute(self.handle(), "GAPCOL\0", DEFAULT_GAP);
        self
    }

    pub fn alignment_lin(&self, line: u32) -> ::VAlignment {
        unsafe {
            let attr = format!("ALIGNMENTLIN{}", line);
            ::VAlignment::from_str(get_str_attribute_slice(self.handle(), &attr).as_bytes())
        }
    }

    pub fn set_alignment_lin(&self, line: u32, alignment: ::VAlignment) -> &Self {
        set_str_attribute(self.handle(), &format!("ALIGNMENTLIN{}", line), alignment.to_str());
        self
    }

    pub fn alignment_lin_all(&self) -> ::VAlignment {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "ALIGNMENTLIN");
            ::VAlignment::from_str(s.as_bytes())
        }
    }

    pub fn set_alignment_lin_all(&self, alignment: ::VAlignment) -> &Self {
        set_str_attribute(self.handle(), "ALIGNMENTLIN", alignment.to_str());
        self
    }

    pub fn alignment_col(&self, column: u32) -> ::HAlignment {
        unsafe {
            let attr = format!("ALIGNMENTCOL{}", column);
            ::HAlignment::from_str(get_str_attribute_slice(self.handle(), &attr).as_bytes())
        }
    }

    pub fn set_alignment_col(&self, column: u32, alignment: ::HAlignment) -> &Self {
        set_str_attribute(self.handle(), &format!("ALIGNMENTCOL{}", column), alignment.to_str());
        self
    }

    pub fn alignment_col_all(&self) -> ::HAlignment {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "ALIGNMENTCOL");
            ::HAlignment::from_str(s.as_bytes())
        }
    }

    pub fn set_alignment_col_all(&self, alignment: ::HAlignment) -> &Self {
        set_str_attribute(self.handle(), "ALIGNMENTCOL", alignment.to_str());
        self
    }

    pub fn num_div(&self) -> NumDiv {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "NUMDIV");
            if s.as_bytes() == b"-1" {
                NumDiv::Auto
            } else {
                NumDiv::Fixed(s.parse().expect("could not convert NUMDIV to an integer"))
            }
        }
    }

    pub fn set_num_div(&self, num: NumDiv) -> &Self {
        match num {
            NumDiv::Fixed(i) => set_str_attribute(self.handle(), "NUMDIV", &i.to_string()),
            NumDiv::Auto => set_str_attribute(self.handle(), "NUMDIV", "AUTO"),
        };
        self
    }

    pub fn num_lin(&self) -> u32 {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "NUMLIN");
            s.parse().expect("could not convert NUMLIN to an integer")
        }
    }

    pub fn num_col(&self) -> u32 {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "NUMCOL");
            s.parse().expect("could not convert NUMLIN to an integer")
        }
    }

    pub fn fit_col_to_children(&self, column: u32) -> &Self {
        set_str_attribute(self.handle(), "FITTOCHILDREN\0", &format!("C{}\0", column));
        self
    }

    pub fn fit_lin_to_children(&self, line: u32) -> &Self {
        set_str_attribute(self.handle(), "FITTOCHILDREN\0", &format!("L{}\0", line));
        self
    }

    pub fn fit_all_to_children(&self) -> &Self {
        for line in 0..self.num_lin() {
            self.fit_lin_to_children(line);
        }
        for column in 0..self.num_col() {
            self.fit_col_to_children(column);
        }
        self
    }

    pub fn size_col(&self) -> u32 {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "SIZECOL\0");
            s.parse().expect("could not convert SIZECOL to an integer")
        }
    }

    pub fn set_size_col(&self, column: u32) -> &Self {
        set_str_attribute(self.handle(), "SIZECOL\0", &format!("{}\0", column));
        self
    }

    pub fn size_lin(&self) -> u32 {
        unsafe {
            let s = get_str_attribute_slice(self.handle(), "SIZELIN\0");
            s.parse().expect("could not convert SIZELIN to an integer")
        }
    }

    pub fn set_size_lin(&self, line: u32) -> &Self {
        set_str_attribute(self.handle(), "SIZELIN\0", &format!("{}\0", line));
        self
    }
}

impl_control_traits!(GridBox);

impl Container for GridBox {}
impl NonDialogContainer for GridBox {}

impl OrientationAttribute for GridBox {}

#[macro_export]
macro_rules! grid_box {
    ($($c:expr),*) => {
        {
            use std::ptr;
            let mut handles = Vec::new();
            $(
                // The control has to be stored in a binding to ensure it isn't dropped before
                // it is added as a child of the container. (Otherwise, the control is destroyed.)
                let c = $c;
                handles.push(c.handle());
            )*
            handles.push(ptr::null_mut());
            unsafe { GridBox::from_handles(handles.as_mut_ptr()) }
        }
    };
    ($($c:expr,)*) => { grid_box!($($c),*) };
}
