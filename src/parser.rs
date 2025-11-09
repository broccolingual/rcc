use std::fmt;

const RESERVED_TRIPLE_OP: [&str; 2] = ["<<=", ">>="];
const RESERVED_DOUBLE_OP: [&str; 19] = [
    "==", "!=", "<=", ">=", "*=", "/=", "%=", "+=", "-=", "&=", "^=", "|=", "<<", ">>", "&&", "||",
    "++", "--", "->",
];
const RESERVED_SINGLE_OP: &str = "+-*/%=()<>;{}&~!^|[],.?:";
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
        let mut tokens = Vec::new();
        let chars = input.chars().collect::<Vec<char>>();
        let mut pos = 0;

        while pos < chars.len() {
            let c = chars[pos];

            // 空白文字をスキップ
            if c.is_whitespace() {
                pos += 1;
                continue;
            }

            // 3文字の記号トークン
            if pos + 2 < chars.len() {
                let three_char_op: String = chars[pos..pos + 3].iter().collect();
                if RESERVED_TRIPLE_OP.contains(&three_char_op.as_str()) {
                    let token = Token::new(TokenKind::Reserved, &three_char_op);
                    tokens.push(token);
                    pos += 3;
                    continue;
                }
            }

            // 2文字の記号トークン
            if pos + 1 < chars.len() {
                let two_char_op: String = chars[pos..pos + 2].iter().collect();
                if RESERVED_DOUBLE_OP.contains(&two_char_op.as_str()) {
                    let token = Token::new(TokenKind::Reserved, &two_char_op);
                    tokens.push(token);
                    pos += 2;
                    continue;
                }
            }

            // 単一文字の記号トークン
            if RESERVED_SINGLE_OP.contains(c) {
                let token = Token::new(TokenKind::Reserved, &c.to_string());
                tokens.push(token);
                pos += 1;
                continue;
            }

            // 数字トークン
            if c.is_digit(10) {
                let mut num_str = String::new();
                num_str.push(c);
                pos += 1;
                while pos < chars.len() {
                    let next_c = chars[pos];
                    if next_c.is_digit(10) {
                        num_str.push(next_c);
                        pos += 1;
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
                pos += 1;
                while pos < chars.len() {
                    let next_c = chars[pos];
                    if next_c.is_ascii_alphanumeric() || next_c == '_' {
                        ident.push(next_c);
                        pos += 1;
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
