use crate::span::Span;
use core::fmt;

pub(crate) const PUNCTUATORS: [&str; 54] = [
    "[", "]", "(", ")", "{", "}", ".", "->", "++", "--", "&", "*", "+", "-", "~", "!", "/", "%",
    "<<", ">>", "<", "<=", ">", ">=", "==", "!=", "^", "|", "&&", "||", "?", ":", ";", "...", "=",
    "*=", "/=", "%=", "+=", "-=", "&=", "^=", "|=", "<<=", ">>=", ",", "#", "##", "<:", ":>", "<%",
    "%>", "%:", "%:%:",
];

pub(crate) const KEYWORDS: [&str; 34] = [
    "auto", "break", "case", "char", "const", "continue", "default", "do", "double", "else",
    "enum", "extern", "float", "for", "goto", "if", "inline", "int", "long", "register",
    "restrict", "return", "short", "signed", "sizeof", "static", "struct", "switch", "typedef",
    "union", "unsigned", "void", "volatile", "while",
];

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum TokenKind {
    Punct(String),   // 記号トークン
    Keyword(String), // キーワード
    Ident(String),   // 識別子
    Number(i64),     // 整数トークン
    String(String),  // 文字列リテラルトークン
    Eof,             // 入力の終わりを表すトークン
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) span: Span, // トークンの開始位置と終了位置
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            TokenKind::Punct(s) => write!(f, "Punct('{}') {:?}", s, self.span),
            TokenKind::Keyword(s) => write!(f, "Keyword('{}') {:?}", s, self.span),
            TokenKind::Ident(s) => write!(f, "Ident('{}') {:?}", s, self.span),
            TokenKind::Number(n) => write!(f, "Num({}) {:?}", n, self.span),
            TokenKind::String(s) => write!(f, "StringLiteral(\"{}\") {:?}", s, self.span),
            TokenKind::Eof => write!(f, "EOF {:?}", self.span),
        }
    }
}

impl Token {
    pub(crate) fn new(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }
}
