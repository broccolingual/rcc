mod kind;
mod operator;

pub(crate) use kind::*;
pub(crate) use operator::*;

use crate::errors::CompileError;
use crate::types::{Type, TypeKind};
use core::fmt;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Node {
    pub(crate) kind: NodeKind,
    pub(crate) ty: Type,
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
            ty: Type::default(),
        }
    }
}

impl Node {
    pub(crate) fn new(kind: NodeKind) -> Self {
        Node {
            kind,
            ty: Type::default(),
        }
    }

    pub(crate) fn new_call(name: &str, args: Vec<Node>, return_ty: Type) -> Self {
        Node {
            kind: NodeKind::Call {
                name: name.to_string(),
                args,
            },
            ty: return_ty,
        }
    }

    pub(crate) fn new_binary(
        op: BinaryOp,
        lhs: Box<Node>,
        rhs: Box<Node>,
    ) -> Result<Self, CompileError> {
        let ty = match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                let lhs_ty = &lhs.ty;
                let rhs_ty = &rhs.ty;

                if lhs_ty.is_scalar() && rhs_ty.is_scalar() {
                    // 両方ともスカラー型の場合、大きい方の型に合わせる
                    if lhs_ty.size_of() >= rhs_ty.size_of() {
                        lhs_ty.clone()
                    } else {
                        rhs_ty.clone()
                    }
                } else if (lhs_ty.is_ptr() || lhs_ty.is_array()) && rhs_ty.is_scalar() {
                    // 左辺がポインタ/配列型、右辺がスカラー型の場合、左辺の型を結果型とする
                    lhs_ty.clone()
                } else if lhs_ty.is_scalar() && (rhs_ty.is_ptr() || rhs_ty.is_array()) {
                    // 右辺がポインタ/配列型、左辺がスカラー型の場合、右辺の型を結果型とする
                    rhs_ty.clone()
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
                let lhs_ty = &lhs.ty;
                let rhs_ty = &rhs.ty;

                if lhs_ty.is_integer() && rhs_ty.is_integer() {
                    // 両方とも整数型の場合、大きい方の型に合わせる
                    if lhs_ty.size_of() >= rhs_ty.size_of() {
                        lhs_ty.clone()
                    } else {
                        rhs_ty.clone()
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
                let lhs_ty = &lhs.ty;
                let rhs_ty = &rhs.ty;

                if lhs_ty.is_integer() && rhs_ty.is_integer() {
                    // 両方とも整数型の場合、大きい方の型に合わせる
                    if lhs_ty.size_of() >= rhs_ty.size_of() {
                        lhs_ty.clone()
                    } else {
                        rhs_ty.clone()
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
                let lhs_ty = &lhs.ty;
                let rhs_ty = &rhs.ty;

                if lhs_ty.is_integer() && rhs_ty.is_integer() {
                    // 両方とも整数型の場合、昇格後の型を結果型とする
                    Type::from(TypeKind::Int, false)
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
                let lhs_ty = &lhs.ty;
                let rhs_ty = &rhs.ty;

                if lhs_ty.is_scalar() && rhs_ty.is_scalar()
                    || (lhs_ty.is_ptr() || lhs_ty.is_array())
                        && (rhs_ty.is_ptr() || rhs_ty.is_array())
                {
                    // 両方ともスカラー型の場合、結果型はint型とする
                    Type::from(TypeKind::Int, false)
                } else {
                    return Err(CompileError::InvalidExpression {
                        msg: format!(
                            "比較演算子はスカラー型またはポインタ/配列型にのみ適用可能です: {:?} と {:?}",
                            lhs_ty, rhs_ty
                        ),
                    });
                }
            }
            BinaryOp::Assign => lhs.ty.clone(),
        };

        Ok(Node {
            kind: NodeKind::BinaryOp { op, lhs, rhs },
            ty,
        })
    }

    pub(crate) fn new_unary(op: UnaryOp, expr: Box<Node>) -> Result<Self, CompileError> {
        let ty = match op {
            UnaryOp::BitNot => {
                let expr_ty = &expr.ty;

                if expr_ty.is_integer() {
                    Type::from(TypeKind::Int, false) // 整数拡張
                } else {
                    return Err(CompileError::InvalidExpression {
                        msg: format!("ビット否定演算子は整数型にのみ適用可能です: {:?}", expr_ty),
                    });
                }
            }
            UnaryOp::LogicalNot => {
                let expr_ty = &expr.ty;

                if expr_ty.is_scalar() || expr_ty.is_ptr() || expr_ty.is_array() {
                    Type::from(TypeKind::Int, false) // 結果型はint型
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
                let expr_ty = &expr.ty;

                // アドレス演算子の型はポインタ型にする
                Type::from(
                    TypeKind::Ptr {
                        to: Box::new(expr_ty.clone()),
                    },
                    false,
                )
            }
            UnaryOp::Deref => {
                let expr_ty = &expr.ty;

                // デリファレンス演算子の型はポインタの指す型にする
                if !(expr_ty.is_ptr() || expr_ty.is_array()) {
                    return Err(CompileError::InvalidExpression {
                        msg: format!(
                            "デリファレンス演算子はポインタ/配列型にのみ適用可能です: {:?}",
                            expr_ty
                        ),
                    });
                }
                expr_ty.base_type().clone()
            }
            UnaryOp::PreInc | UnaryOp::PreDec | UnaryOp::PostInc | UnaryOp::PostDec => {
                let expr_ty = &expr.ty;

                // インクリメント・デクリメント演算子の型はオペランドの型とする
                expr_ty.clone()
            }
        };

        Ok(Node {
            kind: NodeKind::UnaryOp { op, expr },
            ty,
        })
    }

    pub(crate) fn new_assign(op: BinaryOp, lhs: Box<Node>, rhs: Box<Node>) -> Self {
        let ty = lhs.ty.clone(); // 代入演算子の型は左辺の型とする

        Node {
            kind: NodeKind::Assign { op, lhs, rhs },
            ty,
        }
    }

    pub(crate) fn new_logical_and(lhs: Box<Node>, rhs: Box<Node>) -> Result<Self, CompileError> {
        let lhs_ty = &lhs.ty;
        let rhs_ty = &rhs.ty;

        let ty = if lhs_ty.is_scalar() && rhs_ty.is_scalar()
            || (lhs_ty.is_ptr() || lhs_ty.is_array()) && (rhs_ty.is_ptr() || rhs_ty.is_array())
        {
            // 両方ともスカラー型の場合、結果型はint型とする
            Type::from(TypeKind::Int, false)
        } else {
            return Err(CompileError::InvalidExpression {
                msg: format!(
                    "論理演算子はスカラー型またはポインタ/配列型にのみ適用可能です: {:?} と {:?}",
                    lhs_ty, rhs_ty
                ),
            });
        };

        Ok(Node {
            kind: NodeKind::LogicalAnd { lhs, rhs },
            ty,
        })
    }

    pub(crate) fn new_logical_or(lhs: Box<Node>, rhs: Box<Node>) -> Result<Self, CompileError> {
        let lhs_ty = &lhs.ty;
        let rhs_ty = &rhs.ty;

        let ty = if lhs_ty.is_scalar() && rhs_ty.is_scalar()
            || (lhs_ty.is_ptr() || lhs_ty.is_array()) && (rhs_ty.is_ptr() || rhs_ty.is_array())
        {
            // 両方ともスカラー型の場合、結果型はint型とする
            Type::from(TypeKind::Int, false)
        } else {
            return Err(CompileError::InvalidExpression {
                msg: format!(
                    "論理演算子はスカラー型またはポインタ/配列型にのみ適用可能です: {:?} と {:?}",
                    lhs_ty, rhs_ty
                ),
            });
        };

        Ok(Node {
            kind: NodeKind::LogicalOr { lhs, rhs },
            ty,
        })
    }

    pub(crate) fn new_ternary(
        cond: Box<Node>,
        then: Box<Node>,
        els: Box<Node>,
    ) -> Result<Self, CompileError> {
        let cond_ty = &cond.ty;
        let then_ty = &then.ty;
        let els_ty = &els.ty;

        let ty = if cond_ty.is_scalar() || cond_ty.is_ptr() || cond_ty.is_array() {
            if then_ty == els_ty {
                // then節とelse節の型が同じ場合、その型を結果型とする
                then_ty.clone()
            } else if then_ty.is_scalar() && els_ty.is_scalar() {
                // 両方ともスカラー型の場合、大きい方の型に合わせる
                if then_ty.size_of() >= els_ty.size_of() {
                    then_ty.clone()
                } else {
                    els_ty.clone()
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
        };

        Ok(Node {
            kind: NodeKind::Ternary { cond, then, els },
            ty,
        })
    }

    pub(crate) fn new_num(val: i64) -> Self {
        let mut node = Node::new(NodeKind::Number { val });
        node.ty = Type::from(TypeKind::Int, false);
        node
    }

    pub(crate) fn new_var(name: &str, offset: usize, ty: &Type, is_local: bool) -> Self {
        Node {
            kind: NodeKind::Var {
                name: name.to_string(),
                offset,
                is_local,
            },
            ty: ty.clone(),
        }
    }

    pub(crate) fn new_member(obj: Box<Node>, name: &str, offset: usize, ty: &Type) -> Self {
        Node {
            kind: NodeKind::Member {
                obj,
                name: name.to_string(),
                offset,
            },
            ty: ty.clone(),
        }
    }

    // 定数式を評価して、その値を返す
    pub(crate) fn eval_const_expr(&self) -> Result<i64, CompileError> {
        match &self.kind {
            NodeKind::Number { val } => Ok(*val),
            NodeKind::UnaryOp { op, expr } => {
                let val = expr.eval_const_expr()?;
                match op {
                    UnaryOp::BitNot => Ok(!val),
                    UnaryOp::LogicalNot => Ok(if val == 0 { 1 } else { 0 }),
                    _ => Err(CompileError::InvalidExpression {
                        msg: format!("定数式に不正な単項演算子が含まれています: {:?}", op),
                    }),
                }
            }
            NodeKind::BinaryOp { op, lhs, rhs } => {
                let lval = lhs.eval_const_expr()?;
                let rval = rhs.eval_const_expr()?;
                match op {
                    BinaryOp::Add => Ok(lval + rval),
                    BinaryOp::Sub => Ok(lval - rval),
                    BinaryOp::Mul => Ok(lval * rval),
                    BinaryOp::Div => {
                        if rval == 0 {
                            return Err(CompileError::InvalidExpression {
                                msg: "定数式の除算でゼロ除算が発生しました".to_string(),
                            });
                        }
                        Ok(lval / rval)
                    }
                    BinaryOp::Rem => {
                        if rval == 0 {
                            return Err(CompileError::InvalidExpression {
                                msg: "定数式の剰余演算でゼロ除算が発生しました".to_string(),
                            });
                        }
                        Ok(lval % rval)
                    }
                    BinaryOp::Shl => {
                        if !(0..64).contains(&rval) {
                            return Err(CompileError::InvalidExpression {
                                msg: "定数式の左シフト演算で不正なシフト量が指定されました"
                                    .to_string(),
                            });
                        }
                        Ok(lval << rval)
                    }
                    BinaryOp::Shr => {
                        if !(0..64).contains(&rval) {
                            return Err(CompileError::InvalidExpression {
                                msg: "定数式の右シフト演算で不正なシフト量が指定されました"
                                    .to_string(),
                            });
                        }
                        Ok(lval >> rval)
                    }
                    BinaryOp::BitAnd => Ok(lval & rval),
                    BinaryOp::BitOr => Ok(lval | rval),
                    BinaryOp::BitXor => Ok(lval ^ rval),
                    BinaryOp::Eq => Ok(if lval == rval { 1 } else { 0 }),
                    BinaryOp::Ne => Ok(if lval != rval { 1 } else { 0 }),
                    BinaryOp::Lt => Ok(if lval < rval { 1 } else { 0 }),
                    BinaryOp::Le => Ok(if lval <= rval { 1 } else { 0 }),
                    _ => Err(CompileError::InvalidExpression {
                        msg: format!("定数式に不正な二項演算子が含まれています: {:?}", op),
                    }),
                }
            }
            NodeKind::Ternary { cond, then, els } => {
                let cond_val = cond.eval_const_expr()?;
                if cond_val != 0 {
                    then.eval_const_expr()
                } else {
                    els.eval_const_expr()
                }
            }
            NodeKind::LogicalAnd { lhs, rhs } => {
                let lval = lhs.eval_const_expr()?;
                if lval == 0 {
                    Ok(0)
                } else {
                    let rval = rhs.eval_const_expr()?;
                    Ok(if rval != 0 { 1 } else { 0 })
                }
            }
            NodeKind::LogicalOr { lhs, rhs } => {
                let lval = lhs.eval_const_expr()?;
                if lval != 0 {
                    Ok(1)
                } else {
                    let rval = rhs.eval_const_expr()?;
                    Ok(if rval != 0 { 1 } else { 0 })
                }
            }
            NodeKind::Var { name, is_local, .. } => {
                if *is_local {
                    Err(CompileError::InvalidExpression {
                        msg: format!("定数式にローカル変数 '{}' が含まれています", name),
                    })
                } else {
                    // TODO: グローバル変数の定数式評価
                    unimplemented!("グローバル変数の定数式評価は未実装です");
                }
            }
            _ => Err(CompileError::InvalidExpression {
                msg: "定数式に不正なノードが含まれています".to_string(),
            }),
        }
    }

    pub(crate) fn is_expr(&self) -> bool {
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

    pub(crate) fn new_scaled_add(lhs: Box<Node>, rhs: Box<Node>) -> Result<Self, CompileError> {
        let rhs = if lhs.ty.is_ptr() || lhs.ty.is_array() {
            let base_size = lhs.ty.base_type().size_of();
            // ポインタ加算の場合、右辺をスケーリングする
            Box::new(Node::new_binary(
                BinaryOp::Mul,
                rhs,
                Box::new(Node::new_num(base_size as i64)),
            )?)
        } else {
            rhs
        };
        Node::new_binary(BinaryOp::Add, lhs, rhs)
    }

    pub(crate) fn new_scaled_sub(lhs: Box<Node>, rhs: Box<Node>) -> Result<Self, CompileError> {
        let rhs = if lhs.ty.is_ptr() || lhs.ty.is_array() {
            let base_size = lhs.ty.base_type().size_of();
            // ポインタ減算の場合、右辺をスケーリングする
            Box::new(Node::new_binary(
                BinaryOp::Mul,
                rhs,
                Box::new(Node::new_num(base_size as i64)),
            )?)
        } else {
            rhs
        };
        Node::new_binary(BinaryOp::Sub, lhs, rhs)
    }
}
