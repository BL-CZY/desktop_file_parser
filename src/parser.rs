use std::{cell::RefCell, rc::Rc};

use crate::{
    structs::ParseError, DesktopAction, DesktopEntry, DesktopFile, Header, IconString,
    LocaleString, LocaleStringList,
};

#[derive(Debug)]
enum LineType {
    Header,
    ValPair,
}

#[derive(Debug)]
enum EntryType {
    Entry(Rc<RefCell<DesktopEntry>>),
    Action(usize),
}

#[derive(Debug)]
struct Line<'a> {
    content: Vec<Character<'a>>,
    line_number: usize,
}

impl<'a> Line<'a> {
    pub fn from_data(line: &'a str, line_number: usize) -> Self {
        let content: Vec<Character<'a>> = line
            .trim_end()
            .char_indices()
            .map(|(col_number, ch)| Character {
                content: &line[col_number..col_number + ch.len_utf8()],
                line_number,
                col_number,
            })
            .filter(|ch| !(ch.col_number == 0 && ch.content == " "))
            .collect();

        Self {
            content,
            line_number,
        }
    }

    pub fn line_type(&self) -> LineType {
        if self.content[0].content == "[" {
            LineType::Header
        } else {
            LineType::ValPair
        }
    }
}

impl<'a> ToString for Line<'a> {
    fn to_string(&self) -> String {
        self.content
            .iter()
            .map(|ch| ch.content.to_string())
            .collect()
    }
}

#[derive(Debug)]
struct Character<'a> {
    content: &'a str,
    line_number: usize,
    col_number: usize,
}

fn filter_lines(input: &str) -> Vec<Line> {
    input
        .split("\n")
        .enumerate()
        .filter(|element| element.1 != "" && !element.1.trim().starts_with("#"))
        .map(|(num, l)| Line::from_data(l, num))
        .collect()
}

fn parse_header(input: &Line) -> Result<Header, ParseError> {
    enum HeaderParseState {
        Idle,
        Content,
    }

    let mut state = HeaderParseState::Idle;
    let mut result = String::new();

    for (ind, ch) in input.content.iter().enumerate() {
        match state {
            HeaderParseState::Idle => match ch.content {
                "[" => {
                    state = HeaderParseState::Content;
                }
                _ => {
                    return Err(ParseError::InternalError {
                        msg: "line is mis-classified as a header".into(),
                        row: ch.line_number,
                        col: ch.col_number,
                    });
                }
            },
            HeaderParseState::Content => match ch.content {
                "]" => {
                    if ind != input.content.len() - 1 {
                        return Err(ParseError::Syntax {
                            msg: "nothing is expected after \"]\"".to_string(),
                            row: ch.line_number,
                            col: ch.col_number,
                        });
                    }
                }
                "[" => {
                    return Err(ParseError::UnacceptableCharacter {
                        ch: ch.content.to_string(),
                        row: ch.line_number,
                        col: ch.col_number,
                        msg: format!("\"{}\" is not accepted in header", ch.content),
                    });
                }
                _ => {
                    if ch.content.chars().next().unwrap().is_control() {
                        return Err(ParseError::UnacceptableCharacter {
                            ch: ch.content.to_string(),
                            row: ch.line_number,
                            col: ch.col_number,
                            msg: "none".to_string(),
                        });
                    }
                    result.push_str(ch.content);
                }
            },
        }
    }

    if result == "Desktop Entry" {
        Ok(Header::DesktopEntry)
    } else if let Some(remain) = result.strip_prefix("Desktop Action ") {
        Ok(Header::DesktopAction {
            name: remain.to_string(),
        })
    } else {
        Ok(Header::Other { name: result })
    }
}

/// Contains the parsed info of a key value line
#[derive(Clone)]
struct LinePart {
    key: String,
    locale: Option<String>,
    value: String,
    line_number: usize,
}

fn split_into_parts(line: &Line) -> Result<LinePart, ParseError> {
    #[cfg(debug_assertions)]
    println!("This line is: {:?}", line.to_string());

    enum State {
        /// the initial key parser
        Key,
        /// the locale parser
        KeyLocale,
        /// the character that ends the locale spec
        LocaleToValue,
        /// the value
        Value,
    }

    let mut result = LinePart {
        key: "".into(),
        locale: None,
        value: "".into(),
        line_number: line.line_number,
    };

    let mut state = State::Key;

    for ch in line.content.iter() {
        match state {
            State::Key => match ch.content {
                "[" => {
                    state = State::KeyLocale;
                    result.locale = Some("".into())
                }

                "=" => state = State::Value,

                "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" | "K" | "L" | "M"
                | "N" | "O" | "P" | "Q" | "R" | "S" | "T" | "U" | "V" | "W" | "X" | "Y" | "Z"
                | "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m"
                | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z"
                | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "0" | "-" => {
                    result.key.push_str(ch.content)
                }

                _ => {
                    return Err(ParseError::Syntax {
                        msg: "Keys shouldn't have characters other than A-Za-z0-9-".into(),
                        row: ch.line_number,
                        col: ch.col_number,
                    })
                }
            },

            State::KeyLocale => match ch.content {
                "]" => state = State::LocaleToValue,

                _ => {
                    if let Some(ref mut str) = result.locale {
                        str.push_str(ch.content);
                    }
                }
            },

            State::LocaleToValue => match ch.content {
                "=" => state = State::Value,

                _ => {
                    return Err(ParseError::Syntax {
                        msg: "Expect \"=\" after \"=\"".into(),
                        row: ch.line_number,
                        col: ch.col_number,
                    });
                }
            },

            State::Value => match ch.content {
                _ => result.value.push_str(ch.content),
            },
        }
    }

    Ok(result)
}

fn set_locale_str(parts: LinePart, str: &mut LocaleString) -> Result<(), ParseError> {
    // make sure that one property is only declared once

    match parts.locale {
        Some(locale) => {
            if str.variants.contains_key(&locale) {
                return Err(ParseError::RepetitiveKey {
                    key: parts.key,
                    row: parts.line_number,
                    col: 0,
                });
            }
            str.variants.insert(locale, parts.value);
        }
        None => {
            if str.default.is_none() {
                str.default = Some(parts.value);
            } else {
                return Err(ParseError::RepetitiveKey {
                    key: parts.key,
                    row: parts.line_number,
                    col: 0,
                });
            }
        }
    }

    Ok(())
}

fn set_optional_locale_str(
    parts: LinePart,
    opt: &mut Option<LocaleString>,
) -> Result<(), ParseError> {
    match opt {
        Some(str) => set_locale_str(parts, str),

        None => Ok({
            let mut inner = LocaleString::default();

            set_locale_str(parts, &mut inner)?;

            *opt = Some(inner);
        }),
    }
}

fn set_bool(parts: LinePart, val: &mut bool) -> Result<(), ParseError> {
    Ok(*val = parts
        .value
        .parse::<bool>()
        .map_err(|_| ParseError::Syntax {
            msg: "Property's value needs to be bool".into(),
            row: parts.line_number,
            col: 0,
        })?)
}

fn set_optional_bool(parts: LinePart, opt: &mut Option<bool>) -> Result<(), ParseError> {
    // check for redeclaration
    match opt {
        Some(_) => {
            return Err(ParseError::RepetitiveKey {
                key: parts.key,
                row: parts.line_number,
                col: 0,
            });
        }
        None => {
            let mut res = false;
            set_bool(parts, &mut res)?;
            *opt = Some(res);
        }
    }

    Ok(())
}

fn set_optional_list(parts: LinePart, opt: &mut Option<Vec<String>>) -> Result<(), ParseError> {
    if !opt.is_none() {
        return Err(ParseError::RepetitiveKey {
            key: parts.key,
            row: parts.line_number,
            col: 0,
        });
    }

    Ok(*opt = Some(
        parts
            .value
            .split(",")
            .map(|s| s.to_string())
            .collect::<Vec<String>>(),
    ))
}

fn set_optional_str(parts: LinePart, opt: &mut Option<String>) -> Result<(), ParseError> {
    if !opt.is_none() {
        return Err(ParseError::RepetitiveKey {
            key: parts.key,
            row: parts.line_number,
            col: 0,
        });
    }

    Ok(*opt = Some(parts.value))
}

fn set_optional_icon_str(parts: LinePart, opt: &mut Option<IconString>) -> Result<(), ParseError> {
    if !opt.is_none() {
        return Err(ParseError::RepetitiveKey {
            key: parts.key,
            row: parts.line_number,
            col: 0,
        });
    }

    Ok(*opt = Some(IconString {
        content: parts.value,
    }))
}

fn fill_entry_val(entry: &mut DesktopEntry, parts: LinePart) -> Result<(), ParseError> {
    match parts.key.as_str() {
        "Type" => {
            if !entry.entry_type.is_none() {
                return Err(ParseError::RepetitiveKey {
                    key: "Type".into(),
                    row: parts.line_number,
                    col: 0,
                });
            }

            entry.entry_type = Some(crate::EntryType::from(parts.value.as_str()));
        }
        "Version" => set_optional_str(parts, &mut entry.version)?,
        "Name" => set_optional_locale_str(parts, &mut entry.name)?,
        "GenericName" => set_optional_locale_str(parts, &mut entry.generic_name)?,
        "NoDisplay" => set_optional_bool(parts, &mut entry.no_display)?,
        "Comment" => set_optional_locale_str(parts, &mut entry.comment)?,
        "Icon" => set_optional_icon_str(parts, &mut entry.icon)?,
        "Hidden" => set_optional_bool(parts, &mut entry.hidden)?,
        "OnlyShowIn" => set_optional_list(parts, &mut entry.only_show_in)?,
        "DbusActivatable" => set_optional_bool(parts, &mut entry.dbus_activatable)?,
        "TryExec" => set_optional_str(parts, &mut entry.try_exec)?,
        "Exec" => set_optional_str(parts, &mut entry.exec)?,
        "Path" => set_optional_str(parts, &mut entry.path)?,
        "Terminal" => set_optional_bool(parts, &mut entry.terminal)?,
        "Actions" => set_optional_list(parts, &mut entry.actions)?,
        "MimeType" => set_optional_list(parts, &mut entry.mime_type)?,
        "Categories" => set_optional_list(parts, &mut entry.categories)?,
        "Implements" => set_optional_list(parts, &mut entry.implements)?,
        "Keywords" => {
            if !entry.keywords.is_none() {
                return Err(ParseError::RepetitiveKey {
                    key: "Keywords".into(),
                    row: parts.line_number,
                    col: 0,
                });
            }

            let split = parts
                .value
                .split(",")
                .map(|str| str.to_string())
                .collect::<Vec<String>>();

            match entry.keywords {
                Some(ref mut kwds) => match parts.locale {
                    Some(locale) => {
                        if kwds.variants.contains_key(&locale) {
                            return Err(ParseError::RepetitiveKey {
                                key: "Keywords".into(),
                                row: parts.line_number,
                                col: 0,
                            });
                        }

                        kwds.variants.insert(locale, split);
                    }
                    None => {
                        if !kwds.default.is_none() {
                            return Err(ParseError::RepetitiveKey {
                                key: "Keywords".into(),
                                row: parts.line_number,
                                col: 0,
                            });
                        }

                        kwds.default = Some(split);
                    }
                },
                None => {
                    let mut res = LocaleStringList::default();
                    match parts.locale {
                        Some(locale) => {
                            res.variants.insert(locale, split);
                        }
                        None => {
                            res.default = Some(split);
                        }
                    }

                    entry.keywords = Some(res);
                }
            }
        }
        "StarupNotify" => set_optional_bool(parts, &mut entry.startup_notify)?,
        "StartupWmClass" => set_optional_str(parts, &mut entry.startup_wm_class)?,
        "URL" => set_optional_str(parts, &mut entry.url)?,
        "PrefersNonDefaultGPU" => set_optional_bool(parts, &mut entry.prefers_non_default_gpu)?,
        "SingleMainWindow" => set_optional_bool(parts, &mut entry.single_main_window)?,

        _ => {}
    }

    Ok(())
}

fn process_entry_val_pair(line: &Line, entry: &mut DesktopEntry) -> Result<(), ParseError> {
    let parts = split_into_parts(line)?;

    fill_entry_val(entry, parts)
}

fn fill_action_val(action: &mut DesktopAction, parts: LinePart) -> Result<(), ParseError> {
    match parts.key.as_str() {
        "Name" => set_optional_locale_str(parts, &mut action.name)?,
        "Exec" => set_optional_str(parts, &mut action.exec)?,
        "Icon" => set_optional_icon_str(parts, &mut action.icon)?,
        _ => {}
    }

    Ok(())
}

fn process_action_val_pair(line: &Line, action: &mut DesktopAction) -> Result<(), ParseError> {
    let parts = split_into_parts(line)?;

    fill_action_val(action, parts)
}

fn key_err(msg: &str) -> Result<(), ParseError> {
    Err(ParseError::KeyError { msg: msg.into() })
}

fn check_locale_str(opt: &Option<LocaleString>, field: &str) -> Result<(), ParseError> {
    match opt {
        Some(v) => {
            if v.default.is_none() {
                key_err(&format!("The default value of {} is required", field))
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
}

fn check_entry(entry: &DesktopEntry) -> Result<(), ParseError> {
    // check required items
    match entry.entry_type {
        Some(ref t) => {
            match entry.name {
                Some(ref n) => {
                    if n.default.is_none() {
                        return key_err("The default value of name is required");
                    }
                }
                None => return key_err("Name is required"),
            }

            match t {
                crate::EntryType::Link => {
                    if entry.url.is_none() {
                        return key_err("URL is required for Link");
                    }
                }

                _ => {}
            }
        }
        None => {
            return key_err("Type is required");
        }
    }

    // check locale strings
    check_locale_str(&entry.generic_name, "GenericName")?;
    check_locale_str(&entry.comment, "Comment")?;

    Ok(())
}

pub fn parse(input: &str) -> Result<DesktopFile, ParseError> {
    let lines = filter_lines(input);
    let result_entry = Rc::new(RefCell::new(DesktopEntry::default()));

    let mut is_entry_found = false;
    let mut is_first_entry = true;

    let mut result_actions: Vec<DesktopAction> = vec![];
    let mut current_target = EntryType::Entry(result_entry.clone());

    for line in lines.iter() {
        match current_target {
            EntryType::Entry(ref entry) => match line.line_type() {
                LineType::Header => {
                    match parse_header(line)? {
                        Header::DesktopEntry => {
                            if is_entry_found {
                                return Err(ParseError::RepetitiveEntry {
                                    msg: "none".into(),
                                    row: line.line_number,
                                    col: 0,
                                });
                            } else {
                                is_entry_found = true;
                            }

                            if !is_first_entry {
                                return Err(ParseError::InternalError { msg: "it should be able to return error when entry is not in the first header".into(), row: line.line_number, col: 0 });
                            } else {
                                is_first_entry = false;
                            }
                        }
                        Header::DesktopAction { name } => {
                            if !is_entry_found {
                                return Err(ParseError::InternalError { msg: "it should be able to return error when an action appears before an entry".into(), row: line.line_number, col: 0 });
                            }

                            if is_first_entry {
                                return Err(ParseError::FormatError {
                                    msg: "none".into(),
                                    row: line.line_number,
                                    col: 0,
                                });
                            }

                            result_actions.push(DesktopAction {
                                ref_name: name,
                                ..Default::default()
                            });

                            current_target = EntryType::Action(result_actions.len() - 1);
                        }
                        _ => {}
                    };
                }
                LineType::ValPair => {
                    process_entry_val_pair(&line, &mut entry.borrow_mut())?;
                }
            },

            EntryType::Action(index) => match line.line_type() {
                LineType::Header => match parse_header(&line)? {
                    Header::DesktopEntry => {
                        return Err(ParseError::RepetitiveEntry {
                            msg: "There should only be one entry on top".into(),
                            row: line.line_number,
                            col: 0,
                        });
                    }
                    Header::DesktopAction { name } => {
                        result_actions.push(DesktopAction {
                            ref_name: name,
                            ..Default::default()
                        });
                        current_target = EntryType::Action(result_actions.len() - 1)
                    }
                    _ => {}
                },
                LineType::ValPair => {
                    let target = &mut result_actions[index];
                    process_action_val_pair(line, target)?;
                }
            },
        }
    }

    let entry = result_entry.take();
    check_entry(&entry)?;

    Ok(DesktopFile {
        entry,
        actions: result_actions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore = "Already tested"]
    #[test]
    fn filter_lines_test() {
        let res = filter_lines("aaa你好 \n\n\n aaaa\n           #sadas")
            .iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>();

        println!("{:?}", res);
        assert_eq!(vec!["aaa你好", "aaaa"], res);
    }
}
