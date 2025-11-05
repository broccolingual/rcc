use std::fmt;

const RESERVED_COMP_OP: [&str; 4] = ["==", "!=", "<=", ">="];
const RESERVED_SINGLE_OP: &str = "+-*/%=()<>;{}";
const RESERVED_WORDS: [&str; 32] = [
    "auto", "break", "case", "char", "const", "continue", "default", "do", "double", "else",
    "enum", "extern", "float", "for", "goto", "if", "int", "long", "register", "return", "short",
    "signed", "sizeof", "static", "struct", "switch", "typedef", "union", "unsigned", "void",
    "volatile", "while",
];

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum TokenKind {
    Reserved, // 記号
    Ident,    // 識別子
    Num,      // 整数トークン
    EOF,      // 入力の終わりを表すトークン
}

#[derive(Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub val: i64,      // kindがNumの場合、その数値
    pub input: String, // トークン文字列
}

impl Token {
    fn new(kind: TokenKind, input: &str) -> Self {
        Token {
            kind,
            val: 0,
            input: input.to_string(),
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            TokenKind::Reserved => write!(f, "Reserved('{}')", self.input),
            TokenKind::Ident => write!(f, "Ident('{}')", self.input),
            TokenKind::Num => write!(f, "Num({})", self.val),
            TokenKind::EOF => write!(f, "EOF"),
        }
    }
}

pub struct Tokenizer;

impl Tokenizer {
    pub fn new() -> Self {
        Tokenizer
    }

    pub fn tokenize(&mut self, input: &str) -> Vec<Token> {
        let mut c_iter = input.chars().peekable();
        let mut tokens = Vec::new();

        while let Some(c) = c_iter.next() {
            // 空白文字をスキップ
            if c.is_whitespace() {
                continue;
            }

            // 2文字の記号トークン
            if let Some(&next_c) = c_iter.peek() {
                let two_char_op = format!("{}{}", c, next_c);
                if RESERVED_COMP_OP.contains(&two_char_op.as_str()) {
                    let token = Token::new(TokenKind::Reserved, &two_char_op);
                    tokens.push(token);
                    c_iter.next(); // 次の文字を消費
                    continue;
                }
            }

            // 単一文字の記号トークン
            if RESERVED_SINGLE_OP.contains(c) {
                let token = Token::new(TokenKind::Reserved, &c.to_string());
                tokens.push(token);
                continue;
            }

            if c.is_digit(10) {
                let mut num_str = String::new();
                num_str.push(c);
                while let Some(&next_c) = c_iter.peek() {
                    if next_c.is_digit(10) {
                        num_str.push(next_c);
                        c_iter.next();
                    } else {
                        break;
                    }
                }
                let val = num_str.parse::<i64>().unwrap();
                let token = Token {
                    kind: TokenKind::Num,
                    val,
                    input: num_str,
                };
                tokens.push(token);
                continue;
            }

            // 識別子トークン（ローカル変数: 複数文字対応）
            if c.is_ascii_alphabetic() {
                let mut ident = c.to_string();
                while let Some(&next_c) = c_iter.peek() {
                    if next_c.is_ascii_alphanumeric() || next_c == '_' {
                        ident.push(next_c);
                        c_iter.next();
                    } else {
                        break;
                    }
                }
                if RESERVED_WORDS.contains(&ident.as_str()) {
                    let token = Token::new(TokenKind::Reserved, &ident);
                    tokens.push(token);
                    continue;
                } else {
                    let token = Token::new(TokenKind::Ident, &ident);
                    tokens.push(token);
                    continue;
                }
            }
            panic!("不明な文字です: {}", c);
        }
        let eof_token = Token::new(TokenKind::EOF, "");
        tokens.push(eof_token);
        tokens
    }
}
