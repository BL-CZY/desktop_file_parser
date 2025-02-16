use std::{cell::RefCell, rc::Rc};

use crate::{
    structs::ParseError, DesktopAction, DesktopEntry, DesktopFile, Header, IconString, LocaleString,
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
                | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "0" => {
                    result.key.push_str(ch.content)
                }

                _ => {
                    return Err(ParseError::Syntax {
                        msg: "Keys shouldn't have characters other than A-Za-z0-9".into(),
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

fn set_locale_str_property(parts: LinePart, str: &mut LocaleString) {
    match parts.locale {
        Some(locale) => {
            str.variants.insert(locale, parts.value);
        }
        None => str.default = parts.value,
    }
}

fn set_optional_locale_str_property(parts: LinePart, opt: &mut Option<LocaleString>) {
    match opt {
        Some(str) => set_locale_str_property(parts, str),

        None => {
            let mut inner = LocaleString::default();

            set_locale_str_property(parts, &mut inner);

            *opt = Some(inner);
        }
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
    match opt {
        Some(val) => {
            set_bool(parts, val)?;
        }
        None => {
            let mut res = false;
            set_bool(parts, &mut res)?;
            *opt = Some(res);
        }
    }

    Ok(())
}

fn set_optional_list(parts: LinePart, opt: &mut Option<Vec<String>>) {
    *opt = Some(
        parts
            .value
            .split(",")
            .map(|s| s.to_string())
            .collect::<Vec<String>>(),
    )
}

fn fill_entry_val(entry: &mut DesktopEntry, parts: LinePart) -> Result<(), ParseError> {
    match parts.key.as_str() {
        "Type" => entry.entry_type = parts.value,
        "Version" => entry.version = Some(parts.value),
        "Name" => set_locale_str_property(parts, &mut entry.name),
        "GenericName" => set_optional_locale_str_property(parts, &mut entry.generic_name),
        "NoDisplay" => set_optional_bool(parts, &mut entry.no_display)?,
        "Comment" => set_optional_locale_str_property(parts, &mut entry.comment),
        "Icon" => {
            entry.icon = Some(IconString {
                content: parts.value,
            })
        }
        "Hidden" => set_optional_bool(parts, &mut entry.hidden)?,
        "OnlyShowIn" => set_optional_list(parts, &mut entry.only_show_in),
        "DbusActivatable" => set_optional_bool(parts, &mut entry.dbus_activatable)?,
        "TryExec" => entry.try_exec = Some(parts.value),
        "Exec" => entry.exec = Some(parts.value),
        "Path" => entry.path = Some(parts.value),
        "Terminal" => set_optional_bool(parts, &mut entry.terminal)?,
        "Actions" => set_optional_list(parts, &mut entry.actions),
        "MimeType" => set_optional_list(parts, &mut entry.mime_type),
        "Categories" => set_optional_list(parts, &mut entry.categories),
        "Implements" => set_optional_list(parts, &mut entry.implements),
        "Keywords" => {
            entry.keywords = Some({
                parts
                    .value
                    .split(",")
                    .map(|str| str.to_string())
                    .map(|str| {
                        let mut res = LocaleString::default();
                        set_locale_str_property(
                            LinePart {
                                key: parts.key.clone(),
                                locale: parts.locale.clone(),
                                value: str,
                                line_number: parts.line_number,
                            },
                            &mut res,
                        );
                        res
                    })
                    .collect::<Vec<LocaleString>>()
            })
        }
        "StarupNotify" => set_optional_bool(parts, &mut entry.startup_notify)?,
        "StartupWmClass" => entry.startup_wm_class = Some(parts.value),
        "URL" => entry.url = Some(parts.value),
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

fn fill_action_val(entry: &mut DesktopAction, parts: LinePart) -> Result<(), ParseError> {
    match parts.key.as_str() {
        "Name" => set_locale_str_property(parts, &mut entry.name),
        "Exec" => entry.exec = Some(parts.value),
        "Icon" => {
            entry.icon = Some(IconString {
                content: parts.value,
            })
        }
        _ => {}
    }

    Ok(())
}

fn process_action_val_pair(line: &Line, action: &mut DesktopAction) -> Result<(), ParseError> {
    let parts = split_into_parts(line)?;

    fill_action_val(action, parts)
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

    Ok(DesktopFile {
        entry: result_entry.take(),
        actions: result_actions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
