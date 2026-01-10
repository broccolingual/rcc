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

    /// C11準拠のエスケープシーケンスを処理する
    /// posは'\\'の次の文字を指している
    /// 戻り値: (処理された文字の値, 消費したバイト数)
    fn parse_escape_sequence(&self, bytes: &[u8], pos: usize) -> Result<(u8, usize), CompileError> {
        if pos >= bytes.len() {
            return Err(CompileError::InvalidExpr {
                msg: "エスケープシーケンスが不正です".to_string(),
                span: Span::new(pos - 1, pos),
            });
        }

        // C11 6.4.4.4 Character constants - simple-escape-sequence
        match bytes[pos] {
            b'\'' => Ok((b'\'', 1)), // \'
            b'"' => Ok((b'"', 1)),   // \"
            b'?' => Ok((b'?', 1)),   // \?
            b'\\' => Ok((b'\\', 1)), // \\
            b'a' => Ok((0x07, 1)),   // \a (alert/bell)
            b'b' => Ok((0x08, 1)),   // \b (backspace)
            b'f' => Ok((0x0C, 1)),   // \f (form feed)
            b'n' => Ok((b'\n', 1)),  // \n (newline)
            b'r' => Ok((b'\r', 1)),  // \r (carriage return)
            b't' => Ok((b'\t', 1)),  // \t (horizontal tab)
            b'v' => Ok((0x0B, 1)),   // \v (vertical tab)

            // C11 6.4.4.4 - octal-escape-sequence: \octal-digit{1,3}
            b'0'..=b'7' => {
                let mut oct_len = 0;
                let mut oct_val = 0u32;
                while oct_len < 3 && pos + oct_len < bytes.len() {
                    if let b'0'..=b'7' = bytes[pos + oct_len] {
                        let digit = (bytes[pos + oct_len] - b'0') as u32;
                        oct_val = oct_val * 8 + digit;
                        oct_len += 1;
                    } else {
                        break;
                    }
                }
                // 値が0-255の範囲に収まるか確認（charの範囲）
                if oct_val > 255 {
                    return Err(CompileError::InvalidExpr {
                        msg: "8進エスケープシーケンスの値が範囲外です".to_string(),
                        span: Span::new(pos, pos + oct_len),
                    });
                }
                Ok((oct_val as u8, oct_len))
            }

            // C11 6.4.4.4 - hexadecimal-escape-sequence: \x hexadecimal-digit+
            b'x' => {
                let mut hex_len = 0;
                let mut hex_val = 0u32;
                while pos + 1 + hex_len < bytes.len()
                    && bytes[pos + 1 + hex_len].is_ascii_hexdigit()
                {
                    let digit = match bytes[pos + 1 + hex_len] {
                        b'0'..=b'9' => (bytes[pos + 1 + hex_len] - b'0') as u32,
                        b'a'..=b'f' => (bytes[pos + 1 + hex_len] - b'a' + 10) as u32,
                        b'A'..=b'F' => (bytes[pos + 1 + hex_len] - b'A' + 10) as u32,
                        _ => unreachable!(),
                    };
                    hex_val = hex_val * 16 + digit;
                    hex_len += 1;
                }
                if hex_len == 0 {
                    return Err(CompileError::InvalidExpr {
                        msg: "16進エスケープシーケンスに数字がありません".to_string(),
                        span: Span::new(pos, pos + 1),
                    });
                }
                // 値が0-255の範囲に収まるか確認
                if hex_val > 255 {
                    return Err(CompileError::InvalidExpr {
                        msg: "16進エスケープシーケンスの値が範囲外です".to_string(),
                        span: Span::new(pos, pos + 1 + hex_len),
                    });
                }
                Ok((hex_val as u8, 1 + hex_len))
            }

            c => Err(CompileError::InvalidExpr {
                msg: format!("不明なエスケープシーケンス: \\{}", c as char),
                span: Span::new(pos - 1, pos),
            }),
        }
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
                    return Err(CompileError::InvalidStmt {
                        msg: "ブロックコメントが終了していません".to_string(),
                        span: Span::new(pos - 2, pos),
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
                if pos >= bytes.len() {
                    return Err(CompileError::InvalidExpr {
                        msg: "文字定数が不正です".to_string(),
                        span: Span::new(start_pos, pos),
                    });
                }

                let char_val = if bytes[pos] == b'\\' {
                    // エスケープシーケンス
                    pos += 1;
                    let (val, consumed) = self.parse_escape_sequence(bytes, pos)?;
                    pos += consumed;
                    val as i64
                } else {
                    // 通常の文字
                    let val = bytes[pos] as i64;
                    pos += 1;
                    val
                };

                if pos >= bytes.len() || bytes[pos] != b'\'' {
                    return Err(CompileError::InvalidExpr {
                        msg: "文字定数が閉じられていません".to_string(),
                        span: Span::new(start_pos, pos),
                    });
                }
                pos += 1;

                tokens.push(Token::new(TokenKind::CharConst(char_val), Span::new(start_pos, pos)));
                continue;
            }

            // 文字列リテラル
            if c == b'"' {
                let start_pos = pos;
                pos += 1;
                let mut str_bytes = Vec::new();

                while pos < bytes.len() && bytes[pos] != b'"' {
                    if bytes[pos] == b'\\' {
                        // エスケープシーケンス
                        pos += 1;
                        let (val, consumed) = self.parse_escape_sequence(bytes, pos)?;
                        str_bytes.push(val);
                        pos += consumed;
                    } else {
                        // 通常の文字
                        str_bytes.push(bytes[pos]);
                        pos += 1;
                    }
                }

                if pos >= bytes.len() {
                    return Err(CompileError::InvalidExpr {
                        msg: "文字列リテラルが閉じられていません".to_string(),
                        span: Span::new(start_pos, pos),
                    });
                }
                pos += 1; // 閉じる '"' をスキップ

                // バイト列を文字列に変換
                let str_lit =
                    String::from_utf8(str_bytes).map_err(|_| CompileError::InvalidExpr {
                        msg: "文字列リテラルが不正なUTF-8です".to_string(),
                        span: Span::new(start_pos, pos),
                    })?;

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
