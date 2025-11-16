use core::fmt;

pub const PUNCTUATORS: [&str; 55] = [
    "[", "]", "(", ")", "{", "}", ".", "->", "++", "--", "&", "*", "+", "-", "~", "!", "/", "%",
    "<<", ">>", "<", "<=", ">", ">=", "==", "!=", "^", "|", "&&", "||", "?", ":", "::", ";", "...",
    "=", "*=", "/=", "%=", "+=", "-=", "&=", "^=", "|=", "<<=", ">>=", ",", "#", "##", "<:", ":>",
    "<%", "%>", "%:", "%:%:",
];

pub const KEYWORDS: [&str; 44] = [
    "alignas",
    "alignof",
    "auto",
    "bool",
    "break",
    "case",
    "char",
    "const",
    "constexpr",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extern",
    "float",
    "for",
    "goto",
    "if",
    "inline",
    "int",
    "long",
    "nullptr",
    "register",
    "restrict",
    "return",
    "short",
    "signed",
    "sizeof",
    "static",
    "static_assert",
    "struct",
    "switch",
    "thread_local",
    "true",
    "typedef",
    "typeof",
    "typeof_unqual",
    "union",
    "unsigned",
    "void",
    "volatile",
    "while",
];

#[derive(PartialEq, Eq, Clone)]
pub enum Token {
    Punctuator(String), // 記号トークン
    Keyword(String),    // キーワード
    Identifier(String), // 識別子
    Number(i64),        // 整数トークン
    String(String),     // 文字列リテラルトークン
    EOF,                // 入力の終わりを表すトークン
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Punctuator(s) => write!(f, "Punctuator('{}')", s),
            Token::Keyword(s) => write!(f, "Keyword('{}')", s),
            Token::Identifier(s) => write!(f, "Identifier('{}')", s),
            Token::Number(n) => write!(f, "Num({})", n),
            Token::String(s) => write!(f, "StringLiteral(\"{}\")", s),
            Token::EOF => write!(f, "EOF"),
        }
    }
}
