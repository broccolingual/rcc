#[derive(PartialEq, Eq, Clone, Debug)]
pub enum NodeKind {
    Add,    // +
    Sub,    // -
    Mul,    // *
    Div,    // /
    Eq,     // ==
    Ne,     // !=
    Lt,     // <
    Le,     // <=
    Assign, // =
    LVar,   // ローカル変数
    Return, // return
    Num,    // 整数
}

pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub val: i64,    // kindがNumのときに使う
    pub offset: i64, // kindがLVarのときに使う
}

impl Node {
    pub fn new(kind: NodeKind, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Self {
        Node {
            kind,
            lhs,
            rhs,
            val: 0,
            offset: 0,
        }
    }

    pub fn new_num(val: i64) -> Self {
        Node {
            kind: NodeKind::Num,
            lhs: None,
            rhs: None,
            val,
            offset: 0,
        }
    }
}
