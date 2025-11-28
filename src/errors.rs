use core::error;
use core::fmt;

use crate::token::TokenKind;
use crate::types::TypeKind;

#[derive(Debug)]
pub enum CompileError {
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
    },
    Redeclaration {
        name: String,
    },
    InvalidExpression {
        msg: String,
    },
    InvalidStatement {
        msg: String,
    },
    InvalidTypeSpecifier {
        msg: String,
    },
    InvalidDeclaration {
        msg: String,
    },
    InvalidInitializer {
        msg: String,
    },
    InvalidReturnType {
        expected: TypeKind,
        found: TypeKind,
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
            CompileError::UndefinedIdentifier { name } => {
                write!(f, "undefined identifier: '{}'", name)
            }
            CompileError::Redeclaration { name } => {
                write!(f, "redeclaration of variable: '{}'", name)
            }
            CompileError::InvalidExpression { msg } => {
                write!(f, "invalid expression: {}", msg)
            }
            CompileError::InvalidStatement { msg } => {
                write!(f, "invalid statement: {}", msg)
            }
            CompileError::InvalidTypeSpecifier { msg } => {
                write!(f, "invalid type specifier: {}", msg)
            }
            CompileError::InvalidDeclaration { msg } => {
                write!(f, "invalid declaration: {}", msg)
            }
            CompileError::InvalidInitializer { msg } => {
                write!(f, "invalid initializer: {}", msg)
            }
            CompileError::InvalidReturnType { expected, found } => {
                write!(
                    f,
                    "invalid return type: [expected] {:?}, [found] {:?}",
                    expected, found
                )
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
