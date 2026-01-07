use core::error;
use core::fmt;

use crate::span::Span;
use crate::token::TokenKind;

#[derive(Debug)]
pub(crate) enum CompileError {
    UnexpectedToken {
        expected: TokenKind,
        found: TokenKind,
        span: Span,
    },
    MissingToken {
        found: String,
        span: Span,
    },
    UndefinedIdent {
        name: String,
        span: Span,
    },
    UndefinedFunc {
        name: String,
        span: Span,
    },
    ReadOnlyLvalue {
        name: String,
        span: Span,
    },
    Redecl {
        name: String,
        span: Span,
    },
    InvalidExpr {
        msg: String,
        span: Span,
    },
    InvalidStmt {
        msg: String,
        span: Span,
    },
    InvalidDecl {
        msg: String,
        span: Span,
    },
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
            CompileError::UndefinedIdent { name, .. } => {
                write!(
                    f,
                    "Undefined Identifier: '{}'\n  ヒント: 変数や関数が宣言されていない可能性があります",
                    name
                )
            }
            CompileError::UndefinedFunc { name, .. } => {
                write!(
                    f,
                    "Undefined Function: '{}'\n  ヒント: 関数が宣言されていない可能性があります",
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
            CompileError::Redecl { name, .. } => {
                write!(
                    f,
                    "Redeclaration: '{}'\n  ヒント: 同じスコープ内で同じ名前の変数を複数回宣言することはできません",
                    name
                )
            }
            CompileError::InvalidExpr { msg, .. } => {
                write!(f, "Invalid Expression: {}", msg)
            }
            CompileError::InvalidStmt { msg, .. } => {
                write!(f, "Invalid Statement: {}", msg)
            }
            CompileError::InvalidDecl { msg, .. } => {
                write!(f, "Invalid Declaration: {}", msg)
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
            | CompileError::UndefinedIdent { span, .. }
            | CompileError::UndefinedFunc { span, .. }
            | CompileError::ReadOnlyLvalue { span, .. }
            | CompileError::Redecl { span, .. }
            | CompileError::InvalidExpr { span, .. }
            | CompileError::InvalidStmt { span, .. }
            | CompileError::InvalidDecl { span, .. } => Some(*span),
            _ => None,
        };

        if let Some(span) = span {
            self.format_error_with_source(source, span)
        } else {
            format!("{}", self)
        }
    }

    fn format_error_with_source(&self, source: &str, span: Span) -> String {
        let (start, end) = (span.start, span.end);
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
