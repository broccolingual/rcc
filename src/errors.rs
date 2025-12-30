use core::error;
use core::fmt;

use crate::token::TokenKind;

#[derive(Debug)]
pub(crate) enum CompileError {
    UnexpectedToken {
        expected: TokenKind,
        found: TokenKind,
        span: (usize, usize),
    },
    MissingToken {
        found: String,
        span: (usize, usize),
    },
    UndefinedIdentifier {
        name: String,
        span: (usize, usize),
    },
    ReadOnlyLvalue {
        name: String,
        span: (usize, usize),
    },
    Redeclaration {
        name: String,
        span: (usize, usize),
    },
    InvalidExpression {
        msg: String,
        span: (usize, usize),
    },
    InvalidStatement {
        msg: String,
        span: (usize, usize),
    },
    InvalidDeclaration {
        msg: String,
        span: (usize, usize),
    },
    UnexpectedEof,
    InternalError {
        msg: String,
    },
}

impl error::Error for CompileError {}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::UnexpectedToken {
                expected, found, ..
            } => {
                write!(
                    f,
                    "Unexpected Token\n  Expected: {:?}\n  Found: {:?}",
                    expected, found
                )
            }
            CompileError::MissingToken { found, .. } => {
                write!(f, "Missing Token: {}", found)
            }
            CompileError::UndefinedIdentifier { name, .. } => {
                write!(
                    f,
                    "Undefined Identifier: '{}'\n  ヒント: 変数や関数が宣言されていない可能性があります",
                    name
                )
            }
            CompileError::ReadOnlyLvalue { name, .. } => {
                write!(
                    f,
                    "Read-Only Lvalue: '{}'\n  ヒント: const修飾された変数には代入できません",
                    name
                )
            }
            CompileError::Redeclaration { name, .. } => {
                write!(
                    f,
                    "Redeclaration: '{}'\n  ヒント: 同じスコープ内で同じ名前の変数を複数回宣言することはできません",
                    name
                )
            }
            CompileError::InvalidExpression { msg, .. } => {
                write!(f, "Invalid Expression: {}", msg)
            }
            CompileError::InvalidStatement { msg, .. } => {
                write!(f, "Invalid Statement: {}", msg)
            }
            CompileError::InvalidDeclaration { msg, .. } => {
                write!(f, "Invalid Declaration: {}", msg)
            }
            CompileError::UnexpectedEof => {
                write!(
                    f,
                    "Unexpected End of File\n  ヒント: 閉じ括弧やセミコロンが不足している可能性があります"
                )
            }
            CompileError::InternalError { msg } => {
                write!(
                    f,
                    "Internal Error: {}\n  これはコンパイラのバグの可能性があります",
                    msg
                )
            }
        }
    }
}

impl CompileError {
    pub(crate) fn format_error(&self, source: &str) -> String {
        let span = match self {
            CompileError::UnexpectedToken { span, .. }
            | CompileError::MissingToken { span, .. }
            | CompileError::UndefinedIdentifier { span, .. }
            | CompileError::ReadOnlyLvalue { span, .. }
            | CompileError::Redeclaration { span, .. }
            | CompileError::InvalidExpression { span, .. }
            | CompileError::InvalidStatement { span, .. }
            | CompileError::InvalidDeclaration { span, .. } => Some(*span),
            _ => None,
        };

        if let Some(span) = span {
            self.format_error_with_source(source, span)
        } else {
            format!("{}", self)
        }
    }

    fn format_error_with_source(&self, source: &str, span: (usize, usize)) -> String {
        let (start, end) = span;
        let (line_num, col_num) = self.get_line_and_column(source, start);
        let line_content = self.get_line_content(source, line_num);

        // 基本エラーメッセージ
        let error_msg = format!("{}", self);

        // 位置情報
        let location = format!("\n  --> line {}:{}", line_num, col_num);

        // ソースコード行の表示
        let line_num_width = line_num.to_string().len();
        let line_display = format!("\n{:>width$} |", "", width = line_num_width);
        let source_line = format!(
            "\n{:>width$} | {}",
            line_num,
            line_content,
            width = line_num_width
        );

        // エラー箇所を示す矢印
        let error_length = if end > start { end - start } else { 1 };
        let arrow_padding = " ".repeat(col_num - 1);
        let arrows =
            "^".repeat(error_length.min(line_content.len().saturating_sub(col_num - 1).max(1)));
        let arrow_line = format!(
            "\n{:>width$} | {}{}",
            "",
            arrow_padding,
            arrows,
            width = line_num_width
        );

        format!(
            "{}{}{}{}{}",
            error_msg, location, line_display, source_line, arrow_line
        )
    }

    fn get_line_and_column(&self, source: &str, pos: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;

        for (i, ch) in source.char_indices() {
            if i >= pos {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        (line, col)
    }

    fn get_line_content<'a>(&self, source: &'a str, line_num: usize) -> &'a str {
        source.lines().nth(line_num.saturating_sub(1)).unwrap_or("")
    }
}
