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
                expected,
                found,
                span,
            } => {
                write!(
                    f,
                    "unexpected token: [expected] {:?}, [found] {:?} at {:?}",
                    expected, found, span
                )
            }
            CompileError::MissingToken { found, span } => {
                write!(f, "missing token: {} at {:?}", found, span)
            }
            CompileError::UndefinedIdentifier { name, span } => {
                write!(f, "undefined identifier: '{}' at {:?}", name, span)
            }
            CompileError::ReadOnlyLvalue { name, span } => {
                write!(
                    f,
                    "attempt to assign to read-only variable: '{}' at {:?}",
                    name, span
                )
            }
            CompileError::Redeclaration { name, span } => {
                write!(f, "redeclaration of variable: '{}' at {:?}", name, span)
            }
            CompileError::InvalidExpression { msg, span } => {
                write!(f, "invalid expression: {} at {:?}", msg, span)
            }
            CompileError::InvalidStatement { msg, span } => {
                write!(f, "invalid statement: {} at {:?}", msg, span)
            }
            CompileError::InvalidDeclaration { msg } => {
                write!(f, "invalid declaration: {}", msg)
            }
            CompileError::UnexpectedEof => {
                write!(f, "unexpected end of file")
            }
            CompileError::InternalError { msg } => {
                write!(f, "internal error: {}", msg)
            }
        }
    }
}
