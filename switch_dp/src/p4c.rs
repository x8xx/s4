pub mod token;
pub mod tokenize;
pub mod parser;

pub fn compile(src: &str) -> String {
    let mut token = tokenize::tokenize(src);
    let data = parser::parse(token.get_next_token());
    "".to_string()
}
