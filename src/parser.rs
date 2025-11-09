use crate::token::Token;
use crate::token::{RESERVED_OPS, RESERVED_WORDS};

pub struct Tokenizer;

impl Tokenizer {
    pub fn new() -> Self {
        Tokenizer
    }

    pub fn tokenize(&mut self, input: &str) -> Vec<Token> {
        // 記号トークンを長い順にソート
        let mut sorted_reserved_ops = RESERVED_OPS.to_vec();
        sorted_reserved_ops.sort_by(|a, b| b.len().cmp(&a.len()));

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

            // 記号トークン
            let mut matched = false;
            for op in &sorted_reserved_ops {
                let op_len = op.len();
                if pos + op_len <= chars.len() {
                    let candidate: String = chars[pos..pos + op_len].iter().collect();
                    if candidate == *op {
                        tokens.push(Token::Reserved(op.to_string()));
                        pos += op_len;
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
                if RESERVED_WORDS.contains(&ident.as_str()) {
                    // 予約語はReservedトークンとして扱う
                    tokens.push(Token::Reserved(ident));
                    continue;
                } else {
                    // それ以外は識別子トークン
                    tokens.push(Token::Ident(ident));
                    continue;
                }
            }
            panic!("不明な文字です: {}", c);
        }
        tokens.push(Token::EOF);
        tokens
    }
}
