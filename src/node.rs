use core::{fmt, str};

use crate::errors::CompileError;
use crate::types::{Type, TypeKind};

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum BinaryOp {
    Assign, // =
    Add,    // +
    Sub,    // -
    Mul,    // *
    Div,    // /
    Rem,    // %
    Shl,    // <<
    Shr,    // >>
    BitAnd, // &
    BitXor, // ^
    BitOr,  // |
    Eq,     // ==
    Ne,     // !=
    Lt,     // <
    Le,     // <=
}

impl str::FromStr for BinaryOp {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // assignment operators
            "=" => Ok(BinaryOp::Assign),
            "*=" => Ok(BinaryOp::Mul),
            "/=" => Ok(BinaryOp::Div),
            "%=" => Ok(BinaryOp::Rem),
            "+=" => Ok(BinaryOp::Add),
            "-=" => Ok(BinaryOp::Sub),
            "<<=" => Ok(BinaryOp::Shl),
            ">>=" => Ok(BinaryOp::Shr),
            "&=" => Ok(BinaryOp::BitAnd),
            "^=" => Ok(BinaryOp::BitXor),
            "|=" => Ok(BinaryOp::BitOr),
            _ => Err(()),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum UnaryOp {
    BitNot,     // ~
    LogicalNot, // !
    Addr,       // &
    Deref,      // *
    PreInc,     // ++pre
    PreDec,     // --pre
    PostInc,    // post++
    PostDec,    // post--
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum NodeKind {
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
        body: Vec<Box<Node>>,
    }, // {}
    Call {
        name: String,
        args: Vec<Box<Node>>,
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
        index: i64,
    }, // 文字列リテラル
    Nop,      // 空命令
}

#[derive(Clone, PartialEq, Eq)]
pub struct Node {
    pub kind: NodeKind,
    pub ty: Option<Box<Type>>,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            NodeKind::Number { val } => {
                write!(f, ", val: {}", val)?;
            }
            NodeKind::Var {
                ref name,
                offset,
                is_local,
            } => {
                write!(
                    f,
                    ", ident: {}, offset: {}, is_local: {}",
                    name, offset, is_local
                )?;
            }
            NodeKind::Identifier { ref name } => {
                write!(f, ", ident: {}", name)?;
            }
            NodeKind::BinaryOp {
                ref op,
                ref lhs,
                ref rhs,
            } => {
                write!(f, ", op: {:?}, lhs: {:?}, rhs: {:?}", op, lhs, rhs)?;
            }
            NodeKind::UnaryOp { ref op, ref expr } => {
                write!(f, ", op: {:?}, expr: {:?}", op, expr)?;
            }
            NodeKind::Assign {
                ref op,
                ref lhs,
                ref rhs,
            } => {
                write!(f, ", op: {:?}, lhs: {:?}, rhs: {:?}", op, lhs, rhs)?;
            }
            NodeKind::Call { ref name, ref args } => {
                write!(f, ", name: {}, args: {:?}", name, args)?;
            }
            NodeKind::Label { ref name, ref expr } => {
                write!(f, ", name: {}", name)?;
                write!(f, ", expr: {:?}", expr)?;
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
            ty: None,
        }
    }
}

impl Node {
    pub fn new(kind: NodeKind) -> Self {
        Node { kind, ty: None }
    }

    pub fn new_binary(op: BinaryOp, lhs: Box<Node>, rhs: Box<Node>) -> Self {
        Node {
            kind: NodeKind::BinaryOp { op, lhs, rhs },
            ty: None,
        }
    }

    pub fn new_unary(op: UnaryOp, expr: Box<Node>) -> Self {
        Node {
            kind: NodeKind::UnaryOp { op, expr },
            ty: None,
        }
    }

    pub fn new_assign(op: BinaryOp, lhs: Box<Node>, rhs: Box<Node>) -> Self {
        Node {
            kind: NodeKind::Assign { op, lhs, rhs },
            ty: None,
        }
    }

    pub fn new_num(val: i64) -> Self {
        let mut node = Node::new(NodeKind::Number { val });
        node.ty = Some(Box::new(Type::from(&TypeKind::Int, false)));
        node
    }

    pub fn new_var(name: &str, offset: usize, ty: &Type, is_local: bool) -> Self {
        let mut node = Node::new(NodeKind::Var {
            name: name.to_string(),
            offset,
            is_local,
        });
        node.ty = Some(Box::new(ty.clone()));
        node
    }

    pub fn is_expr(&self) -> bool {
        match self.kind {
            // 値を返さない文
            NodeKind::If { .. }
            | NodeKind::While { .. }
            | NodeKind::For { .. }
            | NodeKind::Do { .. }
            | NodeKind::Block { .. }
            | NodeKind::Break
            | NodeKind::Continue
            | NodeKind::Goto { .. }
            | NodeKind::Label { .. }
            | NodeKind::Return { .. }
            | NodeKind::Nop => false,
            _ => true, // 値を返す式
        }
    }

    pub fn scaled_add(
        &mut self,
        mut rhs: Option<Box<Node>>,
    ) -> Result<Option<Box<Node>>, CompileError> {
        if let Some(ty) = &self.ty {
            if ty.is_ptr_or_array() {
                let base_size = ty.base_type().size_of();
                // ポインタ加算の場合、右辺をスケーリングする
                rhs = Some(Box::new(Node::new_binary(
                    BinaryOp::Mul,
                    rhs.unwrap(),
                    Box::new(Node::new_num(base_size as i64)),
                )));
            }
            Ok(Some(Box::new(Node::new_binary(
                BinaryOp::Add,
                Box::new(self.clone()),
                rhs.unwrap(),
            ))))
        } else {
            Err(CompileError::InvalidExpression {
                msg: "型情報が不足しています".to_string(),
            })
        }
    }

    pub fn scaled_sub(
        &mut self,
        mut rhs: Option<Box<Node>>,
    ) -> Result<Option<Box<Node>>, CompileError> {
        if let Some(ty) = &self.ty {
            if ty.is_ptr_or_array() {
                let base_size = ty.base_type().size_of();
                // ポインタ減算の場合、右辺をスケーリングする
                rhs = Some(Box::new(Node::new_binary(
                    BinaryOp::Mul,
                    rhs.unwrap(),
                    Box::new(Node::new_num(base_size as i64)),
                )));
            }
            Ok(Some(Box::new(Node::new_binary(
                BinaryOp::Sub,
                Box::new(self.clone()),
                rhs.unwrap(),
            ))))
        } else {
            Err(CompileError::InvalidExpression {
                msg: "型情報が不足しています".to_string(),
            })
        }
    }

    pub fn assign_types(&mut self) -> Result<(), CompileError> {
        match self.kind {
            NodeKind::BinaryOp {
                ref op,
                ref mut lhs,
                ref mut rhs,
            } => {
                lhs.assign_types()?;
                rhs.assign_types()?;
                match op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                        let lhs_ty = lhs.ty.as_ref().unwrap();
                        let rhs_ty = rhs.ty.as_ref().unwrap();

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
                    BinaryOp::Rem => {
                        let lhs_ty = lhs.ty.as_ref().unwrap();
                        let rhs_ty = rhs.ty.as_ref().unwrap();

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
                    BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor => {
                        let lhs_ty = lhs.ty.as_ref().unwrap();
                        let rhs_ty = rhs.ty.as_ref().unwrap();

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
                    BinaryOp::Shl | BinaryOp::Shr => {
                        let lhs_ty = lhs.ty.as_ref().unwrap();
                        let rhs_ty = rhs.ty.as_ref().unwrap();

                        if lhs_ty.is_integer() && rhs_ty.is_integer() {
                            // 両方とも整数型の場合、昇格後の型を結果型とする
                            self.ty = Some(Box::new(Type::from(&TypeKind::Int, false)));
                        } else {
                            return Err(CompileError::InvalidExpression {
                                msg: format!(
                                    "シフト演算子は整数型にのみ適用可能です: {:?} と {:?}",
                                    lhs_ty, rhs_ty
                                ),
                            });
                        }
                    }
                    BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le => {
                        let lhs_ty = lhs.ty.as_ref().unwrap();
                        let rhs_ty = rhs.ty.as_ref().unwrap();

                        if lhs_ty.is_scalar() && rhs_ty.is_scalar()
                            || lhs_ty.is_ptr_or_array() && rhs_ty.is_ptr_or_array()
                        {
                            // 両方ともスカラー型の場合、結果型はint型とする
                            self.ty = Some(Box::new(Type::from(&TypeKind::Int, false)));
                        } else {
                            return Err(CompileError::InvalidExpression {
                                msg: format!(
                                    "比較演算子はスカラー型またはポインタ/配列型にのみ適用可能です: {:?} と {:?}",
                                    lhs_ty, rhs_ty
                                ),
                            });
                        }
                    }
                    _ => {}
                }
            }
            NodeKind::UnaryOp {
                ref op,
                ref mut expr,
            } => {
                expr.assign_types()?;
                match op {
                    UnaryOp::BitNot => {
                        let expr_ty = expr.ty.as_ref().unwrap();

                        if expr_ty.is_integer() {
                            self.ty = Some(Box::new(Type::from(&TypeKind::Int, false))); // 整数拡張
                        } else {
                            return Err(CompileError::InvalidExpression {
                                msg: format!(
                                    "ビット否定演算子は整数型にのみ適用可能です: {:?}",
                                    expr_ty
                                ),
                            });
                        }
                    }
                    UnaryOp::LogicalNot => {
                        let expr_ty = expr.ty.as_ref().unwrap();

                        if expr_ty.is_scalar() || expr_ty.is_ptr_or_array() {
                            self.ty = Some(Box::new(Type::from(&TypeKind::Int, false))); // 結果型はint型
                        } else {
                            return Err(CompileError::InvalidExpression {
                                msg: format!(
                                    "論理否定演算子はスカラー型またはポインタ/配列型にのみ適用可能です: {:?}",
                                    expr_ty
                                ),
                            });
                        }
                    }
                    UnaryOp::Addr => {
                        let expr_ty = expr.ty.as_ref().unwrap();

                        // アドレス演算子の型はポインタ型にする
                        self.ty = Some(Box::new(Type::from(
                            &TypeKind::Ptr {
                                to: expr_ty.clone(),
                            },
                            false,
                        )));
                    }
                    UnaryOp::Deref => {
                        let expr_ty = expr.ty.as_ref().unwrap();

                        // デリファレンス演算子の型はポインタの指す型にする
                        if !expr_ty.is_ptr_or_array() {
                            return Err(CompileError::InvalidExpression {
                                msg: format!(
                                    "デリファレンス演算子はポインタ/配列型にのみ適用可能です: {:?}",
                                    expr_ty
                                ),
                            });
                        }
                        self.ty = Some(Box::new(expr_ty.base_type().clone()));
                    }
                    UnaryOp::PreInc | UnaryOp::PreDec | UnaryOp::PostInc | UnaryOp::PostDec => {
                        let expr_ty = expr.ty.as_ref().unwrap();

                        // インクリメント・デクリメント演算子の型はオペランドの型とする
                        self.ty = Some(expr_ty.clone());
                    }
                }
            }
            NodeKind::Assign {
                ref mut lhs,
                ref mut rhs,
                ..
            } => {
                lhs.assign_types()?;
                rhs.assign_types()?;
                let lhs_ty = lhs.ty.as_ref().unwrap();

                self.ty = Some(lhs_ty.clone()); // 代入演算子の型は左辺の型とする
            }
            NodeKind::LogicalAnd {
                ref mut lhs,
                ref mut rhs,
            }
            | NodeKind::LogicalOr {
                ref mut lhs,
                ref mut rhs,
            } => {
                lhs.assign_types()?;
                rhs.assign_types()?;
                let lhs_ty = lhs.ty.as_ref().unwrap();
                let rhs_ty = rhs.ty.as_ref().unwrap();

                if lhs_ty.is_scalar() && rhs_ty.is_scalar()
                    || lhs_ty.is_ptr_or_array() && rhs_ty.is_ptr_or_array()
                {
                    // 両方ともスカラー型の場合、結果型はint型とする
                    self.ty = Some(Box::new(Type::from(&TypeKind::Int, false)));
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
                cond.assign_types()?;
                then.assign_types()?;
                els.assign_types()?;

                let cond_ty = cond.ty.as_ref().unwrap();
                let then_ty = then.ty.as_ref().unwrap();
                let els_ty = els.ty.as_ref().unwrap();
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
            NodeKind::Number { .. } => {
                // 数値リテラルの型はすでに設定されているはず
            }
            NodeKind::Var { .. } => {
                // 変数の型はすでに設定されているはず
            }
            _ => {
                // その他のノードは型を設定しない
            }
        }
        Ok(())
    }
}
