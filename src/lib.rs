pub mod parser;
pub mod structs;

pub use parser::parse;
pub use structs::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_file_test() {
        let a = parse(&std::fs::read_to_string("/usr/share/applications/i3.desktop").unwrap());
        println!("{:?}", a);
    }
}
