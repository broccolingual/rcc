use core::fmt;

pub const PUNCTUATORS: [&str; 54] = [
    "[", "]", "(", ")", "{", "}", ".", "->", "++", "--", "&", "*", "+", "-", "~", "!", "/", "%",
    "<<", ">>", "<", "<=", ">", ">=", "==", "!=", "^", "|", "&&", "||", "?", ":", ";", "...", "=",
    "*=", "/=", "%=", "+=", "-=", "&=", "^=", "|=", "<<=", ">>=", ",", "#", "##", "<:", ":>", "<%",
    "%>", "%:", "%:%:",
];

pub const KEYWORDS: [&str; 34] = [
    "auto", "break", "case", "char", "const", "continue", "default", "do", "double", "else",
    "enum", "extern", "float", "for", "goto", "if", "inline", "int", "long", "register",
    "restrict", "return", "short", "signed", "sizeof", "static", "struct", "switch", "typedef",
    "union", "unsigned", "void", "volatile", "while",
];

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenKind {
    Punctuator(String), // 記号トークン
    Keyword(String),    // キーワード
    Identifier(String), // 識別子
    Number(i64),        // 整数トークン
    String(String),     // 文字列リテラルトークン
    EOF,                // 入力の終わりを表すトークン
}

#[derive(Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: (usize, usize), // トークンの開始位置と終了位置
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            TokenKind::Punctuator(s) => write!(f, "Punctuator('{}') {:?}", s, self.span),
            TokenKind::Keyword(s) => write!(f, "Keyword('{}') {:?}", s, self.span),
            TokenKind::Identifier(s) => write!(f, "Identifier('{}') {:?}", s, self.span),
            TokenKind::Number(n) => write!(f, "Num({}) {:?}", n, self.span),
            TokenKind::String(s) => write!(f, "StringLiteral(\"{}\") {:?}", s, self.span),
            TokenKind::EOF => write!(f, "EOF {:?}", self.span),
        }
    }
}

impl Token {
    pub fn new(kind: TokenKind, span: (usize, usize)) -> Self {
        Token { kind, span }
    }
}
