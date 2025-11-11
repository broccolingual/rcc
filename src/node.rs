use crate::types::Type;

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
    Return,       // return
    Num,          // 整数
    Nop,          // 空命令
}

impl NodeKind {
    pub fn from_str(sym: &str) -> Option<Self> {
        match sym {
            // assignment operators
            "=" => Some(NodeKind::Assign),
            "*=" => Some(NodeKind::MulAssign),
            "/=" => Some(NodeKind::DivAssign),
            "%=" => Some(NodeKind::RemAssign),
            "+=" => Some(NodeKind::AddAssign),
            "-=" => Some(NodeKind::SubAssign),
            "<<=" => Some(NodeKind::ShlAssign),
            ">>=" => Some(NodeKind::ShrAssign),
            "&=" => Some(NodeKind::BitAndAssign),
            "^=" => Some(NodeKind::BitXorAssign),
            "|=" => Some(NodeKind::BitOrAssign),
            // unary operators
            "&" => Some(NodeKind::Addr),
            "*" => Some(NodeKind::Deref),
            "+" => Some(NodeKind::Add),
            "-" => Some(NodeKind::Sub),
            "~" => Some(NodeKind::BitNot),
            "!" => Some(NodeKind::LogicalNot),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
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
        node.ty = Some(Box::new(Type::new_int()));
        node
    }
}
