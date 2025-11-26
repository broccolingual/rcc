use core::str::FromStr;

use crate::ast::{Ast, AstError};
use crate::node::{Node, NodeKind};

impl Ast {
    // const_expr ::= cond_expr
    #[allow(dead_code)]
    pub(super) fn const_expr(&mut self) -> Result<Option<Box<Node>>, AstError> {
        let node = self.cond_expr()?;
        if let Some(ref n) = node {
            if !self.is_const_expr(n)? {
                return Err(AstError::SemanticError(
                    "定数式ではない式が定数式として使用されました".to_string(),
                ));
            }
            if let Some(val) = self.eval_const_expr(n)? {
                return Ok(Some(Box::new(Node::new_num(val))));
            } else {
                return Err(AstError::SemanticError(
                    "定数式の評価に失敗しました".to_string(),
                ));
            }
        }
        Ok(node)
    }

    // 定数式かどうかを判定する
    fn is_const_expr(&self, node: &Node) -> Result<bool, AstError> {
        match &node.kind {
            NodeKind::Number { .. } => Ok(true),
            NodeKind::String { .. } => Ok(false),
            NodeKind::Addr => {
                if let Some(op) = &node.lhs {
                    self.is_addressable_const(op)
                } else {
                    Ok(false)
                }
            }
            NodeKind::Ternary { cond, then, els } => {
                if let (Some(c), Some(t), Some(e)) = (cond, then, els) {
                    Ok(
                        self.is_const_expr(c)?
                            && self.is_const_expr(t)?
                            && self.is_const_expr(e)?,
                    )
                } else {
                    Ok(false)
                }
            }
            NodeKind::Add
            | NodeKind::Sub
            | NodeKind::Mul
            | NodeKind::Div
            | NodeKind::Rem
            | NodeKind::BitAnd
            | NodeKind::BitOr
            | NodeKind::BitXor
            | NodeKind::Shl
            | NodeKind::Shr
            | NodeKind::Eq
            | NodeKind::Ne
            | NodeKind::Lt
            | NodeKind::Le
            | NodeKind::LogicalAnd
            | NodeKind::LogicalOr => {
                if let (Some(lhs), Some(rhs)) = (&node.lhs, &node.rhs) {
                    Ok(self.is_const_expr(lhs)? && self.is_const_expr(rhs)?)
                } else {
                    Ok(false)
                }
            }

            NodeKind::BitNot | NodeKind::LogicalNot => {
                if let Some(op) = &node.lhs {
                    self.is_const_expr(op)
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false), // その他のノードは定数式ではないと判断する
        }
    }

    // アドレス定数式かどうかを判定する
    fn is_addressable_const(&self, node: &Node) -> Result<bool, AstError> {
        match &node.kind {
            NodeKind::GVar { .. } => Ok(true),
            NodeKind::String { .. } => Ok(true),
            NodeKind::Add => {
                // a[i] = *(a + i) のような形であれば定数式とみなす．iは整数定数式である必要がある．
                if let (Some(lhs), Some(rhs)) = (&node.lhs, &node.rhs) {
                    if let NodeKind::GVar { .. } = lhs.kind {
                        Ok(self.is_const_expr(rhs)?)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }

    // 定数式を評価する
    #[allow(clippy::only_used_in_recursion)]
    fn eval_const_expr(&self, node: &Node) -> Result<Option<i64>, AstError> {
        match &node.kind {
            NodeKind::Number { val } => Ok(Some(*val)),
            NodeKind::Add => {
                if let (Some(lhs), Some(rhs)) = (&node.lhs, &node.rhs) {
                    let l = self.eval_const_expr(lhs)?;
                    let r = self.eval_const_expr(rhs)?;
                    if let (Some(l_val), Some(r_val)) = (l, r) {
                        Ok(Some(l_val.wrapping_add(r_val)))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            NodeKind::Sub => {
                if let (Some(lhs), Some(rhs)) = (&node.lhs, &node.rhs) {
                    let l = self.eval_const_expr(lhs)?;
                    let r = self.eval_const_expr(rhs)?;
                    if let (Some(l_val), Some(r_val)) = (l, r) {
                        Ok(Some(l_val.wrapping_sub(r_val)))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            NodeKind::Mul => {
                if let (Some(lhs), Some(rhs)) = (&node.lhs, &node.rhs) {
                    let l = self.eval_const_expr(lhs)?;
                    let r = self.eval_const_expr(rhs)?;
                    if let (Some(l_val), Some(r_val)) = (l, r) {
                        Ok(Some(l_val.wrapping_mul(r_val)))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            NodeKind::Div => {
                if let (Some(lhs), Some(rhs)) = (&node.lhs, &node.rhs) {
                    let l = self.eval_const_expr(lhs)?;
                    let r = self.eval_const_expr(rhs)?;
                    if let (Some(l_val), Some(r_val)) = (l, r) {
                        if r_val != 0 {
                            Ok(Some(l_val / r_val))
                        } else {
                            Err(AstError::SemanticError("ゼロ除算".to_string()))
                        }
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            NodeKind::Rem => {
                if let (Some(lhs), Some(rhs)) = (&node.lhs, &node.rhs) {
                    let l = self.eval_const_expr(lhs)?;
                    let r = self.eval_const_expr(rhs)?;
                    if let (Some(l_val), Some(r_val)) = (l, r) {
                        if r_val != 0 {
                            Ok(Some(l_val % r_val))
                        } else {
                            Err(AstError::SemanticError("ゼロ除算".to_string()))
                        }
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            NodeKind::BitAnd => {
                if let (Some(lhs), Some(rhs)) = (&node.lhs, &node.rhs) {
                    let l = self.eval_const_expr(lhs)?;
                    let r = self.eval_const_expr(rhs)?;
                    if let (Some(l_val), Some(r_val)) = (l, r) {
                        Ok(Some(l_val & r_val))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            NodeKind::BitOr => {
                if let (Some(lhs), Some(rhs)) = (&node.lhs, &node.rhs) {
                    let l = self.eval_const_expr(lhs)?;
                    let r = self.eval_const_expr(rhs)?;
                    if let (Some(l_val), Some(r_val)) = (l, r) {
                        Ok(Some(l_val | r_val))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            NodeKind::BitXor => {
                if let (Some(lhs), Some(rhs)) = (&node.lhs, &node.rhs) {
                    let l = self.eval_const_expr(lhs)?;
                    let r = self.eval_const_expr(rhs)?;
                    if let (Some(l_val), Some(r_val)) = (l, r) {
                        Ok(Some(l_val ^ r_val))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            NodeKind::Shl => {
                if let (Some(lhs), Some(rhs)) = (&node.lhs, &node.rhs) {
                    let l = self.eval_const_expr(lhs)?;
                    let r = self.eval_const_expr(rhs)?;
                    if let (Some(l_val), Some(r_val)) = (l, r) {
                        if (0..64).contains(&r_val) {
                            Ok(Some(l_val << r_val))
                        } else {
                            Err(AstError::SemanticError("不正なシフト量".to_string()))
                        }
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            NodeKind::Shr => {
                if let (Some(lhs), Some(rhs)) = (&node.lhs, &node.rhs) {
                    let l = self.eval_const_expr(lhs)?;
                    let r = self.eval_const_expr(rhs)?;
                    if let (Some(l_val), Some(r_val)) = (l, r) {
                        if (0..64).contains(&r_val) {
                            Ok(Some(l_val >> r_val))
                        } else {
                            Err(AstError::SemanticError("不正なシフト量".to_string()))
                        }
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            NodeKind::Eq => {
                if let (Some(lhs), Some(rhs)) = (&node.lhs, &node.rhs) {
                    let l = self.eval_const_expr(lhs)?;
                    let r = self.eval_const_expr(rhs)?;
                    if let (Some(l_val), Some(r_val)) = (l, r) {
                        Ok(Some(if l_val == r_val { 1 } else { 0 }))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            NodeKind::Ne => {
                if let (Some(lhs), Some(rhs)) = (&node.lhs, &node.rhs) {
                    let l = self.eval_const_expr(lhs)?;
                    let r = self.eval_const_expr(rhs)?;
                    if let (Some(l_val), Some(r_val)) = (l, r) {
                        Ok(Some(if l_val != r_val { 1 } else { 0 }))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            NodeKind::Lt => {
                if let (Some(lhs), Some(rhs)) = (&node.lhs, &node.rhs) {
                    let l = self.eval_const_expr(lhs)?;
                    let r = self.eval_const_expr(rhs)?;
                    if let (Some(l_val), Some(r_val)) = (l, r) {
                        Ok(Some(if l_val < r_val { 1 } else { 0 }))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            NodeKind::Le => {
                if let (Some(lhs), Some(rhs)) = (&node.lhs, &node.rhs) {
                    let l = self.eval_const_expr(lhs)?;
                    let r = self.eval_const_expr(rhs)?;
                    if let (Some(l_val), Some(r_val)) = (l, r) {
                        Ok(Some(if l_val <= r_val { 1 } else { 0 }))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            NodeKind::LogicalAnd => {
                if let (Some(lhs), Some(rhs)) = (&node.lhs, &node.rhs) {
                    let l = self.eval_const_expr(lhs)?;
                    let r = self.eval_const_expr(rhs)?;
                    if let (Some(l_val), Some(r_val)) = (l, r) {
                        Ok(Some(if l_val != 0 && r_val != 0 { 1 } else { 0 }))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            NodeKind::LogicalOr => {
                if let (Some(lhs), Some(rhs)) = (&node.lhs, &node.rhs) {
                    let l = self.eval_const_expr(lhs)?;
                    let r = self.eval_const_expr(rhs)?;
                    if let (Some(l_val), Some(r_val)) = (l, r) {
                        Ok(Some(if l_val != 0 || r_val != 0 { 1 } else { 0 }))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            NodeKind::BitNot => {
                if let Some(operand) = &node.lhs {
                    let val = self.eval_const_expr(operand)?;
                    if let Some(v) = val {
                        Ok(Some(!v))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            NodeKind::LogicalNot => {
                if let Some(operand) = &node.lhs {
                    let val = self.eval_const_expr(operand)?;
                    if let Some(v) = val {
                        Ok(Some(if v == 0 { 1 } else { 0 }))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            NodeKind::Ternary { cond, then, els } => {
                if let (Some(c), Some(t), Some(e)) = (cond, then, els) {
                    let cond_val = self.eval_const_expr(c)?;
                    if let Some(cv) = cond_val {
                        if cv != 0 {
                            self.eval_const_expr(t)
                        } else {
                            self.eval_const_expr(e)
                        }
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    // expr ::= assign_expr
    pub(super) fn expr(&mut self) -> Result<Option<Box<Node>>, AstError> {
        self.assign_expr()
    }

    // assign_expr ::= cond_expr
    //                 | ("=" | "*=" | "/=" | "%=" | "+=" | "-=" | "<<=" | ">>=" | "&=" | "^=" | "|=") assign_expr
    pub(super) fn assign_expr(&mut self) -> Result<Option<Box<Node>>, AstError> {
        let mut node = self.cond_expr()?;
        let assignment_ops = [
            "=", "*=", "/=", "%=", "+=", "-=", "<<=", ">>=", "&=", "^=", "|=",
        ];
        for op in &assignment_ops {
            if self.consume_punctuator(op).is_some()
                && let Ok(kind) = NodeKind::from_str(op)
            {
                node = Some(Box::new(Node::new(kind, node, self.assign_expr()?)));
                break;
            }
        }
        Ok(node)
    }

    // cond_expr ::= logical_or_expr
    //               | logical_or_expr "?" expr ":" cond_expr
    fn cond_expr(&mut self) -> Result<Option<Box<Node>>, AstError> {
        let node = self.logical_or_expr()?;
        if self.consume_punctuator("?").is_some() {
            let cond = node;
            let then = self.expr()?;
            self.expect_punctuator(":")?;
            let els = self.cond_expr()?;
            return Ok(Some(Box::new(Node::from(NodeKind::Ternary {
                cond,
                then,
                els,
            }))));
        }
        Ok(node)
    }

    // logical_or_expr ::= logical_and_expr
    //                     | logical_or_expr "||" logical_and_expr
    fn logical_or_expr(&mut self) -> Result<Option<Box<Node>>, AstError> {
        let mut node = self.logical_and_expr()?;

        loop {
            if self.consume_punctuator("||").is_some() {
                // logical or
                node = Some(Box::new(Node::new(
                    NodeKind::LogicalOr,
                    node,
                    self.logical_and_expr()?,
                )));
            } else {
                return Ok(node);
            }
        }
    }

    // logical_and_expr ::= inclusive_or_expr
    //                      | logical_and_expr "&&" inclusive_or_expr
    fn logical_and_expr(&mut self) -> Result<Option<Box<Node>>, AstError> {
        let mut node = self.inclusive_or_expr()?;

        loop {
            if self.consume_punctuator("&&").is_some() {
                // logical and
                node = Some(Box::new(Node::new(
                    NodeKind::LogicalAnd,
                    node,
                    self.inclusive_or_expr()?,
                )));
            } else {
                return Ok(node);
            }
        }
    }

    // inclusive_or_expr ::= exclusive_or_expr
    //                       | inclusive_or_expr "|" exclusive_or_expr
    fn inclusive_or_expr(&mut self) -> Result<Option<Box<Node>>, AstError> {
        let mut node = self.exclusive_or_expr()?;

        loop {
            if self.consume_punctuator("|").is_some() {
                // bitwise or
                node = Some(Box::new(Node::new(
                    NodeKind::BitOr,
                    node,
                    self.exclusive_or_expr()?,
                )));
            } else {
                return Ok(node);
            }
        }
    }

    // exclusive_or_expr ::= and_expr
    //                       | exclusive_or_expr "^" and_expr
    fn exclusive_or_expr(&mut self) -> Result<Option<Box<Node>>, AstError> {
        let mut node = self.and_expr()?;

        loop {
            if self.consume_punctuator("^").is_some() {
                // bitwise xor
                node = Some(Box::new(Node::new(
                    NodeKind::BitXor,
                    node,
                    self.and_expr()?,
                )));
            } else {
                return Ok(node);
            }
        }
    }

    // and_expr ::= equality_expr
    //              | and_expr "&" equality_expr
    fn and_expr(&mut self) -> Result<Option<Box<Node>>, AstError> {
        let mut node = self.equality_expr()?;

        loop {
            if self.consume_punctuator("&").is_some() {
                //bitwise and
                node = Some(Box::new(Node::new(
                    NodeKind::BitAnd,
                    node,
                    self.equality_expr()?,
                )));
            } else {
                return Ok(node);
            }
        }
    }

    // equality_expr ::= relational_expr
    //                   | equality_expr ("==" | "!=") relational_expr
    fn equality_expr(&mut self) -> Result<Option<Box<Node>>, AstError> {
        let mut node = self.relational_expr()?;

        loop {
            if self.consume_punctuator("==").is_some() {
                // equal
                node = Some(Box::new(Node::new(
                    NodeKind::Eq,
                    node,
                    self.relational_expr()?,
                )));
            } else if self.consume_punctuator("!=").is_some() {
                // not equal
                node = Some(Box::new(Node::new(
                    NodeKind::Ne,
                    node,
                    self.relational_expr()?,
                )));
            } else {
                return Ok(node);
            }
        }
    }

    // relational_expr ::= shift_expr
    //                     | relational_expr ("<" | "<=" | ">" | ">=") shift_expr
    fn relational_expr(&mut self) -> Result<Option<Box<Node>>, AstError> {
        let mut node = self.shift_expr()?;

        loop {
            if self.consume_punctuator("<").is_some() {
                // less than
                node = Some(Box::new(Node::new(NodeKind::Lt, node, self.shift_expr()?)));
            } else if self.consume_punctuator("<=").is_some() {
                // less than or equal
                node = Some(Box::new(Node::new(NodeKind::Le, node, self.shift_expr()?)));
            } else if self.consume_punctuator(">").is_some() {
                // greater than
                node = Some(Box::new(Node::new(NodeKind::Lt, self.shift_expr()?, node)));
            } else if self.consume_punctuator(">=").is_some() {
                // greater than or equal
                node = Some(Box::new(Node::new(NodeKind::Le, self.shift_expr()?, node)));
            } else {
                return Ok(node);
            }
        }
    }

    // shift_expr ::= add_expr
    //                | shift_expr ("<<" | ">>") add_expr
    fn shift_expr(&mut self) -> Result<Option<Box<Node>>, AstError> {
        let mut node = self.add_expr()?;

        loop {
            if self.consume_punctuator("<<").is_some() {
                // left shift
                node = Some(Box::new(Node::new(NodeKind::Shl, node, self.add_expr()?)));
            } else if self.consume_punctuator(">>").is_some() {
                // right shift
                node = Some(Box::new(Node::new(NodeKind::Shr, node, self.add_expr()?)));
            } else {
                return Ok(node);
            }
        }
    }

    // add_expr ::= mul_expr
    //              | add_expr ("+" | "-") mul_expr
    fn add_expr(&mut self) -> Result<Option<Box<Node>>, AstError> {
        let mut node = self.mul_expr()?;

        loop {
            if self.consume_punctuator("+").is_some() {
                // addition
                node.as_mut().unwrap().assign_types()?; // lhs
                let mut rhs = self.mul_expr()?;
                rhs.as_mut().unwrap().assign_types()?; // rhs
                if let Some(n) = &node
                    && let Some(ty) = &n.ty
                    && ty.is_ptr_or_array()
                {
                    // ポインタ減算の場合、スケーリングを考慮
                    let size = ty.base_type().size_of();
                    rhs = Some(Box::new(Node::new(
                        NodeKind::Mul,
                        rhs,
                        Some(Box::new(Node::new_num(size))),
                    )));
                }
                node = Some(Box::new(Node::new(NodeKind::Add, node, rhs)));
            } else if self.consume_punctuator("-").is_some() {
                // subtraction
                node.as_mut().unwrap().assign_types()?; // lhs
                let mut rhs = self.mul_expr()?;
                rhs.as_mut().unwrap().assign_types()?; // rhs
                if let Some(n) = &node
                    && let Some(ty) = &n.ty
                    && ty.is_ptr_or_array()
                {
                    // ポインタ減算の場合、スケーリングを考慮
                    let size = ty.base_type().size_of();
                    rhs = Some(Box::new(Node::new(
                        NodeKind::Mul,
                        rhs,
                        Some(Box::new(Node::new_num(size))),
                    )));
                }
                node = Some(Box::new(Node::new(NodeKind::Sub, node, rhs)));
            } else {
                return Ok(node);
            }
        }
    }

    // mul_expr ::= cast_expr
    //              | mul_expr ("*" | "/" | "%") cast_expr
    fn mul_expr(&mut self) -> Result<Option<Box<Node>>, AstError> {
        let mut node = self.cast_expr()?;

        loop {
            if self.consume_punctuator("*").is_some() {
                // multiplication
                node = Some(Box::new(Node::new(NodeKind::Mul, node, self.cast_expr()?)));
            } else if self.consume_punctuator("/").is_some() {
                // division
                node = Some(Box::new(Node::new(NodeKind::Div, node, self.cast_expr()?)));
            } else if self.consume_punctuator("%").is_some() {
                // remainder
                node = Some(Box::new(Node::new(NodeKind::Rem, node, self.cast_expr()?)));
            } else {
                return Ok(node);
            }
        }
    }

    // cast_expr ::= unary_expr
    //               | "(" type_name ")" cast_expr // 未実装
    fn cast_expr(&mut self) -> Result<Option<Box<Node>>, AstError> {
        self.unary_expr()
    }

    // unary_expr ::= postfix_expr
    //                | ("++" | "--") unary_expr
    //                | ( "&" | "*" | "+" | "-" | "~" | "!") cast_expr
    //                | sizeof unary_expr
    //                | sizeof "(" type_name ")" // 未実装
    fn unary_expr(&mut self) -> Result<Option<Box<Node>>, AstError> {
        if self.consume_punctuator("++").is_some() {
            // pre-increment
            return Ok(Some(Box::new(Node::new_unary(
                NodeKind::PreInc,
                self.unary_expr()?,
            ))));
        }
        if self.consume_punctuator("--").is_some() {
            // pre-decrement
            return Ok(Some(Box::new(Node::new_unary(
                NodeKind::PreDec,
                self.unary_expr()?,
            ))));
        }

        if self.consume_punctuator("+").is_some() {
            // unary plus
            return self.cast_expr();
        }
        if self.consume_punctuator("-").is_some() {
            // unary minus
            return Ok(Some(Box::new(Node::new(
                NodeKind::Sub,
                Some(Box::new(Node::new_num(0))),
                self.cast_expr()?,
            ))));
        }
        if self.consume_punctuator("&").is_some() {
            // address-of
            let mut node = Box::new(Node::new_unary(NodeKind::Addr, self.cast_expr()?));
            node.assign_types()?;
            if node.ty.is_none() {
                Err(AstError::TypeError(format!(
                    "&演算子の型情報が設定されていません: {:?}",
                    node
                )))?;
            }
            return Ok(Some(node));
        }
        if self.consume_punctuator("*").is_some() {
            // dereference
            let mut node = Box::new(Node::new_unary(NodeKind::Deref, self.cast_expr()?));
            node.assign_types()?;
            if node.ty.is_none() {
                Err(AstError::TypeError(format!(
                    "*演算子の型情報が設定されていません: {:?}",
                    node
                )))?;
            }
            return Ok(Some(node));
        }
        if self.consume_punctuator("~").is_some() {
            // bitwise not
            return Ok(Some(Box::new(Node::new_unary(
                NodeKind::BitNot,
                self.cast_expr()?,
            ))));
        }
        if self.consume_punctuator("!").is_some() {
            // logical not
            return Ok(Some(Box::new(Node::new_unary(
                NodeKind::LogicalNot,
                self.cast_expr()?,
            ))));
        }

        if self.consume_keyword("sizeof").is_some() {
            // sizeof unary_expr
            let mut node = self.unary_expr()?;
            if let Some(n) = &mut node {
                n.assign_types()?;
                if let Some(ty) = &n.ty {
                    let size = ty.size_of();
                    return Ok(Some(Box::new(Node::new_num(size))));
                } else {
                    return Err(AstError::TypeError(format!(
                        "sizeof演算子の型情報が設定されていません: {:?}",
                        node
                    )));
                }
            }
        }

        self.postfix_expr()
    }

    // postfix_expr ::= primary_expr
    //                  | postfix_expr ("++" | "--")
    fn postfix_expr(&mut self) -> Result<Option<Box<Node>>, AstError> {
        let mut node = self.primary_expr()?;

        loop {
            if self.consume_punctuator("++").is_some() {
                // post-increment
                node = Some(Box::new(Node::new_unary(NodeKind::PostInc, node)));
            } else if self.consume_punctuator("--").is_some() {
                // post-decrement
                node = Some(Box::new(Node::new_unary(NodeKind::PostDec, node)));
            } else {
                return Ok(node);
            }
        }
    }

    // argument_expr_list ::= assign_expr ("," assign_expr)*
    #[allow(clippy::vec_box)]
    fn argument_expr_list(&mut self) -> Result<Vec<Box<Node>>, AstError> {
        let mut args = Vec::new();
        if let Some(arg) = self.assign_expr()? {
            args.insert(0, arg); // 逆順で格納
        } else {
            return Ok(args);
        }

        while self.consume_punctuator(",").is_some() {
            if let Some(arg) = self.assign_expr()? {
                args.insert(0, arg); // 逆順で格納
            } else {
                return Err(AstError::ParseError(
                    "関数呼び出しの引数のパースに失敗しました".to_string(),
                ));
            }
        }
        Ok(args)
    }

    // primary_expr ::= "(" expr ")"
    //                  | ident "(" argument_expr_list? ")"
    //                  | ident ("[" expr "]")*
    //                  | string
    //                  | number
    fn primary_expr(&mut self) -> Result<Option<Box<Node>>, AstError> {
        // "(" expr ")"
        if self.consume_punctuator("(").is_some()
            && let Some(node) = self.expr()?
        {
            self.expect_punctuator(")")?;
            return Ok(Some(node));
        }

        if let Some(name) = self.consume_ident() {
            // 関数呼び出し
            if self.consume_punctuator("(").is_some() {
                // 引数リストをパース
                let args = self.argument_expr_list()?;
                self.expect_punctuator(")")?;
                return Ok(Some(Box::new(Node::from(NodeKind::Call { name, args }))));
            }

            // 変数参照
            if let Ok(current_func) = self.get_current_func()
                && let Some(lvar) = current_func.find_lvar(&name)
            {
                // ローカル変数ノードを作成
                let node = Node::new_lvar(&lvar.name, lvar.offset, &lvar.ty);

                // 配列の場合は自動的にアドレスに変換
                // 例: a[0] -> *(a + 0)
                // 例: a[1][2] -> *(*(a + 1) + 2)
                // TODO: 多次元配列への対応
                if self.consume_punctuator("[").is_some() {
                    let add = Node::new(NodeKind::Add, Some(Box::new(node)), self.expr()?);
                    let mut n = Box::new(Node::new_unary(NodeKind::Deref, Some(Box::new(add))));
                    n.assign_types()?;
                    self.expect_punctuator("]")?;
                    return Ok(Some(n));
                }
                return Ok(Some(Box::new(node)));
            } else if let Some(gvar) = self.find_gvar(&name) {
                // グローバル変数ノードを作成
                let node = Node::new_gvar(&gvar.name, &gvar.ty);

                // 配列の場合は自動的にアドレスに変換
                // TODO: 多次元配列への対応
                if self.consume_punctuator("[").is_some() {
                    let add = Node::new(NodeKind::Add, Some(Box::new(node)), self.expr()?);
                    let mut n = Box::new(Node::new_unary(NodeKind::Deref, Some(Box::new(add))));
                    n.assign_types()?;
                    self.expect_punctuator("]")?;
                    return Ok(Some(n));
                }
                return Ok(Some(Box::new(node)));
            }
            Err(AstError::UndefinedVariable(name))?;
        }

        if let Ok(string) = self.expect_string() {
            let node = Node::from(NodeKind::String {
                val: string.clone(),
                index: self.string_literals.len() as i64,
            });
            self.string_literals.push(string);
            return Ok(Some(Box::new(node)));
        }

        if let Some(num) = self.consume_number() {
            return Ok(Some(Box::new(Node::new_num(num))));
        }

        Ok(None)
    }
}
