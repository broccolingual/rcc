use crate::errors::CompileError;
use crate::token::{KEYWORDS, PUNCTUATORS};
use crate::token::{Token, TokenKind};

pub struct Lexer {}

impl Default for Lexer {
    fn default() -> Self {
        Self::new()
    }
}

impl Lexer {
    pub fn new() -> Self {
        Lexer {}
    }

    pub fn tokenize(&self, input: &str) -> Result<Vec<Token>, CompileError> {
        // 演算子トークンを長い順にソート
        let mut sorted_punctuators = PUNCTUATORS.to_vec();
        sorted_punctuators.sort_by_key(|a| std::cmp::Reverse(a.len()));

        let mut tokens = Vec::new();
        let chars = input.chars().collect::<Vec<char>>();
        let mut pos = 0;

        while pos < chars.len() {
            let c = chars[pos];

            // 空白文字をスキップ
            if matches!(c, ' ' | '\t' | '\n' | '\r') {
                pos += 1;
                continue;
            }

            // 行コメントをスキップ
            if c == '/' && pos + 1 < chars.len() && chars[pos + 1] == '/' {
                pos += 2;
                while pos < chars.len() && chars[pos] != '\n' {
                    pos += 1;
                }
                pos += 1;
                continue;
            }

            // ブロックコメントをスキップ
            if c == '/' && pos + 1 < chars.len() && chars[pos + 1] == '*' {
                pos += 2;
                while pos + 1 < chars.len() {
                    if chars[pos] == '*' && chars[pos + 1] == '/' {
                        pos += 2;
                        break;
                    }
                    pos += 1;
                }
                if pos == chars.len() - 1 {
                    return Err(CompileError::InternalError {
                        msg: "unterminated block comment".to_string(),
                    });
                }
                continue;
            }

            // 演算子トークン
            let mut matched = false;
            for symbol in &sorted_punctuators {
                let symbol_len = symbol.len();
                if pos + symbol_len <= chars.len() {
                    let candidate: String = chars[pos..pos + symbol_len].iter().collect();
                    if candidate == *symbol {
                        tokens.push(Token::new(
                            TokenKind::Punctuator(symbol.to_string()),
                            (pos, pos + symbol_len),
                        ));
                        pos += symbol_len;
                        matched = true;
                        break;
                    }
                }
            }
            if matched {
                continue;
            }

            // 文字列リテラルトークン
            if c == '"' {
                pos += 1; // 開始の"をスキップ
                let mut str_lit = String::new();
                while pos < chars.len() {
                    let next_c = chars[pos];
                    if next_c == '"' {
                        pos += 1; // 終了の"をスキップ
                        break;
                    } else {
                        str_lit.push(next_c);
                        pos += 1;
                    }
                }
                tokens.push(Token::new(
                    TokenKind::String(str_lit.clone()),
                    (pos - str_lit.len() - 2, pos),
                ));
                continue;
            }

            // 数字トークン
            if c.is_ascii_digit() {
                let mut num_str = String::new();
                num_str.push(c);
                pos += 1;
                while pos < chars.len() {
                    let next_c = chars[pos];
                    if next_c.is_ascii_digit() {
                        num_str.push(next_c);
                        pos += 1;
                    } else {
                        break;
                    }
                }
                let val = num_str.parse::<i64>().unwrap();
                tokens.push(Token::new(
                    TokenKind::Number(val),
                    (pos - num_str.len(), pos),
                ));
                continue;
            }

            // 識別子トークン
            if matches!(c, 'a'..='z' | 'A'..='Z' | '_') {
                let mut ident = c.to_string();
                pos += 1;
                while pos < chars.len() {
                    let next_c = chars[pos];
                    if matches!(next_c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_') {
                        ident.push(next_c);
                        pos += 1;
                    } else {
                        break;
                    }
                }
                if KEYWORDS.contains(&ident.as_str()) {
                    // 予約語はKeywordトークンとして扱う
                    tokens.push(Token::new(
                        TokenKind::Keyword(ident.clone()),
                        (pos - ident.len(), pos),
                    ));
                    continue;
                } else {
                    // それ以外は識別子トークン
                    tokens.push(Token::new(
                        TokenKind::Identifier(ident.clone()),
                        (pos - ident.len(), pos),
                    ));
                    continue;
                }
            }
            return Err(CompileError::MissingToken {
                found: c.to_string(),
                span: (pos, pos + 1),
            });
        }
        tokens.push(Token::new(TokenKind::EOF, (pos, pos)));
        Ok(tokens)
    }
}
