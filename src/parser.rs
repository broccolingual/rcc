use crate::token::Token;
use crate::token::{KEYWORDS, PUNCTUATORS};

pub struct Tokenizer {}

impl Tokenizer {
    pub fn new() -> Self {
        Tokenizer {}
    }

    pub fn tokenize(&self, input: &str) -> Result<Vec<Token>, String> {
        // 演算子トークンを長い順にソート
        let mut sorted_punctuators = PUNCTUATORS.to_vec();
        sorted_punctuators.sort_by(|a, b| b.len().cmp(&a.len()));

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
                    return Err("ブロックコメントが閉じられていません".to_string());
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
                        tokens.push(Token::Punctuator(symbol.to_string()));
                        pos += symbol_len;
                        matched = true;
                        break;
                    }
                }
            }
            if matched {
                continue;
            }

            // 数字トークン
            if matches!(c, '0'..='9') {
                let mut num_str = String::new();
                num_str.push(c);
                pos += 1;
                while pos < chars.len() {
                    let next_c = chars[pos];
                    if matches!(next_c, '0'..='9') {
                        num_str.push(next_c);
                        pos += 1;
                    } else {
                        break;
                    }
                }
                let val = num_str.parse::<i64>().unwrap();
                tokens.push(Token::Num(val));
                continue;
            }

            // 識別子トークン
            if matches!(c, 'a'..='z' | 'A'..='Z') {
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
                    tokens.push(Token::Keyword(ident));
                    continue;
                } else {
                    // それ以外は識別子トークン
                    tokens.push(Token::Identifier(ident));
                    continue;
                }
            }
            return Err(format!("不明な文字が含まれています: {}", c));
        }
        tokens.push(Token::EOF);
        Ok(tokens)
    }
}
