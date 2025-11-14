use core::{fmt, str};

use crate::types::{Type, TypeKind};

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub enum NodeKind {
    Add,          // +
    Sub,          // -
    Mul,          // *
    Div,          // /
    Rem,          // %
    Shl,          // <<
    Shr,          // >>
    BitAnd,       // &
    BitXor,       // ^
    BitOr,        // |
    BitNot,       // ~
    LogicalNot,   // !
    LogicalAnd,   // &&
    LogicalOr,    // ||
    Eq,           // ==
    Ne,           // !=
    Lt,           // <
    Le,           // <=
    Ternary,      // ?:
    Assign,       // =
    AddAssign,    // +=
    SubAssign,    // -=
    MulAssign,    // *=
    DivAssign,    // /=
    RemAssign,    // %=
    ShlAssign,    // <<=
    ShrAssign,    // >>=
    BitAndAssign, // &=
    BitOrAssign,  // |=
    BitXorAssign, // ^=
    PreInc,       // ++pre
    PreDec,       // --pre
    PostInc,      // post++
    PostDec,      // post--
    Addr,         // &
    Deref,        // *
    If,           // if
    While,        // while
    For,          // for
    Do,           // do
    Block,        // {}
    Call,         // 関数呼び出し
    Label,        // ラベル
    Goto,         // goto
    Break,        // break
    Continue,     // continue
    LVar,         // ローカル変数
    GVar,         // グローバル変数
    Return,       // return
    Num,          // 整数
    Nop,          // 空命令
}

impl str::FromStr for NodeKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // assignment operators
            "=" => Ok(NodeKind::Assign),
            "*=" => Ok(NodeKind::MulAssign),
            "/=" => Ok(NodeKind::DivAssign),
            "%=" => Ok(NodeKind::RemAssign),
            "+=" => Ok(NodeKind::AddAssign),
            "-=" => Ok(NodeKind::SubAssign),
            "<<=" => Ok(NodeKind::ShlAssign),
            ">>=" => Ok(NodeKind::ShrAssign),
            "&=" => Ok(NodeKind::BitAndAssign),
            "^=" => Ok(NodeKind::BitXorAssign),
            "|=" => Ok(NodeKind::BitOrAssign),
            _ => Err(()),
        }
    }
}

#[derive(Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub val: i64,                // kindがNumのときに使う
    pub offset: i64,             // 変数のオフセット(kindがLVarのときに使う)
    pub name: String,            // 変数名(kindがLVarのときに使う)
    pub ty: Option<Box<Type>>,   // ノードの型情報(kindがLVarのときに使う)
    pub cond: Option<Box<Node>>, // if, while文の条件式
    pub then: Option<Box<Node>>, // if, while文のthen節
    pub els: Option<Box<Node>>,  // if文のelse節
    pub init: Option<Box<Node>>, // for文の初期化式
    pub inc: Option<Box<Node>>,  // for文の更新式
    pub body: Vec<Box<Node>>,    // ブロック内のstatementリスト
    pub label_name: String,      // ラベル名
    pub func_name: String,       // 関数名
    pub args: Vec<Box<Node>>,    // 関数呼び出しの引数リスト
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node {{ {:?}, ty: {:?}", self.kind, self.ty)?;
        if let Some(ref lhs) = self.lhs {
            write!(f, ", lhs: {:?}", lhs)?;
        }
        if let Some(ref rhs) = self.rhs {
            write!(f, ", rhs: {:?}", rhs)?;
        }
        if self.val != 0 {
            write!(f, ", val: {}", self.val)?;
        }
        if self.offset != 0 {
            write!(f, ", offset: {}", self.offset)?;
        }
        if !self.name.is_empty() {
            write!(f, ", name: {}", self.name)?;
        }
        write!(f, " }}")
    }
}

impl Node {
    pub fn new(kind: NodeKind, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Self {
        Node {
            kind,
            lhs,
            rhs,
            val: 0,
            offset: 0,
            name: String::new(),
            ty: None,
            cond: None,
            then: None,
            els: None,
            init: None,
            inc: None,
            body: Vec::new(),
            label_name: String::new(),
            func_name: String::new(),
            args: Vec::new(),
        }
    }

    pub fn from(kind: NodeKind) -> Self {
        Node::new(kind, None, None)
    }

    pub fn new_unary(kind: NodeKind, op: Option<Box<Node>>) -> Self {
        Node::new(kind, op, None)
    }

    pub fn new_num(val: i64) -> Self {
        let mut node = Node::new(NodeKind::Num, None, None);
        node.val = val;
        node.ty = Some(Box::new(Type::new(TypeKind::Int)));
        node
    }

    pub fn new_lvar(name: &str, offset: i64, ty: &Type) -> Self {
        let mut node = Node::new(NodeKind::LVar, None, None);
        node.name = name.to_string();
        node.offset = offset;
        node.ty = Some(Box::new(ty.clone()));
        node
    }

    pub fn new_gvar(name: &str, ty: &Type) -> Self {
        let mut node = Node::new(NodeKind::GVar, None, None);
        node.name = name.to_string();
        node.ty = Some(Box::new(ty.clone()));
        node
    }
}
