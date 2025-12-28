use crate::node::Node;
use crate::node::operator::{BinaryOp, UnaryOp};

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) enum NodeKind {
    BinaryOp {
        op: BinaryOp,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Node>,
    },
    Assign {
        op: BinaryOp,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    LogicalAnd {
        lhs: Box<Node>,
        rhs: Box<Node>,
    }, // &&
    LogicalOr {
        lhs: Box<Node>,
        rhs: Box<Node>,
    }, // ||
    If {
        cond: Box<Node>,
        then: Box<Node>,
        els: Option<Box<Node>>,
    }, // if
    Ternary {
        cond: Box<Node>,
        then: Box<Node>,
        els: Box<Node>,
    }, // cond ? then : else
    While {
        cond: Box<Node>,
        then: Box<Node>,
    }, // while
    For {
        init: Option<Box<Node>>,
        cond: Option<Box<Node>>,
        inc: Option<Box<Node>>,
        then: Box<Node>,
    }, // for
    Do {
        cond: Box<Node>,
        then: Box<Node>,
    }, // do
    Block {
        body: Vec<Node>,
    }, // {}
    Call {
        name: String,
        args: Vec<Node>,
    }, // 関数呼び出し
    Label {
        name: String,
        expr: Box<Node>,
    }, // ラベル
    Goto {
        name: String,
    }, // goto
    Break,    // break
    Continue, // continue
    Var {
        name: String,
        offset: usize,
        is_local: bool,
    }, // 変数
    Identifier {
        name: String,
    }, // 識別子（変数名など）
    Return {
        expr: Option<Box<Node>>,
    }, // return
    Number {
        val: i64,
    }, // 整数
    String {
        val: String,
        index: usize,
    }, // 文字列リテラル
    Nop,      // 空命令
}
