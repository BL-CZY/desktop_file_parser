pub mod parser;
pub mod structs;

pub use parser::parse;
pub use structs::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "passed"]
    fn entry_test() {
        let a = parse(&std::fs::read_to_string("/usr/share/applications/i3.desktop").unwrap());
        println!("{:?}", a);
    }

    #[test]
    #[ignore = "passed"]
    fn action_test() {
        let a = parse(
            &std::fs::read_to_string("/usr/share/applications/wechat-uos-qt.desktop").unwrap(),
        );
        println!("{:?}", a);
    }
}
