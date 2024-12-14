use crate::{structs::ParseError, Header, Value};

#[derive(Debug)]
enum LineType {
    Header,
    ValPair,
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

fn parse_val_pair(line: &str) -> Option<LineType> {
    let mut split = line.split("=");

    let key = split.next()?;
    let val = split.next()?;

    let val = val
        .split(";")
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    None
}

fn process_single_line(line: &Line) {
    match line.line_type() {
        LineType::Header => {}
        LineType::ValPair => {}
    }
}

pub fn parse_token(input: &str) -> Vec<String> {
    vec![]
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
