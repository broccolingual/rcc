use core::{fmt, str};

use crate::errors::CompileError;
use crate::types::{Type, TypeKind};

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
        node.ty = Some(Box::new(Type::new(&TypeKind::Int)));
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

    pub fn assign_types(&mut self) -> Result<(), CompileError> {
        if let Some(ref mut lhs) = self.lhs {
            lhs.assign_types()?;
        }
        if let Some(ref mut rhs) = self.rhs {
            rhs.assign_types()?;
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
            NodeKind::Add | NodeKind::Sub | NodeKind::Mul | NodeKind::Div => {
                let lhs_ty = self.lhs.as_ref().unwrap().ty.as_ref().unwrap();
                let rhs_ty = self.rhs.as_ref().unwrap().ty.as_ref().unwrap();

                if lhs_ty.is_scalar() && rhs_ty.is_scalar() {
                    // 両方ともスカラー型の場合、大きい方の型に合わせる
                    if lhs_ty.size_of() >= rhs_ty.size_of() {
                        self.ty = Some(lhs_ty.clone());
                    } else {
                        self.ty = Some(rhs_ty.clone());
                    }
                } else if lhs_ty.is_ptr_or_array() && rhs_ty.is_scalar() {
                    // 左辺がポインタ/配列型、右辺がスカラー型の場合、左辺の型を結果型とする
                    self.ty = Some(lhs_ty.clone());
                } else if lhs_ty.is_scalar() && rhs_ty.is_ptr_or_array() {
                    // 右辺がポインタ/配列型、左辺がスカラー型の場合、右辺の型を結果型とする
                    self.ty = Some(rhs_ty.clone());
                } else {
                    return Err(CompileError::InvalidExpression {
                        msg: format!(
                            "算術演算子はスカラー型またはポインタ/配列型にのみ適用可能です: {:?} と {:?}",
                            lhs_ty, rhs_ty
                        ),
                    });
                }
            }
            NodeKind::Rem => {
                let lhs_ty = self.lhs.as_ref().unwrap().ty.as_ref().unwrap();
                let rhs_ty = self.rhs.as_ref().unwrap().ty.as_ref().unwrap();

                if lhs_ty.is_integer() && rhs_ty.is_integer() {
                    // 両方とも整数型の場合、大きい方の型に合わせる
                    if lhs_ty.size_of() >= rhs_ty.size_of() {
                        self.ty = Some(lhs_ty.clone());
                    } else {
                        self.ty = Some(rhs_ty.clone());
                    }
                } else {
                    return Err(CompileError::InvalidExpression {
                        msg: format!(
                            "剰余演算子は整数型にのみ適用可能です: {:?} と {:?}",
                            lhs_ty, rhs_ty
                        ),
                    });
                }
            }
            NodeKind::BitAnd | NodeKind::BitOr | NodeKind::BitXor => {
                let lhs_ty = self.lhs.as_ref().unwrap().ty.as_ref().unwrap();
                let rhs_ty = self.rhs.as_ref().unwrap().ty.as_ref().unwrap();

                if lhs_ty.is_integer() && rhs_ty.is_integer() {
                    // 両方とも整数型の場合、大きい方の型に合わせる
                    if lhs_ty.size_of() >= rhs_ty.size_of() {
                        self.ty = Some(lhs_ty.clone());
                    } else {
                        self.ty = Some(rhs_ty.clone());
                    }
                } else {
                    return Err(CompileError::InvalidExpression {
                        msg: format!(
                            "ビット演算子は整数型にのみ適用可能です: {:?} と {:?}",
                            lhs_ty, rhs_ty
                        ),
                    });
                }
            }
            NodeKind::Shl | NodeKind::Shr => {
                let lhs_ty = self.lhs.as_ref().unwrap().ty.as_ref().unwrap();
                let rhs_ty = self.rhs.as_ref().unwrap().ty.as_ref().unwrap();

                if lhs_ty.is_integer() && rhs_ty.is_integer() {
                    // 両方とも整数型の場合、昇格後の型を結果型とする
                    self.ty = Some(Box::new(Type::new(&TypeKind::Int)));
                } else {
                    return Err(CompileError::InvalidExpression {
                        msg: format!(
                            "シフト演算子は整数型にのみ適用可能です: {:?} と {:?}",
                            lhs_ty, rhs_ty
                        ),
                    });
                }
            }
            NodeKind::Eq | NodeKind::Ne | NodeKind::Lt | NodeKind::Le => {
                let lhs_ty = self.lhs.as_ref().unwrap().ty.as_ref().unwrap();
                let rhs_ty = self.rhs.as_ref().unwrap().ty.as_ref().unwrap();

                if lhs_ty.is_scalar() && rhs_ty.is_scalar()
                    || lhs_ty.is_ptr_or_array() && rhs_ty.is_ptr_or_array()
                {
                    // 両方ともスカラー型の場合、結果型はint型とする
                    self.ty = Some(Box::new(Type::new(&TypeKind::Int)));
                } else {
                    return Err(CompileError::InvalidExpression {
                        msg: format!(
                            "比較演算子はスカラー型またはポインタ/配列型にのみ適用可能です: {:?} と {:?}",
                            lhs_ty, rhs_ty
                        ),
                    });
                }
            }
            NodeKind::LogicalAnd | NodeKind::LogicalOr => {
                let lhs_ty = self.lhs.as_ref().unwrap().ty.as_ref().unwrap();
                let rhs_ty = self.rhs.as_ref().unwrap().ty.as_ref().unwrap();

                if lhs_ty.is_scalar() && rhs_ty.is_scalar()
                    || lhs_ty.is_ptr_or_array() && rhs_ty.is_ptr_or_array()
                {
                    // 両方ともスカラー型の場合、結果型はint型とする
                    self.ty = Some(Box::new(Type::new(&TypeKind::Int)));
                } else {
                    return Err(CompileError::InvalidExpression {
                        msg: format!(
                            "論理演算子はスカラー型またはポインタ/配列型にのみ適用可能です: {:?} と {:?}",
                            lhs_ty, rhs_ty
                        ),
                    });
                }
            }
            NodeKind::Ternary {
                ref mut cond,
                ref mut then,
                ref mut els,
            } => {
                cond.as_mut().unwrap().assign_types()?;
                then.as_mut().unwrap().assign_types()?;
                els.as_mut().unwrap().assign_types()?;

                let cond_ty = cond.as_ref().unwrap().ty.as_ref().unwrap();
                let then_ty = then.as_ref().unwrap().ty.as_ref().unwrap();
                let els_ty = els.as_ref().unwrap().ty.as_ref().unwrap();

                if cond_ty.is_scalar() || cond_ty.is_ptr_or_array() {
                    if then_ty == els_ty {
                        // then節とelse節の型が同じ場合、その型を結果型とする
                        self.ty = Some(then_ty.clone());
                    } else if then_ty.is_scalar() && els_ty.is_scalar() {
                        // 両方ともスカラー型の場合、大きい方の型に合わせる
                        if then_ty.size_of() >= els_ty.size_of() {
                            self.ty = Some(then_ty.clone());
                        } else {
                            self.ty = Some(els_ty.clone());
                        }
                    } else {
                        return Err(CompileError::InvalidExpression {
                            msg: format!(
                                "条件演算子のthen節とelse節は同じ型か、両方ともスカラー型である必要があります: {:?} と {:?}",
                                then_ty, els_ty
                            ),
                        });
                    }
                } else {
                    return Err(CompileError::InvalidExpression {
                        msg: format!(
                            "条件演算子の条件式はスカラー型にのみ適用可能です: {:?}",
                            cond_ty
                        ),
                    });
                }
            }
            NodeKind::Assign
            | NodeKind::AddAssign
            | NodeKind::SubAssign
            | NodeKind::MulAssign
            | NodeKind::DivAssign
            | NodeKind::RemAssign
            | NodeKind::ShlAssign
            | NodeKind::ShrAssign
            | NodeKind::BitAndAssign
            | NodeKind::BitOrAssign
            | NodeKind::BitXorAssign => {
                let lhs_ty = self.lhs.as_ref().unwrap().ty.as_ref().unwrap();

                // 代入演算子の型は左辺の型とする
                self.ty = Some(lhs_ty.clone());
            }
            NodeKind::BitNot => {
                let lhs_ty = self.lhs.as_ref().unwrap().ty.as_ref().unwrap();

                if lhs_ty.is_integer() {
                    self.ty = Some(Box::new(Type::new(&TypeKind::Int))); // 整数拡張
                } else {
                    return Err(CompileError::InvalidExpression {
                        msg: format!("ビット否定演算子は整数型にのみ適用可能です: {:?}", lhs_ty),
                    });
                }
            }
            NodeKind::LogicalNot => {
                let lhs_ty = self.lhs.as_ref().unwrap().ty.as_ref().unwrap();

                if lhs_ty.is_scalar() || lhs_ty.is_ptr_or_array() {
                    self.ty = Some(Box::new(Type::new(&TypeKind::Int))); // 結果型はint型
                } else {
                    return Err(CompileError::InvalidExpression {
                        msg: format!(
                            "論理否定演算子はスカラー型またはポインタ/配列型にのみ適用可能です: {:?}",
                            lhs_ty
                        ),
                    });
                }
            }
            NodeKind::Addr => {
                let lhs_ty = self.lhs.as_ref().unwrap().ty.as_ref().unwrap();

                // アドレス演算子の型はポインタ型にする
                self.ty = Some(Box::new(Type::new(&TypeKind::Ptr { to: lhs_ty.clone() })));
            }
            NodeKind::Deref => {
                let lhs_ty = self.lhs.as_ref().unwrap().ty.as_ref().unwrap();

                // デリファレンス演算子の型はポインタの指す型にする
                if !lhs_ty.is_ptr_or_array() {
                    return Err(CompileError::InvalidExpression {
                        msg: format!(
                            "デリファレンス演算子はポインタ/配列型にのみ適用可能です: {:?}",
                            lhs_ty
                        ),
                    });
                }
                self.ty = Some(Box::new(lhs_ty.base_type().clone()));
            }
            NodeKind::PreInc | NodeKind::PreDec | NodeKind::PostInc | NodeKind::PostDec => {
                let lhs_ty = self.lhs.as_ref().unwrap().ty.as_ref().unwrap();

                // インクリメント・デクリメント演算子の型はオペランドの型とする
                self.ty = Some(lhs_ty.clone());
            }
            _ => {
                // その他のノードは型を設定しない
            }
        }
        Ok(())
    }
}
