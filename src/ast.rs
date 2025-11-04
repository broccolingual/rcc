#[derive(PartialEq, Eq, Clone, Debug)]
pub enum NodeKind {
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Eq,  // ==
    Ne,  // !=
    Lt,  // <
    Le,  // <=
    Num, // 整数
}

pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub val: i64,
}

impl Node {
    pub fn new(kind: NodeKind, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Self {
        Node {
            kind,
            lhs,
            rhs,
            val: 0,
        }
    }

    pub fn new_num(val: i64) -> Self {
        Node {
            kind: NodeKind::Num,
            lhs: None,
            rhs: None,
            val,
        }
    }
}
