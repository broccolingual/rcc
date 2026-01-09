use crate::errors::CompileError;
use crate::token::{KEYWORDS, PUNCTUATORS, Token, TokenKind};
use crate::utils::Span;

pub(crate) struct Lexer<'a> {
    source: &'a str,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Lexer { source }
    }

    pub(crate) fn tokenize(&self) -> Result<Vec<Token>, CompileError> {
        // 演算子トークンを長い順にソート
        let mut sorted_puncts = PUNCTUATORS.to_vec();
        sorted_puncts.sort_by_key(|a| std::cmp::Reverse(a.len()));

        let mut tokens = Vec::new();
        let bytes = self.source.as_bytes();
        let mut pos = 0;

        while pos < bytes.len() {
            let c = bytes[pos];

            // 空白文字をスキップ
            if matches!(c, b' ' | b'\t' | b'\n' | b'\r') {
                pos += 1;
                continue;
            }

            // 行コメントをスキップ
            if c == b'/' && pos + 1 < bytes.len() && bytes[pos + 1] == b'/' {
                pos += 2;
                while pos < bytes.len() && bytes[pos] != b'\n' {
                    pos += 1;
                }
                pos += 1;
                continue;
            }

            // ブロックコメントをスキップ
            if c == b'/' && pos + 1 < bytes.len() && bytes[pos + 1] == b'*' {
                pos += 2;
                while pos + 1 < bytes.len() {
                    if bytes[pos] == b'*' && bytes[pos + 1] == b'/' {
                        pos += 2;
                        break;
                    }
                    pos += 1;
                }
                if pos == bytes.len() - 1 {
                    return Err(CompileError::InternalError {
                        msg: "ブロックコメントが終了していません".to_string(),
                    });
                }
                continue;
            }

            // 演算子
            let remaining = &self.source[pos..];
            if let Some(symbol) = sorted_puncts.iter().find(|s| remaining.starts_with(*s)) {
                tokens.push(Token::new(
                    TokenKind::Punct(symbol.to_string()),
                    Span::new(pos, pos + symbol.len()),
                ));
                pos += symbol.len();
                continue;
            }

            // 文字定数トークン
            if c == b'\'' {
                let start_pos = pos;
                pos += 1;
                let char_val = bytes[pos] as i64;
                pos += 2;
                tokens.push(Token::new(TokenKind::CharConst(char_val), Span::new(start_pos, pos)));
                continue;
            }

            // 文字列リテラル
            if c == b'"' {
                let start_pos = pos;
                pos += 1;
                let str_start = pos;
                while pos < bytes.len() && bytes[pos] != b'"' {
                    pos += 1;
                }
                let str_lit = self.source[str_start..pos].to_string();
                pos += 1;
                tokens.push(Token::new(TokenKind::StrLiteral(str_lit), Span::new(start_pos, pos)));
                continue;
            }

            // 数字定数
            if c.is_ascii_digit() {
                let start_pos = pos;
                if c != b'0' {
                    // 10進数
                    while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                        pos += 1;
                    }
                    let val = self.source[start_pos..pos].parse::<i64>().unwrap();
                    tokens.push(Token::new(TokenKind::IntConst(val), Span::new(start_pos, pos)));
                } else if pos + 1 < bytes.len() && matches!(bytes[pos + 1], b'x' | b'X') {
                    // 16進数
                    pos += 2;
                    let hex_start = pos;
                    while pos < bytes.len() && bytes[pos].is_ascii_hexdigit() {
                        pos += 1;
                    }
                    let hex_str = if pos > hex_start {
                        &self.source[hex_start..pos]
                    } else {
                        return Err(CompileError::InvalidExpr {
                            msg: "16進数リテラルに数字がありません".to_string(),
                            span: Span::new(start_pos, pos),
                        });
                    };
                    let val = i64::from_str_radix(hex_str, 16).unwrap();
                    tokens.push(Token::new(TokenKind::IntConst(val), Span::new(start_pos, pos)));
                } else {
                    // 8進数
                    pos += 1;
                    let oct_start = pos;
                    while pos < bytes.len() && matches!(bytes[pos], b'0'..=b'7') {
                        pos += 1;
                    }
                    let oct_str = if pos > oct_start { &self.source[oct_start..pos] } else { "0" };
                    let val = i64::from_str_radix(oct_str, 8).unwrap();
                    tokens.push(Token::new(TokenKind::IntConst(val), Span::new(start_pos, pos)));
                }
                continue;
            }

            // 識別子トークン
            if matches!(c, b'a'..=b'z' | b'A'..=b'Z' | b'_') {
                let start_pos = pos;
                while pos < bytes.len()
                    && matches!(bytes[pos], b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')
                {
                    pos += 1;
                }
                let ident = &self.source[start_pos..pos];
                let kind = if KEYWORDS.contains(&ident) {
                    TokenKind::Keyword(ident.to_string())
                } else {
                    TokenKind::Ident(ident.to_string())
                };
                tokens.push(Token::new(kind, Span::new(start_pos, pos)));
                continue;
            }
            return Err(CompileError::MissingToken {
                found: format!("{}", c as char),
                span: Span::new(pos, pos + 1),
            });
        }
        tokens.push(Token::new(TokenKind::Eof, Span::new(pos, pos)));
        Ok(tokens)
    }
}
