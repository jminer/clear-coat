/* Copyright 2016 Jordan Miner
 *
 * Licensed under the MIT license <LICENSE or
 * http://opensource.org/licenses/MIT>. This file may not be copied,
 * modified, or distributed except according to those terms.
 */

use super::control_prelude::*;
use super::containers::{
    Container,
    NonDialogContainer,
};

#[derive(Clone)]
pub struct TabInfo<'a, 'b> {
    title: &'a str,
    control: &'b Control,
}

impl<'a, 'b> TabInfo<'a, 'b> {
    pub fn new(control: &'b Control) -> Self {
        TabInfo {
            title: "",
            control: control,
        }
    }

    pub fn title(&mut self, title: &'a str) -> &mut Self {
        self.title = title;
        self
    }
}

/// # Examples
///
/// Using the `append_tabs` helper:
///
/// ```
/// # use clear_coat::*;
/// let tabs = Tabs::new();
/// tabs.append_tabs(&[
///     TabInfo::new(&Text::new()).title("First"),
///     TabInfo::new(&Text::new()).title("Second"),
/// ]);
/// ```
///
/// or the more verbose way:
///
/// ```
/// # use clear_coat::*;
/// let tabs = Tabs::new();
/// tabs.append(&Text::new());
/// tabs.set_tab_title(0, "First");
/// tabs.append(&Text::new());
/// tabs.set_tab_title(1, "Second");
/// ```
#[derive(Clone)]
pub struct Tabs(HandleRc);

impl Tabs {
    pub fn new() -> Self {
        unsafe {
            ::iup_open();
            let ih = IupTabsv(ptr::null_mut());
            Tabs(HandleRc::new(ih))
        }
    }

    /// A `pos` of 0 is the first tab.
    pub fn tab_title(&self, pos: usize) -> String {
        get_str_attribute(self.handle(), &format!("TABTITLE{}\0", pos))
    }

    /// A `pos` of 0 is the first tab.
    pub fn set_tab_title(&self, pos: usize, title: &str) -> &Self {
        set_str_attribute(self.handle(), &format!("TABTITLE{}\0", pos), title);
        self
    }

    pub fn append_tabs(&self, info: &[&TabInfo]) -> &Self {
        let mut index = self.child_count();
        for ti in info {
            self.append(ti.control).expect("failed to append tab");
            self.set_tab_title(index, ti.title);
            index += 1;
        }
        self
    }
}

impl_control_traits!(Tabs);

impl Container for Tabs {}
impl NonDialogContainer for Tabs {}

impl ActiveAttribute for Tabs {}
impl ExpandAttribute for Tabs {}
impl MinMaxSizeAttribute for Tabs {}
impl TipAttribute for Tabs {}
impl VisibleAttribute for Tabs {}

impl MenuCommonCallbacks for Tabs {}
impl GetKillFocusCallbacks for Tabs {}
impl EnterLeaveWindowCallbacks for Tabs {}

