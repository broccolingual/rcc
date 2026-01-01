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
        label: usize,
    }, // &&
    LogicalOr {
        lhs: Box<Node>,
        rhs: Box<Node>,
        label: usize,
    }, // ||
    If {
        cond: Box<Node>,
        then: Box<Node>,
        els: Option<Box<Node>>,
        label: usize,
    }, // if
    Ternary {
        cond: Box<Node>,
        then: Box<Node>,
        els: Box<Node>,
        label: usize,
    }, // cond ? then : else
    While {
        cond: Box<Node>,
        then: Box<Node>,
        label: usize,
    }, // while
    For {
        init: Option<Box<Node>>,
        cond: Option<Box<Node>>,
        inc: Option<Box<Node>>,
        then: Box<Node>,
        label: usize,
    }, // for
    Do {
        cond: Box<Node>,
        then: Box<Node>,
        label: usize,
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
    Break {
        label: usize,
    }, // break
    Continue {
        label: usize,
    }, // continue
    Var {
        name: String,
        offset: usize,
        is_local: bool,
    }, // 変数
    Member {
        obj: Box<Node>,
        name: String,
        offset: usize,
    }, // 構造体メンバーアクセス
    Ident {
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
    Nop, // 空命令
}
