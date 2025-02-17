pub mod internal_structs;
pub mod parser;
pub mod structs;

pub use parser::parse;
pub use structs::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_valid_entry() {
        let content = r#"
[Desktop Entry]
Name=Firefox
Exec=firefox %U
Type=Application
Categories=Network;WebBrowser;
"#;
        let f = parse(content).unwrap();
        let entry = f.entry;

        assert_eq!(entry.name.default, "Firefox");

        match entry.entry_type {
            EntryType::Application(fields) => {
                assert_eq!(fields.exec.unwrap(), "firefox %U");
                assert_eq!(fields.categories.unwrap(), vec!["Network", "WebBrowser"]);
            }
            _ => panic!("Entry type is not Application"),
        }
    }

    #[test]
    fn test_localized_strings() {
        let content = r#"
[Desktop Entry]
Name=Text Editor
Name[es]=Editor de texto
Name[fr]=Éditeur de texte
Name[de]=Texteditor
GenericName=Text Editor
GenericName[es]=Editor
Comment=Edit text files
Comment[fr]=Éditer des fichiers texte
Exec=gedit %F
Type=Application
"#;
        let f = parse(content).unwrap();
        let entry = f.entry;

        let name = entry.name;
        assert_eq!(name.default, "Text Editor");
        assert_eq!(name.variants.get("es").unwrap(), "Editor de texto");
        assert_eq!(name.variants.get("fr").unwrap(), "Éditeur de texte");
        assert_eq!(name.variants.get("de").unwrap(), "Texteditor");

        let generic_name = entry.generic_name.unwrap();
        assert_eq!(generic_name.default, "Text Editor");
        assert_eq!(generic_name.variants.get("es").unwrap(), "Editor");

        let comment = entry.comment.unwrap();
        assert_eq!(comment.default, "Edit text files");
        assert_eq!(
            comment.variants.get("fr").unwrap(),
            "Éditer des fichiers texte"
        );
    }

    #[test]
    fn test_desktop_actions() {
        let content = r#"
[Desktop Entry]
Name=Firefox
Exec=firefox %U
Type=Application
Actions=new-window;new-private-window;

[Desktop Action new-window]
Name=New Window
Name[es]=Nueva ventana
Exec=firefox --new-window
Icon=firefox-new-window

[Desktop Action new-private-window]
Name=New Private Window
Exec=firefox --private-window
"#;
        let f = parse(content).unwrap();
        let actions = f.actions;

        assert_eq!(actions.len(), 2);

        assert_eq!(
            actions.get("new-window").unwrap().name.default,
            "New Window"
        );
        assert_eq!(
            actions
                .get("new-window")
                .unwrap()
                .name
                .variants
                .get("es")
                .unwrap(),
            "Nueva ventana"
        );
        assert_eq!(
            actions.get("new-window").unwrap().exec.as_ref().unwrap(),
            "firefox --new-window"
        );
        assert_eq!(
            actions
                .get("new-window")
                .unwrap()
                .icon
                .as_ref()
                .unwrap()
                .content,
            "firefox-new-window"
        );

        assert_eq!(
            actions.get("new-private-window").unwrap().name.default,
            "New Private Window"
        );
        assert_eq!(
            actions
                .get("new-private-window")
                .unwrap()
                .exec
                .as_ref()
                .unwrap(),
            "firefox --private-window"
        );
        assert!(actions.get("new-private-window").unwrap().icon.is_none());
    }

    #[test]
    fn test_boolean_values() {
        let content = r#"
[Desktop Entry]
Name=Test App
Exec=test
Type=Application
Terminal=true
NoDisplay=false
Hidden=true
DBusActivatable=true
StartupNotify=true
PrefersNonDefaultGPU=true
SingleMainWindow=true
"#;
        let f = parse(content).unwrap();
        let entry = f.entry;

        assert_eq!(entry.no_display.unwrap(), false);
        assert_eq!(entry.hidden.unwrap(), true);
        assert_eq!(entry.dbus_activatable.unwrap(), true);

        match entry.entry_type {
            EntryType::Application(fields) => {
                assert_eq!(fields.terminal.unwrap(), true);
                assert_eq!(fields.startup_notify.unwrap(), true);
                assert_eq!(fields.prefers_non_default_gpu.unwrap(), true);
                assert_eq!(fields.single_main_window.unwrap(), true);
            }
            _ => panic!("Type not Application"),
        }
    }

    #[test]
    fn test_list_values() {
        let content = r#"
[Desktop Entry]
Name=Test App
Exec=test
Type=Application
Categories=Development;IDE;Programming;
MimeType=text/plain;application/x-python;
OnlyShowIn=GNOME;KDE;
NotShowIn=XFCE;
Keywords=development;coding;
Keywords[es]=desarrollo;programación;
Implements=org.freedesktop.Application;
"#;
        let f = parse(content).unwrap();
        let entry = f.entry;
        assert_eq!(entry.only_show_in.unwrap(), vec!["GNOME", "KDE"]);
        assert_eq!(entry.not_show_in.unwrap(), vec!["XFCE"]);

        match entry.entry_type {
            EntryType::Application(fields) => {
                let keywords = fields.keywords.unwrap();
                assert_eq!(keywords.default, vec!["development", "coding"]);
                assert_eq!(
                    keywords.variants.get("es").unwrap(),
                    &vec!["desarrollo", "programación"]
                );

                assert_eq!(
                    fields.categories.unwrap(),
                    vec!["Development", "IDE", "Programming"]
                );
                assert_eq!(
                    fields.mime_type.unwrap(),
                    vec!["text/plain", "application/x-python"]
                );
                assert_eq!(
                    fields.implements.unwrap(),
                    vec!["org.freedesktop.Application"]
                );
            }
            _ => panic!("Entry type is not Application"),
        }
    }

    #[test]
    fn test_entry_types() {
        let app_content = "[Desktop Entry]\nType=Application\nName=Test\nExec=test";
        let link_content = "[Desktop Entry]\nType=Link\nName=Test\nURL=https://example.com";
        let dir_content = "[Desktop Entry]\nType=Directory\nName=Test";
        let unknown_content = "[Desktop Entry]\nType=CustomType\nName=Test";

        let f = parse(app_content).unwrap();
        let app_entry = f.entry;
        let f = parse(link_content).unwrap();
        let link_entry = f.entry;
        let f = parse(dir_content).unwrap();
        let dir_entry = f.entry;
        let f = parse(unknown_content).unwrap();
        let unknown_entry = f.entry;

        assert!(matches!(app_entry.entry_type, EntryType::Application(_)));
        assert!(matches!(link_entry.entry_type, EntryType::Link(_)));
        assert!(matches!(dir_entry.entry_type, EntryType::Directory));
        assert!(matches!(unknown_entry.entry_type, EntryType::Unknown));
    }

    #[test]
    fn test_icon_string() {
        let content = r#"
[Desktop Entry]
Name=Test App
Exec=test
Type=Application
Icon=test-icon
"#;
        let f = parse(content).unwrap();
        let entry = f.entry;
        let icon = entry.icon.unwrap();
        assert_eq!(icon.content, "test-icon");
        // Note: We can't effectively test get_icon_path() here without mocking the filesystem
    }

    #[test]
    #[should_panic]
    fn test_missing_required_fields() {
        let content = r#"
[Desktop Entry]
Exec=test
Type=Application
"#;
        parse(content).unwrap();
    }
}
