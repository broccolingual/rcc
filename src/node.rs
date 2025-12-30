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
    pub(crate) span: (usize, usize), // 開始位置と終了位置
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
            span: (0, 0),
        }
    }
}

impl Node {
    pub(crate) fn new(kind: NodeKind, span: (usize, usize)) -> Self {
        Node {
            kind,
            ty: Type::default(),
            span,
        }
    }

    pub(crate) fn new_call(
        name: &str,
        args: Vec<Node>,
        return_ty: Type,
        span: (usize, usize),
    ) -> Self {
        Node {
            kind: NodeKind::Call {
                name: name.to_string(),
                args,
            },
            ty: return_ty,
            span,
        }
    }

    pub(crate) fn new_binary(
        op: BinaryOp,
        lhs: Box<Node>,
        rhs: Box<Node>,
        span: (usize, usize),
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
                        span,
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
                        span,
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
                        span,
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
                        span,
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
                        span,
                    });
                }
            }
            BinaryOp::Assign => lhs.ty.clone(),
        };

        Ok(Node {
            kind: NodeKind::BinaryOp { op, lhs, rhs },
            ty,
            span,
        })
    }

    pub(crate) fn new_unary(
        op: UnaryOp,
        expr: Box<Node>,
        span: (usize, usize),
    ) -> Result<Self, CompileError> {
        let ty = match op {
            UnaryOp::BitNot => {
                let expr_ty = &expr.ty;

                if expr_ty.is_integer() {
                    Type::from(TypeKind::Int, false) // 整数拡張
                } else {
                    return Err(CompileError::InvalidExpression {
                        msg: format!("ビット否定演算子は整数型にのみ適用可能です: {:?}", expr_ty),
                        span,
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
                        span,
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
                        span,
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
            span,
        })
    }

    pub(crate) fn new_assign(
        op: BinaryOp,
        lhs: Box<Node>,
        rhs: Box<Node>,
        span: (usize, usize),
    ) -> Self {
        let ty = lhs.ty.clone(); // 代入演算子の型は左辺の型とする

        Node {
            kind: NodeKind::Assign { op, lhs, rhs },
            ty,
            span,
        }
    }

    pub(crate) fn new_logical_and(
        lhs: Box<Node>,
        rhs: Box<Node>,
        span: (usize, usize),
    ) -> Result<Self, CompileError> {
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
                span,
            });
        };

        Ok(Node {
            kind: NodeKind::LogicalAnd { lhs, rhs },
            ty,
            span,
        })
    }

    pub(crate) fn new_logical_or(
        lhs: Box<Node>,
        rhs: Box<Node>,
        span: (usize, usize),
    ) -> Result<Self, CompileError> {
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
                span,
            });
        };

        Ok(Node {
            kind: NodeKind::LogicalOr { lhs, rhs },
            ty,
            span,
        })
    }

    pub(crate) fn new_ternary(
        cond: Box<Node>,
        then: Box<Node>,
        els: Box<Node>,
        span: (usize, usize),
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
                    span,
                });
            }
        } else {
            return Err(CompileError::InvalidExpression {
                msg: format!(
                    "条件演算子の条件式はスカラー型にのみ適用可能です: {:?}",
                    cond_ty
                ),
                span,
            });
        };

        Ok(Node {
            kind: NodeKind::Ternary { cond, then, els },
            ty,
            span,
        })
    }

    pub(crate) fn new_num(val: i64, span: (usize, usize)) -> Self {
        Node {
            kind: NodeKind::Number { val },
            ty: Type::from(TypeKind::Int, false),
            span,
        }
    }

    pub(crate) fn new_var(
        name: &str,
        offset: usize,
        ty: &Type,
        is_local: bool,
        span: (usize, usize),
    ) -> Self {
        Node {
            kind: NodeKind::Var {
                name: name.to_string(),
                offset,
                is_local,
            },
            ty: ty.clone(),
            span,
        }
    }

    pub(crate) fn new_member(
        obj: Box<Node>,
        name: &str,
        offset: usize,
        ty: &Type,
        span: (usize, usize),
    ) -> Self {
        Node {
            kind: NodeKind::Member {
                obj,
                name: name.to_string(),
                offset,
            },
            ty: ty.clone(),
            span,
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
                        span: expr.span,
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
                                span: rhs.span,
                            });
                        }
                        Ok(lval / rval)
                    }
                    BinaryOp::Rem => {
                        if rval == 0 {
                            return Err(CompileError::InvalidExpression {
                                msg: "定数式の剰余演算でゼロ除算が発生しました".to_string(),
                                span: rhs.span,
                            });
                        }
                        Ok(lval % rval)
                    }
                    BinaryOp::Shl => {
                        if !(0..64).contains(&rval) {
                            return Err(CompileError::InvalidExpression {
                                msg: "定数式の左シフト演算で不正なシフト量が指定されました"
                                    .to_string(),
                                span: rhs.span,
                            });
                        }
                        Ok(lval << rval)
                    }
                    BinaryOp::Shr => {
                        if !(0..64).contains(&rval) {
                            return Err(CompileError::InvalidExpression {
                                msg: "定数式の右シフト演算で不正なシフト量が指定されました"
                                    .to_string(),
                                span: rhs.span,
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
                        span: self.span,
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
                        span: self.span,
                    })
                } else {
                    // TODO: グローバル変数の定数式評価
                    unimplemented!("グローバル変数の定数式評価は未実装です");
                }
            }
            _ => Err(CompileError::InvalidExpression {
                msg: "定数式に不正なノードが含まれています".to_string(),
                span: self.span,
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

    pub(crate) fn new_scaled_add(
        lhs: Box<Node>,
        rhs: Box<Node>,
        span: (usize, usize),
    ) -> Result<Self, CompileError> {
        // 左辺がポインタ/配列、右辺がスカラーの場合: ptr + n -> ptr + (n * sizeof(*ptr))
        if (lhs.ty.is_ptr() || lhs.ty.is_array()) && rhs.ty.is_scalar() {
            let base_size = lhs.ty.base_type().size_of();
            let scaled_rhs = Box::new(Node::new_binary(
                BinaryOp::Mul,
                rhs,
                Box::new(Node::new_num(base_size as i64, span)),
                span,
            )?);
            Node::new_binary(BinaryOp::Add, lhs, scaled_rhs, span)
        }
        // 右辺がポインタ/配列、左辺がスカラーの場合: n + ptr -> (n * sizeof(*ptr)) + ptr
        else if lhs.ty.is_scalar() && (rhs.ty.is_ptr() || rhs.ty.is_array()) {
            let base_size = rhs.ty.base_type().size_of();
            let scaled_lhs = Box::new(Node::new_binary(
                BinaryOp::Mul,
                lhs,
                Box::new(Node::new_num(base_size as i64, span)),
                span,
            )?);
            Node::new_binary(BinaryOp::Add, scaled_lhs, rhs, span)
        }
        // 両方ともスカラーの場合: 通常の加算
        else {
            Node::new_binary(BinaryOp::Add, lhs, rhs, span)
        }
    }

    pub(crate) fn new_scaled_sub(
        lhs: Box<Node>,
        rhs: Box<Node>,
        span: (usize, usize),
    ) -> Result<Self, CompileError> {
        // ポインタ同士の減算: ptr1 - ptr2 -> (ptr1 - ptr2) / sizeof(*ptr)
        if (lhs.ty.is_ptr() || lhs.ty.is_array()) && (rhs.ty.is_ptr() || rhs.ty.is_array()) {
            let base_size = lhs.ty.base_type().size_of();
            let sub_result = Box::new(Node::new_binary(BinaryOp::Sub, lhs, rhs, span)?);
            // 結果をsizeofで割って要素数の差にする
            Node::new_binary(
                BinaryOp::Div,
                sub_result,
                Box::new(Node::new_num(base_size as i64, span)),
                span,
            )
        }
        // 左辺がポインタ/配列、右辺がスカラーの場合: ptr - n -> ptr - (n * sizeof(*ptr))
        else if (lhs.ty.is_ptr() || lhs.ty.is_array()) && rhs.ty.is_scalar() {
            let base_size = lhs.ty.base_type().size_of();
            let scaled_rhs = Box::new(Node::new_binary(
                BinaryOp::Mul,
                rhs,
                Box::new(Node::new_num(base_size as i64, span)),
                span,
            )?);
            Node::new_binary(BinaryOp::Sub, lhs, scaled_rhs, span)
        }
        // 両方ともスカラーの場合: 通常の減算
        else {
            Node::new_binary(BinaryOp::Sub, lhs, rhs, span)
        }
    }

    pub(crate) fn new_scaled_increment(
        expr: Box<Node>,
        is_pre: bool,
        span: (usize, usize),
    ) -> Result<Self, CompileError> {
        if expr.ty.is_ptr() || expr.ty.is_array() {
            let base_size = expr.ty.base_type().size_of();
            let assign_node = Box::new(Node::new_assign(
                BinaryOp::Add,
                expr,
                Box::new(Node::new_num(base_size as i64, span)),
                span,
            ));
            if is_pre {
                // 前置インクリメント: ++expr
                Ok(*assign_node)
            } else {
                // 後置インクリメント: expr++
                // 元の値を返すため、代入後にサイズ分減算
                Node::new_binary(
                    BinaryOp::Sub,
                    assign_node,
                    Box::new(Node::new_num(base_size as i64, span)),
                    span,
                )
            }
        } else {
            // スカラー型の通常のインクリメント
            let op = if is_pre {
                UnaryOp::PreInc
            } else {
                UnaryOp::PostInc
            };
            Node::new_unary(op, expr, span)
        }
    }

    pub(crate) fn new_scaled_decrement(
        expr: Box<Node>,
        is_pre: bool,
        span: (usize, usize),
    ) -> Result<Self, CompileError> {
        if expr.ty.is_ptr() || expr.ty.is_array() {
            let size = expr.ty.base_type().size_of();
            let assign_node = Box::new(Node::new_assign(
                BinaryOp::Sub,
                expr,
                Box::new(Node::new_num(size as i64, span)),
                span,
            ));

            if is_pre {
                // 前置デクリメント: --expr
                Ok(*assign_node)
            } else {
                // 後置デクリメント: expr--
                // 元の値を返すため、代入後にサイズ分加算
                Node::new_binary(
                    BinaryOp::Add,
                    assign_node,
                    Box::new(Node::new_num(size as i64, span)),
                    span,
                )
            }
        } else {
            // スカラー型の通常のデクリメント
            let op = if is_pre {
                UnaryOp::PreDec
            } else {
                UnaryOp::PostDec
            };
            Node::new_unary(op, expr, span)
        }
    }
}
