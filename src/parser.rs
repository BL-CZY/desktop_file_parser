use crate::{
    structs::ParseError, DesktopAction, DesktopEntry, DesktopFile, Header, IconString, LocaleString,
};

#[derive(Debug)]
enum LineType {
    Header,
    ValPair,
}

#[derive(Debug)]
enum EntryType<'a> {
    Entry(&'a mut DesktopEntry),
    Action(&'a mut DesktopAction),
}

#[derive(Debug)]
enum TokenParseState {
    Idle,
    Header,
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

#[derive(Debug)]
enum Value {
    String(String),
    LocaleString(LocaleString),
    IconString(IconString),
    Boolean(bool),
    Number(f64),
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
struct LinePart {
    key: String,
    locale: Option<String>,
    value: String,
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

fn parse_val_pair(line: &Line, current: &mut EntryType) -> Result<(), ParseError> {
    Ok(())
}

pub fn parse(input: &str) -> Result<DesktopFile, ParseError> {
    let lines = filter_lines(input);
    let mut result_entry = DesktopEntry::default();

    let mut is_entry_found = false;
    let mut is_first_entry = true;

    let mut result_actions: Vec<DesktopAction> = vec![];
    let mut current_target = EntryType::Entry(&mut result_entry);

    for line in lines.iter() {
        match current_target {
            EntryType::Entry(ref entry_ref) => match line.line_type() {
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

                            result_actions.push(DesktopAction::default());
                            current_target = EntryType::Action(result_actions.last_mut().unwrap());
                        }
                        _ => {}
                    };
                }
                LineType::ValPair => {}
            },
            EntryType::Action(ref action_ref) => {}
        }
    }

    Ok(DesktopFile {
        entry: result_entry,
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
