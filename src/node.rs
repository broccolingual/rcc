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
    Number,       // 整数
    String,       // 文字列リテラル
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

#[derive(Clone, PartialEq, Eq)]
pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub val: i64,                // kindがNumberのときに使う
    pub offset: i64,             // 変数のオフセット / ストリングリテラルの識別子
    pub name: String,            // 変数名 / 関数名 / ストリングリテラルの内容 / ラベル名
    pub ty: Option<Box<Type>>,   // ノードの型情報(kindがLVarのときに使う)
    pub cond: Option<Box<Node>>, // if, while文の条件式
    pub then: Option<Box<Node>>, // if, while文のthen節
    pub els: Option<Box<Node>>,  // if文のelse節
    pub init: Option<Box<Node>>, // for文の初期化式
    pub inc: Option<Box<Node>>,  // for文の更新式
    pub body: Vec<Box<Node>>,    // ブロック内のstatementリスト
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
        if self.kind == NodeKind::Number {
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

impl Default for Node {
    fn default() -> Self {
        Node {
            kind: NodeKind::Nop,
            lhs: None,
            rhs: None,
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
            args: Vec::new(),
        }
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
        let mut node = Node::new(NodeKind::Number, None, None);
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

    pub fn assign_types(&mut self) {
        if let Some(ref mut lhs) = self.lhs {
            lhs.assign_types();
        }
        if let Some(ref mut rhs) = self.rhs {
            rhs.assign_types();
        }

        match self.kind {
            NodeKind::Number => {
                // 数値リテラルの型はすでに設定されているはず
            }
            NodeKind::LVar => {
                // ローカル変数の型はすでに設定されているはず
            }
            NodeKind::GVar => {
                // グローバル変数の型はすでに設定されているはず
            }
            NodeKind::Add
            | NodeKind::Sub
            | NodeKind::Mul
            | NodeKind::Div
            | NodeKind::Rem
            | NodeKind::Shl
            | NodeKind::Shr
            | NodeKind::BitAnd
            | NodeKind::BitOr
            | NodeKind::BitXor => {
                // 二項演算子の型は左辺から決定
                self.ty = self.lhs.as_ref().unwrap().ty.clone();
            }
            NodeKind::Addr => {
                // アドレス演算子の型はポインタ型にする
                let base_ty = self.lhs.as_ref().unwrap().ty.as_ref().unwrap();
                let ptr_ty = Type::new_ptr(base_ty);
                self.ty = Some(Box::new(ptr_ty));
            }
            NodeKind::Deref => {
                // デリファレンス演算子の型はポインタの指す型にする
                let ptr_ty = self.lhs.as_ref().unwrap().ty.as_ref().unwrap();
                let to = match &ptr_ty.kind {
                    TypeKind::Ptr { to } => to,
                    TypeKind::Array { base, .. } => base,
                    _ => panic!(
                        "ポインタ型ではないものをデリファレンスしようとしました: {:?}",
                        self
                    ),
                };
                self.ty = Some(to.clone());
            }
            _ => {
                // その他のノードはとりあえずNoneにする
                self.ty = None;
            }
        }
    }
}
