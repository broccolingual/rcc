use core::{fmt, str};

use crate::types::Type;

#[derive(PartialEq, Eq, Clone, Debug)]
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
    If {
        cond: Option<Box<Node>>,
        then: Option<Box<Node>>,
        els: Option<Box<Node>>,
    }, // if
    Ternary {
        cond: Option<Box<Node>>,
        then: Option<Box<Node>>,
        els: Option<Box<Node>>,
    }, // cond ? then : else
    While {
        cond: Option<Box<Node>>,
        then: Option<Box<Node>>,
    }, // while
    For {
        init: Option<Box<Node>>,
        cond: Option<Box<Node>>,
        inc: Option<Box<Node>>,
        then: Option<Box<Node>>,
    }, // for
    Do {
        cond: Option<Box<Node>>,
        then: Option<Box<Node>>,
    }, // do
    Block {
        body: Vec<Box<Node>>,
    }, // {}
    Call {
        name: String,
        args: Vec<Box<Node>>,
    }, // 関数呼び出し
    Label {
        name: String,
    }, // ラベル
    Goto {
        name: String,
    }, // goto
    Break,        // break
    Continue,     // continue
    LVar {
        name: String,
        offset: i64,
    }, // ローカル変数
    GVar {
        name: String,
    }, // グローバル変数
    Return,       // return
    Number {
        val: i64,
    }, // 整数
    String {
        val: String,
        index: i64,
    }, // 文字列リテラル
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
    pub ty: Option<Box<Type>>,
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
        match self.kind {
            NodeKind::Number { val } => {
                write!(f, ", val: {}", val)?;
            }
            NodeKind::LVar { ref name, offset } => {
                write!(f, ", name: {}, offset: {}", name, offset)?;
            }
            NodeKind::GVar { ref name } => {
                write!(f, ", name: {}", name)?;
            }
            NodeKind::Call { ref name, ref args } => {
                write!(f, ", name: {}, args: {:?}", name, args)?;
            }
            NodeKind::Label { ref name } => {
                write!(f, ", name: {}", name)?;
            }
            _ => {}
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
            ty: None,
        }
    }
}

impl Node {
    pub fn new(kind: NodeKind, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Self {
        Node {
            kind,
            lhs,
            rhs,
            ty: None,
        }
    }

    pub fn from(kind: NodeKind) -> Self {
        Node::new(kind, None, None)
    }

    pub fn new_unary(kind: NodeKind, op: Option<Box<Node>>) -> Self {
        Node::new(kind, op, None)
    }

    pub fn new_num(val: i64) -> Self {
        let mut node = Node::new(NodeKind::Number { val }, None, None);
        node.ty = Some(Box::new(Type::Int));
        node
    }

    pub fn new_lvar(name: &str, offset: i64, ty: &Type) -> Self {
        let mut node = Node::new(
            NodeKind::LVar {
                name: name.to_string(),
                offset,
            },
            None,
            None,
        );
        node.ty = Some(Box::new(ty.clone()));
        node
    }

    pub fn new_gvar(name: &str, ty: &Type) -> Self {
        let mut node = Node::new(
            NodeKind::GVar {
                name: name.to_string(),
            },
            None,
            None,
        );
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
            NodeKind::Number { .. } => {
                // 数値リテラルの型はすでに設定されているはず
            }
            NodeKind::LVar { .. } => {
                // ローカル変数の型はすでに設定されているはず
            }
            NodeKind::GVar { .. } => {
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
                self.ty = Some(Box::new(Type::Ptr {
                    to: base_ty.clone(),
                }));
            }
            NodeKind::Deref => {
                // デリファレンス演算子の型はポインタの指す型にする
                let ptr_ty = self.lhs.as_ref().unwrap().ty.as_ref().unwrap();
                if !ptr_ty.is_ptr_or_array() {
                    panic!(
                        "ポインタ型ではないものをデリファレンスしようとしました: {:?}",
                        self
                    );
                }
                self.ty = Some(Box::new(ptr_ty.base_type().clone()));
            }
            _ => {
                // その他のノードはとりあえずNoneにする
                self.ty = None;
            }
        }
    }
}
