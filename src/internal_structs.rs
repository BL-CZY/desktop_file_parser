use std::collections::HashMap;

use crate::{DesktopAction, DesktopEntry, IconString, LocaleString, LocaleStringList};

#[derive(Debug, Clone, Default)]
#[doc(hidden)]
pub enum EntryTypeInternal {
    #[default]
    Application,
    Link,
    Directory,
    Unknown(String),
}

impl From<&str> for EntryTypeInternal {
    fn from(value: &str) -> Self {
        match value {
            "Application" => Self::Application,
            "Link" => Self::Link,
            "Directory" => Self::Directory,
            _ => Self::Unknown(value.into()),
        }
    }
}

#[derive(Debug, Clone, Default)]
#[doc(hidden)]
pub struct LocaleStringInternal {
    pub default: Option<String>, // required
    pub variants: HashMap<String, String>,
}

#[derive(Debug, Clone, Default)]
#[doc(hidden)]
pub struct LocaleStringListInternal {
    pub default: Option<Vec<String>>,
    pub variants: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Default)]
#[doc(hidden)]
pub struct DesktopEntryInternal {
    /// This specification defines 3 types of desktop entries: Application (type 1), Link (type 2) and Directory (type 3). To allow the addition of new types in the future, implementations should ignore desktop entries with an unknown type.
    pub entry_type: Option<EntryTypeInternal>, // required
    /// Version of the Desktop Entry Specification that the desktop entry conforms with. Entries that confirm with this version of the specification should use 1.5. Note that the version field is not required to be present.
    pub version: Option<String>,
    /// Specific name of the application, for example "Mozilla".
    pub name: Option<LocaleStringInternal>, // required
    /// Generic name of the application, for example "Web Browser".
    pub generic_name: Option<LocaleStringInternal>,
    /// NoDisplay means "this application exists, but don't display it in the menus". This can be useful to e.g. associate this application with MIME types, so that it gets launched from a file manager (or other apps), without having a menu entry for it (there are tons of good reasons for this, including e.g. the netscape -remote, or kfmclient openURL kind of stuff).
    pub no_display: Option<bool>,
    /// Tooltip for the entry, for example "View sites on the Internet". The value should not be redundant with the values of Name and GenericName.
    pub comment: Option<LocaleStringInternal>,
    /// Icon to display in file manager, menus, etc. If the name is an absolute path, the given file will be used. If the name is not an absolute path, the algorithm described in the Icon Theme Specification will be used to locate the icon.
    pub icon: Option<IconString>,
    /// Hidden should have been called Deleted. It means the user deleted (at their level) something that was present (at an upper level, e.g. in the system dirs). It's strictly equivalent to the .desktop file not existing at all, as far as that user is concerned. This can also be used to "uninstall" existing files (e.g. due to a renaming) - by letting make install install a file with Hidden=true in it.
    pub hidden: Option<bool>,
    /// A list of strings identifying the desktop environments that should display/not display a given desktop entry.
    /// By default, a desktop file should be shown, unless an OnlyShowIn key is present, in which case, the default is for the file not to be shown.
    /// If $XDG_CURRENT_DESKTOP is set then it contains a colon-separated list of strings. In order, each string is considered. If a matching entry is found in OnlyShowIn then the desktop file is shown. If an entry is found in NotShowIn then the desktop file is not shown. If none of the strings match then the default action is taken (as above).
    /// $XDG_CURRENT_DESKTOP should have been set by the login manager, according to the value of the DesktopNames found in the session file. The entry in the session file has multiple values separated in the usual way: with a semicolon.
    /// The same desktop name may not appear in both OnlyShowIn and NotShowIn of a group.
    pub only_show_in: Option<Vec<String>>,
    /// A list of strings identifying the desktop environments that should display/not display a given desktop entry.
    /// By default, a desktop file should be shown, unless an OnlyShowIn key is present, in which case, the default is for the file not to be shown.
    /// If $XDG_CURRENT_DESKTOP is set then it contains a colon-separated list of strings. In order, each string is considered. If a matching entry is found in OnlyShowIn then the desktop file is shown. If an entry is found in NotShowIn then the desktop file is not shown. If none of the strings match then the default action is taken (as above).
    /// $XDG_CURRENT_DESKTOP should have been set by the login manager, according to the value of the DesktopNames found in the session file. The entry in the session file has multiple values separated in the usual way: with a semicolon.
    /// The same desktop name may not appear in both OnlyShowIn and NotShowIn of a group.
    pub not_show_in: Option<Vec<String>>,
    /// A boolean value specifying if D-Bus activation is supported for this application. If this key is missing, the default value is false. If the value is true then implementations should ignore the Exec key and send a D-Bus message to launch the application. See D-Bus Activation for more information on how this works. Applications should still include Exec= lines in their desktop files for compatibility with implementations that do not understand the DBusActivatable key.
    pub dbus_activatable: Option<bool>,
    /// Path to an executable file on disk used to determine if the program is actually installed. If the path is not an absolute path, the file is looked up in the $PATH environment variable. If the file is not present or if it is not executable, the entry may be ignored (not be used in menus, for example).
    pub try_exec: Option<String>,
    /// Program to execute, possibly with arguments. See the Exec key for details on how this key works. The Exec key is required if DBusActivatable is not set to true. Even if DBusActivatable is true, Exec should be specified for compatibility with implementations that do not understand DBusActivatable.
    pub exec: Option<String>,
    /// If entry is of type Application, the working directory to run the program in.
    pub path: Option<String>,
    /// Whether the program runs in a terminal window.
    pub terminal: Option<bool>,
    /// Identifiers for application actions. This can be used to tell the application to make a specific action, different from the default behavior. The Application actions section describes how actions work.
    pub actions: Option<Vec<String>>,
    /// The MIME type(s) supported by this application.
    pub mime_type: Option<Vec<String>>,
    /// Categories in which the entry should be shown in a menu (for possible values see the Desktop Menu Specification).
    pub categories: Option<Vec<String>>,
    /// A list of interfaces that this application implements. By default, a desktop file implements no interfaces. See Interfaces for more information on how this works.
    pub implements: Option<Vec<String>>,
    /// A list of strings which may be used in addition to other metadata to describe this entry. This can be useful e.g. to facilitate searching through entries. The values are not meant for display, and should not be redundant with the values of Name or GenericName.
    pub keywords: Option<LocaleStringListInternal>,
    /// If true, it is KNOWN that the application will send a "remove" message when started with the DESKTOP_STARTUP_ID environment variable set. If false, it is KNOWN that the application does not work with startup notification at all (does not shown any window, breaks even when using StartupWMClass, etc.). If absent, a reasonable handling is up to implementations (assuming false, using StartupWMClass, etc.). (See the [Startup Notification Protocol Specification](https://www.freedesktop.org/wiki/Specifications/startup-notification-spec/) for more details).
    pub startup_notify: Option<bool>,
    /// If specified, it is known that the application will map at least one window with the given string as its WM class or WM name hint (see the [Startup Notification Protocol Specification](https://www.freedesktop.org/wiki/Specifications/startup-notification-spec/) for more details).
    pub startup_wm_class: Option<String>,
    /// If entry is Link type, the URL to access. Required if entry_type is link
    pub url: Option<String>,
    /// If true, the application prefers to be run on a more powerful discrete GPU if available, which we describe as “a GPU other than the default one” in this spec to avoid the need to define what a discrete GPU is and in which cases it might be considered more powerful than the default GPU. This key is only a hint and support might not be present depending on the implementation.
    pub prefers_non_default_gpu: Option<bool>,
    /// If true, the application has a single main window, and does not support having an additional one opened. This key is used to signal to the implementation to avoid offering a UI to launch another window of the app. This key is only a hint and support might not be present depending on the implementation.
    pub single_main_window: Option<bool>,
}

#[derive(Default, Clone, Debug)]
#[doc(hidden)]
pub struct DesktopActionInternal {
    pub ref_name: String,
    pub name: Option<LocaleStringInternal>, // required
    pub exec: Option<String>,
    pub icon: Option<IconString>,
}

impl Into<LocaleString> for LocaleStringInternal {
    fn into(self) -> LocaleString {
        LocaleString {
            default: self.default.unwrap(),
            variants: self.variants,
        }
    }
}

impl Into<LocaleStringList> for LocaleStringListInternal {
    fn into(self) -> LocaleStringList {
        LocaleStringList {
            default: self.default.unwrap(),
            variants: self.variants,
        }
    }
}

impl Into<DesktopAction> for DesktopActionInternal {
    fn into(self) -> DesktopAction {
        DesktopAction {
            ref_name: self.ref_name,
            name: self.name.unwrap().into(),
            exec: self.exec,
            icon: self.icon,
        }
    }
}

impl Into<DesktopEntry> for DesktopEntryInternal {
    fn into(self) -> DesktopEntry {
        DesktopEntry {
            entry_type: self.entry_type.unwrap(),
            version: self.version,
            name: self.name.unwrap().into(),
            generic_name: match self.generic_name {
                Some(l) => Some(l.into()),
                None => None,
            },
            no_display: self.no_display,
            comment: match self.comment {
                Some(l) => Some(l.into()),
                None => None,
            },
            icon: self.icon,
            hidden: self.hidden,
            only_show_in: self.only_show_in,
            not_show_in: self.not_show_in,
            dbus_activatable: self.dbus_activatable,
            try_exec: self.try_exec,
            exec: self.exec,
            path: self.path,
            terminal: self.terminal,
            actions: self.actions,
            mime_type: self.mime_type,
            categories: self.categories,
            implements: self.implements,
            keywords: match self.keywords {
                Some(l) => Some(l.into()),
                None => None,
            },
            startup_notify: self.startup_notify,
            startup_wm_class: self.startup_wm_class,
            url: self.url,
            prefers_non_default_gpu: self.prefers_non_default_gpu,
            single_main_window: self.single_main_window,
        }
    }
}
