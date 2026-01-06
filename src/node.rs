mod kind;
mod operator;

pub(crate) use kind::*;
pub(crate) use operator::*;

use crate::errors::CompileError;
use crate::span::Span;
use crate::symbol::SymbolId;
use crate::types::{TypeAttr, TypeKind, TypeRef};
use core::fmt;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Node {
    pub(crate) kind: NodeKind,
    pub(crate) ty: TypeRef,
    pub(crate) span: Span, // 開始位置と終了位置
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            NodeKind::Number { val } => {
                write!(f, ", val: {}", val)?;
            }
            NodeKind::Var { symbol_id } => {
                write!(f, ", symbol_id: {:?}", symbol_id)?;
            }
            NodeKind::Ident { name } => {
                write!(f, ", ident: {}", name)?;
            }
            NodeKind::BinaryOp { op, lhs, rhs } => {
                write!(f, ", op: {:?}, lhs: {:?}, rhs: {:?}", op, lhs, rhs)?;
            }
            NodeKind::UnaryOp { op, expr } => {
                write!(f, ", op: {:?}, expr: {:?}", op, expr)?;
            }
            NodeKind::Assign { op, lhs, rhs } => {
                write!(f, ", op: {:?}, lhs: {:?}, rhs: {:?}", op, lhs, rhs)?;
            }
            NodeKind::Call { name, args } => {
                write!(f, ", name: {}, args: {:?}", name, args)?;
            }
            NodeKind::Label { name, expr } => {
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
            ty: TypeRef::default(),
            span: Span::default(),
        }
    }
}

impl Node {
    pub(crate) fn new(kind: NodeKind, span: Span) -> Self {
        Node {
            kind,
            ty: TypeRef::default(),
            span,
        }
    }

    pub(crate) fn new_call(name: &str, args: Vec<Node>, return_ty: TypeRef, span: Span) -> Self {
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
        span: Span,
    ) -> Result<Self, CompileError> {
        let ty = match op {
            BinaryOp::Add => {
                let lhs_ty = lhs.ty;
                let rhs_ty = rhs.ty;

                if (lhs_ty.is_integer() || lhs_ty.is_floating_point())
                    && (rhs_ty.is_integer() || rhs_ty.is_floating_point())
                {
                    // 整数/浮動小数点 + 整数/浮動小数点: 大きい方の型に合わせる
                    if lhs_ty.size_of() >= rhs_ty.size_of() {
                        lhs_ty
                    } else {
                        rhs_ty
                    }
                } else if (lhs_ty.is_ptr() || lhs_ty.is_array()) && rhs_ty.is_integer() {
                    // ポインタ + 整数: ポインタ型を返す
                    lhs_ty
                } else if lhs_ty.is_integer() && (rhs_ty.is_ptr() || rhs_ty.is_array()) {
                    // 整数 + ポインタ: ポインタ型を返す
                    rhs_ty
                } else {
                    return Err(CompileError::InvalidExpr {
                        msg: format!(
                            "加算演算子は、算術型同士、またはポインタと整数の組み合わせにのみ適用可能です: {:?} と {:?}",
                            lhs_ty, rhs_ty
                        ),
                        span,
                    });
                }
            }
            BinaryOp::Sub => {
                let lhs_ty = lhs.ty;
                let rhs_ty = rhs.ty;

                if (lhs_ty.is_ptr() || lhs_ty.is_array()) && (rhs_ty.is_ptr() || rhs_ty.is_array())
                {
                    // ポインタ - ポインタ: 整数型（要素数の差）を返す
                    TypeRef::register(TypeKind::Int, TypeAttr::default(), None)
                } else if (lhs_ty.is_ptr() || lhs_ty.is_array()) && rhs_ty.is_integer() {
                    // ポインタ - 整数: ポインタ型を返す
                    lhs_ty
                } else if (lhs_ty.is_integer() || lhs_ty.is_floating_point())
                    && (rhs_ty.is_integer() || rhs_ty.is_floating_point())
                {
                    // 整数/浮動小数点 - 整数/浮動小数点: 大きい方の型に合わせる
                    if lhs_ty.size_of() >= rhs_ty.size_of() {
                        lhs_ty
                    } else {
                        rhs_ty
                    }
                } else {
                    return Err(CompileError::InvalidExpr {
                        msg: format!(
                            "減算演算子は、算術型同士、ポインタ-整数、またはポインタ同士にのみ適用可能です: {:?} と {:?}",
                            lhs_ty, rhs_ty
                        ),
                        span,
                    });
                }
            }
            BinaryOp::Mul | BinaryOp::Div => {
                let lhs_ty = lhs.ty;
                let rhs_ty = rhs.ty;

                if (lhs_ty.is_integer() || lhs_ty.is_floating_point())
                    && (rhs_ty.is_integer() || rhs_ty.is_floating_point())
                {
                    // 整数/浮動小数点 * 整数/浮動小数点、整数/浮動小数点 / 整数/浮動小数点: 大きい方の型に合わせる
                    if lhs_ty.size_of() >= rhs_ty.size_of() {
                        lhs_ty
                    } else {
                        rhs_ty
                    }
                } else {
                    return Err(CompileError::InvalidExpr {
                        msg: format!(
                            "乗算・除算演算子は整数型または浮動小数点型にのみ適用可能です: {:?} と {:?}",
                            lhs_ty, rhs_ty
                        ),
                        span,
                    });
                }
            }
            BinaryOp::Rem => {
                let lhs_ty = lhs.ty;
                let rhs_ty = rhs.ty;

                if lhs_ty.is_integer() && rhs_ty.is_integer() {
                    // 両方とも整数型の場合、大きい方の型に合わせる
                    if lhs_ty.size_of() >= rhs_ty.size_of() {
                        lhs_ty
                    } else {
                        rhs_ty
                    }
                } else {
                    return Err(CompileError::InvalidExpr {
                        msg: format!(
                            "剰余演算子は整数型にのみ適用可能です: {:?} と {:?}",
                            lhs_ty, rhs_ty
                        ),
                        span,
                    });
                }
            }
            BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor => {
                let lhs_ty = lhs.ty;
                let rhs_ty = rhs.ty;

                if lhs_ty.is_integer() && rhs_ty.is_integer() {
                    // 両方とも整数型の場合、大きい方の型に合わせる
                    if lhs_ty.size_of() >= rhs_ty.size_of() {
                        lhs_ty
                    } else {
                        rhs_ty
                    }
                } else {
                    return Err(CompileError::InvalidExpr {
                        msg: format!(
                            "ビット演算子は整数型にのみ適用可能です: {:?} と {:?}",
                            lhs_ty, rhs_ty
                        ),
                        span,
                    });
                }
            }
            BinaryOp::Shl | BinaryOp::Shr => {
                let lhs_ty = lhs.ty;
                let rhs_ty = rhs.ty;

                if lhs_ty.is_integer() && rhs_ty.is_integer() {
                    // 両方とも整数型の場合、昇格後の型を結果型とする
                    TypeRef::register(TypeKind::Int, TypeAttr::default(), None)
                } else {
                    return Err(CompileError::InvalidExpr {
                        msg: format!(
                            "シフト演算子は整数型にのみ適用可能です: {:?} と {:?}",
                            lhs_ty, rhs_ty
                        ),
                        span,
                    });
                }
            }
            BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le => {
                let lhs_ty = lhs.ty;
                let rhs_ty = rhs.ty;

                if (lhs_ty.is_scalar() || lhs_ty.is_array())
                    && (rhs_ty.is_scalar() || rhs_ty.is_array())
                {
                    // 結果はint型
                    TypeRef::register(TypeKind::Int, TypeAttr::default(), None)
                } else {
                    return Err(CompileError::InvalidExpr {
                        msg: format!(
                            "比較演算子はスカラー型または配列型にのみ適用可能です: {:?} と {:?}",
                            lhs_ty, rhs_ty
                        ),
                        span,
                    });
                }
            }
            BinaryOp::Assign => lhs.ty,
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
        span: Span,
    ) -> Result<Self, CompileError> {
        let ty = match op {
            UnaryOp::BitNot => {
                let expr_ty = &expr.ty;

                if expr_ty.is_integer() {
                    TypeRef::register(TypeKind::Int, TypeAttr::default(), None) // 整数拡張
                } else {
                    return Err(CompileError::InvalidExpr {
                        msg: format!("ビット否定演算子は整数型にのみ適用可能です: {:?}", expr_ty),
                        span,
                    });
                }
            }
            UnaryOp::LogicalNot => {
                let expr_ty = &expr.ty;

                if expr_ty.is_scalar() || expr_ty.is_array() {
                    TypeRef::register(TypeKind::Int, TypeAttr::default(), None) // 結果型はint型
                } else {
                    return Err(CompileError::InvalidExpr {
                        msg: format!(
                            "論理否定演算子はスカラー型または配列型にのみ適用可能です: {:?}",
                            expr_ty
                        ),
                        span,
                    });
                }
            }
            UnaryOp::Addr => {
                let expr_ty = &expr.ty;

                // アドレス演算子の型はポインタ型にする
                TypeRef::register(TypeKind::Ptr { to: *expr_ty }, TypeAttr::default(), None)
            }
            UnaryOp::Deref => {
                let expr_ty = &expr.ty;

                // デリファレンス演算子の型はポインタの指す型にする
                if !(expr_ty.is_ptr() || expr_ty.is_array()) {
                    return Err(CompileError::InvalidExpr {
                        msg: format!(
                            "デリファレンス演算子はポインタ/配列型にのみ適用可能です: {:?}",
                            expr_ty
                        ),
                        span,
                    });
                }
                expr_ty.base_type()
            }
            UnaryOp::PreInc | UnaryOp::PreDec | UnaryOp::PostInc | UnaryOp::PostDec => {
                expr.ty // インクリメント・デクリメント演算子の型はオペランドの型とする
            }
        };

        Ok(Node {
            kind: NodeKind::UnaryOp { op, expr },
            ty,
            span,
        })
    }

    pub(crate) fn new_assign(op: BinaryOp, lhs: Box<Node>, rhs: Box<Node>, span: Span) -> Self {
        let ty = lhs.ty; // 代入演算子の型は左辺の型とする

        Node {
            kind: NodeKind::Assign { op, lhs, rhs },
            ty,
            span,
        }
    }

    pub(crate) fn new_logical_and(
        lhs: Box<Node>,
        rhs: Box<Node>,
        label: usize,
        span: Span,
    ) -> Result<Self, CompileError> {
        let lhs_ty = &lhs.ty;
        let rhs_ty = &rhs.ty;

        let ty = if (lhs_ty.is_scalar() || lhs_ty.is_array())
            && (rhs_ty.is_scalar() || rhs_ty.is_array())
        {
            // 両方ともスカラー型または配列型の場合、結果型はint型とする
            TypeRef::register(TypeKind::Int, TypeAttr::default(), None)
        } else {
            return Err(CompileError::InvalidExpr {
                msg: format!(
                    "論理演算子はスカラー型または配列型にのみ適用可能です: {:?} と {:?}",
                    lhs_ty, rhs_ty
                ),
                span,
            });
        };

        Ok(Node {
            kind: NodeKind::LogicalAnd { lhs, rhs, label },
            ty,
            span,
        })
    }

    pub(crate) fn new_logical_or(
        lhs: Box<Node>,
        rhs: Box<Node>,
        label: usize,
        span: Span,
    ) -> Result<Self, CompileError> {
        let lhs_ty = &lhs.ty;
        let rhs_ty = &rhs.ty;

        let ty = if (lhs_ty.is_scalar() || lhs_ty.is_array())
            && (rhs_ty.is_scalar() || rhs_ty.is_array())
        {
            // 両方ともスカラー型または配列型の場合、結果型はint型とする
            TypeRef::register(TypeKind::Int, TypeAttr::default(), None)
        } else {
            return Err(CompileError::InvalidExpr {
                msg: format!(
                    "論理演算子はスカラー型または配列型にのみ適用可能です: {:?} と {:?}",
                    lhs_ty, rhs_ty
                ),
                span,
            });
        };

        Ok(Node {
            kind: NodeKind::LogicalOr { lhs, rhs, label },
            ty,
            span,
        })
    }

    pub(crate) fn new_ternary(
        cond: Box<Node>,
        then: Box<Node>,
        els: Box<Node>,
        label: usize,
        span: Span,
    ) -> Result<Self, CompileError> {
        let cond_ty = cond.ty;
        let then_ty = then.ty;
        let els_ty = els.ty;

        let ty = if cond_ty.is_scalar() || cond_ty.is_array() {
            if then_ty == els_ty {
                // then節とelse節の型が同じ場合、その型を結果型とする
                then_ty
            } else if then_ty.is_scalar() && els_ty.is_scalar() {
                // 両方ともスカラー型の場合、大きい方の型に合わせる
                if then_ty.size_of() >= els_ty.size_of() {
                    then_ty
                } else {
                    els_ty
                }
            } else {
                return Err(CompileError::InvalidExpr {
                    msg: format!(
                        "条件演算子のthen節とelse節は同じ型か、両方ともスカラー型である必要があります: {:?} と {:?}",
                        then_ty, els_ty
                    ),
                    span,
                });
            }
        } else {
            return Err(CompileError::InvalidExpr {
                msg: format!(
                    "条件演算子の条件式はスカラー型にのみ適用可能です: {:?}",
                    cond_ty
                ),
                span,
            });
        };

        Ok(Node {
            kind: NodeKind::Ternary {
                cond,
                then,
                els,
                label,
            },
            ty,
            span,
        })
    }

    pub(crate) fn new_num(val: i64, span: Span) -> Self {
        Node {
            kind: NodeKind::Number { val },
            ty: TypeRef::register(TypeKind::Int, TypeAttr::default(), None),
            span,
        }
    }

    pub(crate) fn new_var(symbol_id: SymbolId, ty: TypeRef, span: Span) -> Self {
        Node {
            kind: NodeKind::Var { symbol_id },
            ty,
            span,
        }
    }

    pub(crate) fn new_member(
        obj: Box<Node>,
        name: &str,
        offset: usize,
        ty: TypeRef,
        span: Span,
    ) -> Self {
        Node {
            kind: NodeKind::Member {
                obj,
                name: name.to_string(),
                offset,
            },
            ty,
            span,
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
            | NodeKind::Break { .. }
            | NodeKind::Continue { .. }
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
        span: Span,
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
        span: Span,
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
        span: Span,
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
        span: Span,
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
