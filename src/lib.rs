pub mod parser;
pub mod structs;

pub use parser::parse_token;
pub use structs::*;

#[derive(Debug, Clone)]
pub struct DesktopAction {
    pub name: String,
    pub description: String,
    pub exec: String,
}

pub fn parse(content: &str) -> Vec<DesktopAction> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_file_test() {
        let a = parse(&std::fs::read_to_string("/usr/share/applications/i3.desktop").unwrap());
        println!("{:?}", a);
    }
}
