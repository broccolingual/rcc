pub const RESERVED_OPS: [&str; 45] = [
    "+", "-", "*", "/", "%", "=", "(", ")", "<", ">", ";", "{", "}", "&", "~", "!", "^", "|", "[",
    "]", ",", ".", "?", ":", "==", "!=", "<=", ">=", "*=", "/=", "%=", "+=", "-=", "&=", "^=",
    "|=", "<<", ">>", "&&", "||", "++", "--", "->", "<<=", ">>=",
];

pub const RESERVED_WORDS: [&str; 32] = [
    "auto", "break", "case", "char", "const", "continue", "default", "do", "double", "else",
    "enum", "extern", "float", "for", "goto", "if", "int", "long", "register", "return", "short",
    "signed", "sizeof", "static", "struct", "switch", "typedef", "union", "unsigned", "void",
    "volatile", "while",
];

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Token {
    Reserved(String), // 記号
    Ident(String),    // 識別子
    Num(i64),         // 整数トークン
    EOF,              // 入力の終わりを表すトークン
}
