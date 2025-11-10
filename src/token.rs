use core::fmt;

pub const RESERVED_SYMBOLS: [&str; 45] = [
    "+", "-", "*", "/", "%", "=", "&", "~", "!", "^", "|", "==", "!=", "<", "<=", ">", ">=", "*=",
    "/=", "%=", "+=", "-=", "&=", "^=", "|=", "<<", ">>", "&&", "||", "++", "--", "<<=", ">>=",
    "->", "(", ")", "{", "}", "[", "]", ";", ",", ".", "?", ":",
];

pub const RESERVED_TYPES: [&str; 12] = [
    "int", "char", "void", "short", "long", "float", "double", "signed", "unsigned", "struct",
    "union", "enum",
];

pub const RESERVED_WORDS: [&str; 20] = [
    "auto", "break", "case", "const", "continue", "default", "do", "else", "extern", "for", "goto",
    "if", "register", "return", "sizeof", "static", "switch", "typedef", "volatile", "while",
];

#[derive(PartialEq, Eq, Clone)]
pub enum Token {
    Type(String),     // 型
    Symbol(String),   // 記号トークン
    Reserved(String), // 予約語
    Ident(String),    // 識別子
    Num(i64),         // 整数トークン
    EOF,              // 入力の終わりを表すトークン
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Type(s) => write!(f, "Type    ('{}')", s),
            Token::Symbol(s) => write!(f, "Symbol  ('{}')", s),
            Token::Reserved(s) => write!(f, "Reserved('{}')", s),
            Token::Ident(s) => write!(f, "Ident   ('{}')", s),
            Token::Num(n) => write!(f, "Num     ({})", n),
            Token::EOF => write!(f, "EOF"),
        }
    }
}
