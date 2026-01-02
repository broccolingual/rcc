use crate::ast::Ast;
use crate::errors::CompileError;
use crate::node::{BinaryOp, Node, NodeKind, UnaryOp};
use crate::types::TypeKind;
use core::str::FromStr;
use std::ops::Deref;

impl Ast<'_> {
    // const_expr ::= cond_expr
    #[allow(dead_code)]
    pub(super) fn const_expr(&mut self) -> Result<i64, CompileError> {
        let node = self
            .cond_expr()?
            .ok_or_else(|| CompileError::InternalError {
                msg: "定数式がありません".to_string(),
            })?;
        Self::eval_const_expr(&node) // 定数式を評価
    }

    // 定数式を評価して、その値を返す
    pub(crate) fn eval_const_expr(expr: &Node) -> Result<i64, CompileError> {
        match &expr.kind {
            NodeKind::Number { val } => Ok(*val),
            NodeKind::UnaryOp { op, expr } => {
                let val = Self::eval_const_expr(expr)?;
                match op {
                    UnaryOp::BitNot => Ok(!val),
                    UnaryOp::LogicalNot => Ok(if val == 0 { 1 } else { 0 }),
                    _ => Err(CompileError::InvalidExpr {
                        msg: format!("定数式に不正な単項演算子が含まれています: {:?}", op),
                        span: expr.span,
                    }),
                }
            }
            NodeKind::BinaryOp { op, lhs, rhs } => {
                let lval = Self::eval_const_expr(lhs)?;
                let rval = Self::eval_const_expr(rhs)?;
                match op {
                    BinaryOp::Add => Ok(lval + rval),
                    BinaryOp::Sub => Ok(lval - rval),
                    BinaryOp::Mul => Ok(lval * rval),
                    BinaryOp::Div => {
                        if rval == 0 {
                            return Err(CompileError::InvalidExpr {
                                msg: "定数式の除算でゼロ除算が発生しました".to_string(),
                                span: rhs.span,
                            });
                        }
                        Ok(lval / rval)
                    }
                    BinaryOp::Rem => {
                        if rval == 0 {
                            return Err(CompileError::InvalidExpr {
                                msg: "定数式の剰余演算でゼロ除算が発生しました".to_string(),
                                span: rhs.span,
                            });
                        }
                        Ok(lval % rval)
                    }
                    BinaryOp::Shl => {
                        if !(0..64).contains(&rval) {
                            return Err(CompileError::InvalidExpr {
                                msg: "定数式の左シフト演算で不正なシフト量が指定されました"
                                    .to_string(),
                                span: rhs.span,
                            });
                        }
                        Ok(lval << rval)
                    }
                    BinaryOp::Shr => {
                        if !(0..64).contains(&rval) {
                            return Err(CompileError::InvalidExpr {
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
                    _ => Err(CompileError::InvalidExpr {
                        msg: format!("定数式に不正な二項演算子が含まれています: {:?}", op),
                        span: expr.span,
                    }),
                }
            }
            NodeKind::Ternary {
                cond, then, els, ..
            } => {
                let cond_val = Self::eval_const_expr(cond)?;
                if cond_val != 0 {
                    Self::eval_const_expr(then)
                } else {
                    Self::eval_const_expr(els)
                }
            }
            NodeKind::LogicalAnd { lhs, rhs, .. } => {
                let lval = Self::eval_const_expr(lhs)?;
                if lval == 0 {
                    Ok(0)
                } else {
                    let rval = Self::eval_const_expr(rhs)?;
                    Ok(if rval != 0 { 1 } else { 0 })
                }
            }
            NodeKind::LogicalOr { lhs, rhs, .. } => {
                let lval = Self::eval_const_expr(lhs)?;
                if lval != 0 {
                    Ok(1)
                } else {
                    let rval = Self::eval_const_expr(rhs)?;
                    Ok(if rval != 0 { 1 } else { 0 })
                }
            }
            NodeKind::Var { .. } => {
                unimplemented!("定数式内の変数参照の評価は未実装です")
            }
            _ => Err(CompileError::InvalidExpr {
                msg: "定数式に不正なノードが含まれています".to_string(),
                span: expr.span,
            }),
        }
    }

    // expr ::= assign_expr
    pub(super) fn expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        self.assign_expr()
    }

    // assign_expr ::= cond_expr
    //               | ("=" | "*=" | "/=" | "%=" | "+=" | "-=" | "<<=" | ">>=" | "&=" | "^=" | "|=") assign_expr
    pub(super) fn assign_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node = self.cond_expr()?;
        let assign_op_str_list = [
            "=", "*=", "/=", "%=", "+=", "-=", "<<=", ">>=", "&=", "^=", "|=",
        ];
        for assign_op_str in &assign_op_str_list {
            if let Some(token) = self.consume_punct(assign_op_str)
                && let Ok(kind) = BinaryOp::from_str(assign_op_str)
            {
                let span = token.span;
                let lhs = node.ok_or_else(|| CompileError::InvalidExpr {
                    msg: format!("代入演算子 '{}' の左辺に式がありません", assign_op_str),
                    span,
                })?;
                let rhs = self
                    .assign_expr()?
                    .ok_or_else(|| CompileError::InvalidExpr {
                        msg: format!("代入演算子 '{}' の右辺に式がありません", assign_op_str),
                        span,
                    })?;
                node = Some(Box::new(Node::new_assign(kind, lhs, rhs, span)));
                break;
            }
        }
        Ok(node)
    }

    // cond_expr ::= logical_or_expr
    //             | logical_or_expr "?" expr ":" cond_expr
    fn cond_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let node = self.logical_or_expr()?;
        if let Some(token) = self.consume_punct("?") {
            let span = token.span;
            let label = self.next_label();
            let cond = node.ok_or_else(|| CompileError::InvalidExpr {
                msg: "三項演算子 '?' の前に条件式がありません".to_string(),
                span,
            })?;
            let then = self.expr()?.ok_or_else(|| CompileError::InvalidExpr {
                msg: "三項演算子の ':' の前に式がありません (true の場合の値)".to_string(),
                span,
            })?;
            self.expect_punct(":")?;
            let els = self.cond_expr()?.ok_or_else(|| CompileError::InvalidExpr {
                msg: "三項演算子の ':' の後に式がありません (false の場合の値)".to_string(),
                span,
            })?;
            return Ok(Some(Box::new(Node::new_ternary(
                cond, then, els, label, span,
            )?)));
        }
        Ok(node)
    }

    // logical_or_expr ::= logical_and_expr
    //                   | logical_or_expr "||" logical_and_expr
    fn logical_or_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node = self.logical_and_expr()?;

        loop {
            if let Some(token) = self.consume_punct("||") {
                let span = token.span;
                let label = self.next_label();
                // logical or
                let lhs = node.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'||'演算子の左辺値がありません".to_string(),
                    span,
                })?;
                let rhs = self
                    .logical_and_expr()?
                    .ok_or_else(|| CompileError::InvalidExpr {
                        msg: "'||'演算子の右辺値がありません".to_string(),
                        span,
                    })?;
                node = Some(Box::new(Node::new_logical_or(lhs, rhs, label, span)?));
            } else {
                return Ok(node);
            }
        }
    }

    // logical_and_expr ::= inclusive_or_expr
    //                    | logical_and_expr "&&" inclusive_or_expr
    fn logical_and_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node = self.inclusive_or_expr()?;

        loop {
            if let Some(token) = self.consume_punct("&&") {
                let span = token.span;
                let label = self.next_label();
                // logical and
                let lhs = node.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'&&'演算子の左辺値がありません".to_string(),
                    span,
                })?;
                let rhs = self
                    .inclusive_or_expr()?
                    .ok_or_else(|| CompileError::InvalidExpr {
                        msg: "'&&'演算子の右辺値がありません".to_string(),
                        span,
                    })?;
                node = Some(Box::new(Node::new_logical_and(lhs, rhs, label, span)?));
            } else {
                return Ok(node);
            }
        }
    }

    // inclusive_or_expr ::= exclusive_or_expr
    //                     | inclusive_or_expr "|" exclusive_or_expr
    fn inclusive_or_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node = self.exclusive_or_expr()?;

        loop {
            if let Some(token) = self.consume_punct("|") {
                let span = token.span;
                // bitwise or
                let lhs = node.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'|'演算子の左辺値がありません".to_string(),
                    span,
                })?;
                let rhs = self
                    .exclusive_or_expr()?
                    .ok_or_else(|| CompileError::InvalidExpr {
                        msg: "'|'演算子の右辺値がありません".to_string(),
                        span,
                    })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::BitOr, lhs, rhs, span)?));
            } else {
                return Ok(node);
            }
        }
    }

    // exclusive_or_expr ::= and_expr
    //                     | exclusive_or_expr "^" and_expr
    fn exclusive_or_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node = self.and_expr()?;

        loop {
            if let Some(token) = self.consume_punct("^") {
                let span = token.span;
                // bitwise xor
                let lhs = node.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'^'演算子の左辺値がありません".to_string(),
                    span,
                })?;
                let rhs = self.and_expr()?.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'^'演算子の右辺値がありません".to_string(),
                    span,
                })?;
                node = Some(Box::new(Node::new_binary(
                    BinaryOp::BitXor,
                    lhs,
                    rhs,
                    span,
                )?));
            } else {
                return Ok(node);
            }
        }
    }

    // and_expr ::= equality_expr
    //            | and_expr "&" equality_expr
    fn and_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node = self.equality_expr()?;

        loop {
            if let Some(token) = self.consume_punct("&") {
                let span = token.span;
                //bitwise and
                let lhs = node.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'&'演算子の左辺値がありません".to_string(),
                    span,
                })?;
                let rhs = self
                    .equality_expr()?
                    .ok_or_else(|| CompileError::InvalidExpr {
                        msg: "'&'演算子の右辺値がありません".to_string(),
                        span,
                    })?;
                node = Some(Box::new(Node::new_binary(
                    BinaryOp::BitAnd,
                    lhs,
                    rhs,
                    span,
                )?));
            } else {
                return Ok(node);
            }
        }
    }

    // equality_expr ::= relational_expr
    //                 | equality_expr ("==" | "!=") relational_expr
    fn equality_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node = self.relational_expr()?;

        loop {
            if let Some(token) = self.consume_punct("==") {
                let span = token.span;
                // equal
                let lhs = node.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'=='演算子の左辺値がありません".to_string(),
                    span,
                })?;
                let rhs = self
                    .relational_expr()?
                    .ok_or_else(|| CompileError::InvalidExpr {
                        msg: "'=='演算子の右辺値がありません".to_string(),
                        span,
                    })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Eq, lhs, rhs, span)?));
            } else if let Some(token) = self.consume_punct("!=") {
                let span = token.span;
                // not equal
                let lhs = node.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'!='演算子の左辺値がありません".to_string(),
                    span,
                })?;
                let rhs = self
                    .relational_expr()?
                    .ok_or_else(|| CompileError::InvalidExpr {
                        msg: "'!='演算子の右辺値がありません".to_string(),
                        span,
                    })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Ne, lhs, rhs, span)?));
            } else {
                return Ok(node);
            }
        }
    }

    // relational_expr ::= shift_expr
    //                   | relational_expr ("<" | "<=" | ">" | ">=") shift_expr
    fn relational_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node = self.shift_expr()?;

        loop {
            if let Some(token) = self.consume_punct("<") {
                let span = token.span;
                // less than
                let lhs = node.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'<'演算子の左辺値がありません".to_string(),
                    span,
                })?;
                let rhs = self
                    .shift_expr()?
                    .ok_or_else(|| CompileError::InvalidExpr {
                        msg: "'<'演算子の右辺値がありません".to_string(),
                        span,
                    })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Lt, lhs, rhs, span)?));
            } else if let Some(token) = self.consume_punct("<=") {
                let span = token.span;
                // less than or equal
                let lhs = node.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'<='演算子の左辺値がありません".to_string(),
                    span,
                })?;
                let rhs = self
                    .shift_expr()?
                    .ok_or_else(|| CompileError::InvalidExpr {
                        msg: "'<='演算子の右辺値がありません".to_string(),
                        span,
                    })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Le, lhs, rhs, span)?));
            } else if let Some(token) = self.consume_punct(">") {
                let span = token.span;
                // greater than
                let lhs = self
                    .shift_expr()?
                    .ok_or_else(|| CompileError::InvalidExpr {
                        msg: "'>'演算子の右辺値がありません".to_string(),
                        span,
                    })?;
                let rhs = node.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'>'演算子の左辺値がありません".to_string(),
                    span,
                })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Lt, lhs, rhs, span)?));
            } else if let Some(token) = self.consume_punct(">=") {
                let span = token.span;
                // greater than or equal
                let lhs = self
                    .shift_expr()?
                    .ok_or_else(|| CompileError::InvalidExpr {
                        msg: "'>='演算子の右辺値がありません".to_string(),
                        span,
                    })?;
                let rhs = node.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'>='演算子の左辺値がありません".to_string(),
                    span,
                })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Le, lhs, rhs, span)?));
            } else {
                return Ok(node);
            }
        }
    }

    // shift_expr ::= add_expr
    //              | shift_expr ("<<" | ">>") add_expr
    fn shift_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node = self.add_expr()?;

        loop {
            if let Some(token) = self.consume_punct("<<") {
                let span = token.span;
                // left shift
                let lhs = node.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'<<'演算子の左辺値がありません".to_string(),
                    span,
                })?;
                let rhs = self.add_expr()?.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'<<'演算子の右辺値がありません".to_string(),
                    span,
                })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Shl, lhs, rhs, span)?));
            } else if let Some(token) = self.consume_punct(">>") {
                let span = token.span;
                // right shift
                let lhs = node.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'>>'演算子の左辺値がありません".to_string(),
                    span,
                })?;
                let rhs = self.add_expr()?.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'>>'演算子の右辺値がありません".to_string(),
                    span,
                })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Shr, lhs, rhs, span)?));
            } else {
                return Ok(node);
            }
        }
    }

    // add_expr ::= mul_expr
    //            | add_expr ("+" | "-") mul_expr
    fn add_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node = self.mul_expr()?;

        loop {
            if let Some(token) = self.consume_punct("+") {
                let span = token.span;
                // addition
                if let Some(n) = node.take() {
                    let rhs = self.mul_expr()?.ok_or_else(|| CompileError::InvalidExpr {
                        msg: "'+'演算子の右辺値がありません".to_string(),
                        span,
                    })?;
                    node = Some(Box::new(Node::new_scaled_add(n, rhs, span)?));
                }
            } else if let Some(token) = self.consume_punct("-") {
                let span = token.span;
                // subtraction
                if let Some(n) = node.take() {
                    let rhs = self.mul_expr()?.ok_or_else(|| CompileError::InvalidExpr {
                        msg: "'-'演算子の右辺値がありません".to_string(),
                        span,
                    })?;
                    node = Some(Box::new(Node::new_scaled_sub(n, rhs, span)?));
                }
            } else {
                return Ok(node);
            }
        }
    }

    // mul_expr ::= cast_expr
    //            | mul_expr ("*" | "/" | "%") cast_expr
    fn mul_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node = self.cast_expr()?;

        loop {
            if let Some(token) = self.consume_punct("*") {
                let span = token.span;
                // multiplication
                let lhs = node.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'*'演算子の左辺値がありません".to_string(),
                    span,
                })?;
                let rhs = self.cast_expr()?.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'*'演算子の右辺値がありません".to_string(),
                    span,
                })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Mul, lhs, rhs, span)?));
            } else if let Some(token) = self.consume_punct("/") {
                let span = token.span;
                // division
                let lhs = node.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'/'演算子の左辺値がありません".to_string(),
                    span,
                })?;
                let rhs = self.cast_expr()?.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'/'演算子の右辺値がありません".to_string(),
                    span,
                })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Div, lhs, rhs, span)?));
            } else if let Some(token) = self.consume_punct("%") {
                let span = token.span;
                // remainder
                let lhs = node.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'%'演算子の左辺値がありません".to_string(),
                    span,
                })?;
                let rhs = self.cast_expr()?.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "'%'演算子の右辺値がありません".to_string(),
                    span,
                })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Rem, lhs, rhs, span)?));
            } else {
                return Ok(node);
            }
        }
    }

    // cast_expr ::= unary_expr
    //             | "(" type_name ")" cast_expr // TODO: 未実装
    fn cast_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        self.unary_expr()
    }

    // unary_expr ::= postfix_expr
    //              | ("++" | "--") unary_expr
    //              | ( "&" | "*" | "+" | "-" | "~" | "!") cast_expr
    //              | sizeof unary_expr
    //              | sizeof "(" type_name ")"
    fn unary_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        if let Some(token) = self.consume_punct("++") {
            let span = token.span;
            // pre-increment
            let node = self
                .unary_expr()?
                .ok_or_else(|| CompileError::InvalidExpr {
                    msg: "単項'++'の後に式がありません".to_string(),
                    span,
                })?;
            return Ok(Some(Box::new(Node::new_scaled_increment(
                node, true, span,
            )?)));
        }
        if let Some(token) = self.consume_punct("--") {
            let span = token.span;
            // pre-decrement
            let node = self
                .unary_expr()?
                .ok_or_else(|| CompileError::InvalidExpr {
                    msg: "単項'--'の後に式がありません".to_string(),
                    span,
                })?;
            return Ok(Some(Box::new(Node::new_scaled_decrement(
                node, true, span,
            )?)));
        }

        if self.consume_punct("+").is_some() {
            // unary plus
            return self.cast_expr();
        }
        if let Some(token) = self.consume_punct("-") {
            let span = token.span;
            // unary minus
            let expr = self.cast_expr()?.ok_or_else(|| CompileError::InvalidExpr {
                msg: "単項'-'の後に式がありません".to_string(),
                span,
            })?;
            return Ok(Some(Box::new(Node::new_binary(
                BinaryOp::Sub,
                Box::new(Node::new_num(0, span)),
                expr,
                span,
            )?)));
        }
        if let Some(token) = self.consume_punct("&") {
            let span = token.span;
            // address-of
            let expr = self.cast_expr()?.ok_or_else(|| CompileError::InvalidExpr {
                msg: "単項'&'の後に式がありません".to_string(),
                span,
            })?;
            return Ok(Some(Box::new(Node::new_unary(UnaryOp::Addr, expr, span)?)));
        }
        if let Some(token) = self.consume_punct("*") {
            let span = token.span;
            // dereference
            let expr = self.cast_expr()?.ok_or_else(|| CompileError::InvalidExpr {
                msg: "単項'*'の後に式がありません".to_string(),
                span,
            })?;
            return Ok(Some(Box::new(Node::new_unary(UnaryOp::Deref, expr, span)?)));
        }
        if let Some(token) = self.consume_punct("~") {
            let span = token.span;
            // bitwise not
            let expr = self.cast_expr()?.ok_or_else(|| CompileError::InvalidExpr {
                msg: "単項'~'の後に式がありません".to_string(),
                span,
            })?;
            return Ok(Some(Box::new(Node::new_unary(
                UnaryOp::BitNot,
                expr,
                span,
            )?)));
        }
        if let Some(token) = self.consume_punct("!") {
            let span = token.span;
            // logical not
            let expr = self.cast_expr()?.ok_or_else(|| CompileError::InvalidExpr {
                msg: "単項'!'の後に式がありません".to_string(),
                span,
            })?;
            return Ok(Some(Box::new(Node::new_unary(
                UnaryOp::LogicalNot,
                expr,
                span,
            )?)));
        }

        if let Some(token) = self.consume_keyword("sizeof") {
            let span = token.span;
            // sizeof ( type_name )
            if self.peek_punct("(") {
                let token_pos = self.token_pos;
                self.consume_punct("(");
                if let Ok(ty) = self.type_name() {
                    self.expect_punct(")")?;
                    let size = ty.size_of();
                    return Ok(Some(Box::new(Node::new_num(size as i64, span))));
                }
                self.token_pos = token_pos; // 型名をパースできなかった場合、トークン位置を元に戻す
            }

            // sizeof unary_expr
            let mut node = self.unary_expr()?;
            if let Some(n) = &mut node {
                let size = n.ty.size_of();
                return Ok(Some(Box::new(Node::new_num(size as i64, span))));
            }
        }

        self.postfix_expr()
    }

    // 識別子ノードを変数ノードに解決するヘルパー関数
    // その他のノードはそのまま返す
    fn resolve_ident_to_var(
        &mut self,
        node: Option<Box<Node>>,
    ) -> Result<Option<Box<Node>>, CompileError> {
        if let Some(n) = &node
            && let NodeKind::Ident { name } = &n.kind
        {
            // 変数参照
            if let Some(symbol_idx) = self.find_var(name) {
                // 変数ノードを作成
                let symbol = self.get_symbol_by_id(symbol_idx).ok_or_else(|| {
                    CompileError::InternalError {
                        msg: format!("変数シンボルが見つかりません: {}", name),
                    }
                })?;
                let node = Node::new_var(symbol_idx, symbol.ty.clone(), n.span);
                return Ok(Some(Box::new(node)));
            }
            Err(CompileError::UndefinedIdent {
                name: name.clone(),
                span: n.span,
            })?;
        }
        Ok(node)
    }

    // 構造体のメンバアクセスノードを作成するヘルパー関数
    fn create_member_access_node(
        &mut self,
        obj: Box<Node>,
        member_name: &str,
        span: (usize, usize),
    ) -> Result<Box<Node>, CompileError> {
        if !obj.ty.is_struct() {
            return Err(CompileError::InvalidExpr {
                msg: format!(
                    "型 '{:?}' は構造体型ではありません。メンバアクセス演算子 '.' は構造体型にのみ使用できます",
                    obj.ty
                ),
                span,
            });
        }

        let member_decl =
            obj.ty
                .find_struct_member(member_name)
                .ok_or_else(|| CompileError::InvalidExpr {
                    msg: format!(
                        "構造体のメンバ '{}' が見つかりません。利用可能なメンバを確認してください",
                        member_name
                    ),
                    span,
                })?;
        let member_offset = member_decl
            .offset
            .ok_or_else(|| CompileError::InternalError {
                msg: format!(
                    "構造体メンバ '{}' のオフセットが設定されていません",
                    member_name
                ),
            })?;

        let member_ty = member_decl.ty.clone();

        Ok(Box::new(Node::new_member(
            obj,
            member_name,
            member_offset,
            &member_ty,
            span,
        )))
    }

    // postfix_expr ::= primary_expr
    //                | postfix_expr "[" expr "]"
    //                | postfix_expr "(" argument_expr_list? ")"
    //                | postfix_expr "." ident
    //                | postfix_expr "->" ident
    //                | postfix_expr ("++" | "--")
    fn postfix_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node = self.primary_expr()?;

        loop {
            if let Some(token) = self.consume_punct("[") {
                let span = token.span;
                // 配列の場合は自動的にアドレスに変換
                // 例: a[0] -> *(a + 0)
                // 例: a[1][2] -> *(*(a + 1) + 2)
                node = self.resolve_ident_to_var(node)?; // 識別子を変数に割り当て
                let index_expr = self.expr()?.ok_or_else(|| CompileError::InternalError {
                    msg: "配列のインデックス計算に失敗しました".to_string(),
                })?;
                if let Some(n) = node.take() {
                    let scaled_add = Box::new(Node::new_scaled_add(n, index_expr, span)?);
                    node = Some(Box::new(Node::new_unary(UnaryOp::Deref, scaled_add, span)?));
                }
                self.expect_punct("]")?;
            } else if let Some(token) = self.consume_punct("(") {
                let span = token.span;
                // 関数呼び出し
                let args = self.argument_expr_list()?;
                self.expect_punct(")")?;
                let func_name = if let Some(n) = &node
                    && let NodeKind::Ident { name } = &n.kind
                {
                    name
                } else {
                    return Err(CompileError::InternalError {
                        msg: "関数呼び出しの関数名のパースに失敗しました".to_string(),
                    });
                };
                if let Some(symbol_idx) = self.symbol_table.find_symbol(func_name)
                    && let Some(symbol) = self.get_symbol_by_id(symbol_idx)
                {
                    if symbol.is_func()
                        && let TypeKind::Func { return_ty, .. } = &symbol.ty.kind
                    {
                        node = Some(Box::new(Node::new_call(
                            func_name,
                            args,
                            return_ty.deref().clone(),
                            span,
                        )));
                    } else {
                        return Err(CompileError::InvalidExpr {
                            msg: format!("'{}' は関数ではありません", func_name),
                            span,
                        });
                    }
                } else {
                    return Err(CompileError::UndefinedFunc {
                        name: func_name.clone(),
                        span,
                    });
                }
            } else if let Some(token) = self.consume_punct(".") {
                let span = token.span;
                // 構造体のメンバアクセス
                node = self.resolve_ident_to_var(node)?; // 識別子を変数に割り当て
                let (member_name, _) =
                    self.consume_ident()
                        .ok_or_else(|| CompileError::InvalidExpr {
                            msg: "構造体メンバアクセスのメンバ名がありません".to_string(),
                            span,
                        })?;
                let obj = node.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "構造体オブジェクトがありません".to_string(),
                    span,
                })?;
                node = Some(self.create_member_access_node(obj, &member_name, span)?);
            } else if let Some(token) = self.consume_punct("->") {
                let span = token.span;
                // 構造体ポインタのメンバアクセス
                // ptr->member は (*ptr).member と同等
                node = self.resolve_ident_to_var(node)?; // 識別子を変数に割り当て
                let (member_name, _) =
                    self.consume_ident()
                        .ok_or_else(|| CompileError::InvalidExpr {
                            msg: "構造体ポインタメンバアクセスのメンバ名がありません".to_string(),
                            span,
                        })?;
                let ptr = node.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "構造体ポインタがありません".to_string(),
                    span,
                })?;
                // ポインタであることを確認
                if !(ptr.ty.is_ptr() || ptr.ty.is_array()) {
                    return Err(CompileError::InvalidExpr {
                        msg: format!(
                            "型 '{:?}' はポインタ型または配列型ではありません。アロー演算子 '->' はポインタ型にのみ使用できます\n  ヒント: 通常の構造体変数には '.' 演算子を使用してください",
                            ptr.ty
                        ),
                        span: ptr.span,
                    });
                }
                // デリファレンスして構造体を取得
                let deref_node = Box::new(Node::new_unary(UnaryOp::Deref, ptr, span)?);
                node = Some(self.create_member_access_node(deref_node, &member_name, span)?);
            } else if let Some(token) = self.consume_punct("++") {
                let span = token.span;
                // post-increment
                node = self.resolve_ident_to_var(node)?; // 識別子を変数に割り当て
                let expr = node.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "単項'++'の前に式がありません".to_string(),
                    span,
                })?;
                node = Some(Box::new(Node::new_scaled_increment(expr, false, span)?));
            } else if let Some(token) = self.consume_punct("--") {
                let span = token.span;
                // post-decrement
                node = self.resolve_ident_to_var(node)?; // 識別子を変数に割り当て
                let expr = node.ok_or_else(|| CompileError::InvalidExpr {
                    msg: "単項'--'の前に式がありません".to_string(),
                    span,
                })?;
                node = Some(Box::new(Node::new_scaled_decrement(expr, false, span)?));
            } else {
                node = self.resolve_ident_to_var(node)?; // 識別子を変数に割り当て
                return Ok(node);
            }
        }
    }

    // argument_expr_list ::= assign_expr ("," assign_expr)*
    #[allow(clippy::vec_box)]
    fn argument_expr_list(&mut self) -> Result<Vec<Node>, CompileError> {
        let mut args = Vec::new();
        if let Some(arg) = self.assign_expr()? {
            args.push(*arg);
        } else {
            return Ok(args);
        }

        while let Some(token) = self.consume_punct(",") {
            let span = token.span;
            if let Some(arg) = self.assign_expr()? {
                args.push(*arg);
            } else {
                return Err(CompileError::InvalidExpr {
                    msg: "関数呼び出しの引数リストのパースに失敗しました".to_string(),
                    span,
                })?;
            }
        }
        Ok(args)
    }

    // primary_expr ::= "(" expr ")"
    //                | ident
    //                | string
    //                | number
    fn primary_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        // "(" expr ")"
        if self.consume_punct("(").is_some()
            && let Some(node) = self.expr()?
        {
            self.expect_punct(")")?;
            return Ok(Some(node));
        }

        if let Some((name, token)) = self.consume_ident() {
            let span = token.span;
            return Ok(Some(Box::new(Node::new(NodeKind::Ident { name }, span))));
        }

        if let Some((val, token)) = self.consume_string() {
            let span = token.span;
            let index = self.register_string_literal(&val);
            return Ok(Some(Box::new(Node::new(
                NodeKind::String { val, index },
                span,
            ))));
        }

        if let Some((num, token)) = self.consume_number() {
            let span = token.span;
            return Ok(Some(Box::new(Node::new_num(num, span))));
        }

        Ok(None)
    }
}
