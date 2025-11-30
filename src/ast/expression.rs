use core::str::FromStr;

use crate::ast::Ast;
use crate::errors::CompileError;
use crate::node::{Node, NodeKind};

impl Ast {
    // const_expr ::= cond_expr
    #[allow(dead_code)]
    pub(super) fn const_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        self.cond_expr()
    }

    // expr ::= assign_expr
    pub(super) fn expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        self.assign_expr()
    }

    // assign_expr ::= cond_expr
    //                 | ("=" | "*=" | "/=" | "%=" | "+=" | "-=" | "<<=" | ">>=" | "&=" | "^=" | "|=") assign_expr
    pub(super) fn assign_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
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
    fn cond_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
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
    fn logical_or_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
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
    fn logical_and_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
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
    fn inclusive_or_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
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
    fn exclusive_or_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
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
    fn and_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
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
    fn equality_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
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
    fn relational_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
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
    fn shift_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
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
    fn add_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
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
                        Some(Box::new(Node::new_num(size as i64))),
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
                        Some(Box::new(Node::new_num(size as i64))),
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
    fn mul_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
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
    fn cast_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        self.unary_expr()
    }

    // unary_expr ::= postfix_expr
    //                | ("++" | "--") unary_expr
    //                | ( "&" | "*" | "+" | "-" | "~" | "!") cast_expr
    //                | sizeof unary_expr
    //                | sizeof "(" type_name ")"
    fn unary_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
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
                return Err(CompileError::InternalError {
                    msg: "＆演算子の型情報が設定されていません".to_string(),
                })?;
            }
            return Ok(Some(node));
        }
        if self.consume_punctuator("*").is_some() {
            // dereference
            let mut node = Box::new(Node::new_unary(NodeKind::Deref, self.cast_expr()?));
            node.assign_types()?;
            if node.ty.is_none() {
                return Err(CompileError::InternalError {
                    msg: "*演算子の型情報が設定されていません".to_string(),
                })?;
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
            // sizeof ( type_name )
            if self.peek_punctuator("(") {
                let token_pos = self.token_pos;
                self.consume_punctuator("(");
                if let Ok(ty) = self.type_name() {
                    self.expect_punctuator(")")?;
                    let size = ty.size_of();
                    return Ok(Some(Box::new(Node::new_num(size as i64))));
                }
                self.token_pos = token_pos; // 型名をパースできなかった場合、トークン位置を元に戻す
            }

            // sizeof unary_expr
            let mut node = self.unary_expr()?;
            if let Some(n) = &mut node {
                n.assign_types()?;
                if let Some(ty) = &n.ty {
                    let size = ty.size_of();
                    return Ok(Some(Box::new(Node::new_num(size as i64))));
                } else {
                    return Err(CompileError::InternalError {
                        msg: "sizeof演算子の型情報が設定されていません".to_string(),
                    })?;
                }
            }
        }

        self.postfix_expr()
    }

    // 未確定の識別子をローカル変数またはグローバル変数に割り当てる
    // その他のノードはそのまま返す
    fn assign_identifier(
        &mut self,
        node: Option<Box<Node>>,
    ) -> Result<Option<Box<Node>>, CompileError> {
        if let Some(n) = &node
            && let NodeKind::Identifier { name } = &n.kind
        {
            // 変数参照
            if let Ok(current_func) = self.get_current_func()
                && let Some(lvar) = current_func.find_lvar(name)
            {
                // ローカル変数ノードを作成
                let node = Node::new_var(&lvar.name, lvar.offset, &lvar.ty, true);
                return Ok(Some(Box::new(node)));
            } else if let Some(gvar) = self.find_gvar(name) {
                // グローバル変数ノードを作成
                let node = Node::new_var(&gvar.name, 0, &gvar.ty, false);
                return Ok(Some(Box::new(node)));
            }
            Err(CompileError::UndefinedIdentifier { name: name.clone() })?;
        }
        Ok(node)
    }

    // postfix_expr ::= primary_expr
    //                  | postfix_expr "[" expr "]"
    //                  | postfix_expr "(" argument_expr_list? ")"
    //                  | postfix_expr "." identifier
    //                  | postfix_expr "->" identifier
    //                  | postfix_expr ("++" | "--")
    fn postfix_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node = self.primary_expr()?;

        loop {
            if self.consume_punctuator("[").is_some() {
                // 配列の場合は自動的にアドレスに変換
                // 例: a[0] -> *(a + 0)
                // 例: a[1][2] -> *(*(a + 1) + 2)
                // TODO: 多次元配列アクセスの際の問題点の修正
                node = self.assign_identifier(node)?; // 識別子を変数に割り当て
                let add_node = Node::new(NodeKind::Add, node, self.expr()?);
                node = Some(Box::new(Node::new_unary(
                    NodeKind::Deref,
                    Some(Box::new(add_node)),
                )));
                node.as_mut().unwrap().assign_types()?;
                self.expect_punctuator("]")?;
            } else if self.consume_punctuator("(").is_some() {
                let args = self.argument_expr_list()?;
                self.expect_punctuator(")")?;
                node = Some(Box::new(Node::from(NodeKind::Call {
                    name: if let Some(n) = &node
                        && let NodeKind::Identifier { name } = &n.kind
                    {
                        name.clone()
                    } else {
                        return Err(CompileError::InternalError {
                            msg: "関数呼び出しの関数名のパースに失敗しました".to_string(),
                        });
                    },
                    args,
                })));
            } else if self.consume_punctuator(".").is_some() {
                unimplemented!("構造体メンバアクセスは未実装です");
            } else if self.consume_punctuator("->").is_some() {
                unimplemented!("構造体ポインタメンバアクセスは未実装です");
            } else if self.consume_punctuator("++").is_some() {
                // post-increment
                node = self.assign_identifier(node)?; // 識別子を変数に割り当て
                node = Some(Box::new(Node::new_unary(NodeKind::PostInc, node)));
            } else if self.consume_punctuator("--").is_some() {
                // post-decrement
                node = self.assign_identifier(node)?; // 識別子を変数に割り当て
                node = Some(Box::new(Node::new_unary(NodeKind::PostDec, node)));
            } else {
                node = self.assign_identifier(node)?; // 識別子を変数に割り当て
                return Ok(node);
            }
        }
    }

    // argument_expr_list ::= assign_expr ("," assign_expr)*
    #[allow(clippy::vec_box)]
    fn argument_expr_list(&mut self) -> Result<Vec<Box<Node>>, CompileError> {
        let mut args = Vec::new();
        if let Some(arg) = self.assign_expr()? {
            args.push(arg);
        } else {
            return Ok(args);
        }

        while self.consume_punctuator(",").is_some() {
            if let Some(arg) = self.assign_expr()? {
                args.push(arg);
            } else {
                return Err(CompileError::InternalError {
                    msg: "関数呼び出しの引数リストのパースに失敗しました".to_string(),
                })?;
            }
        }
        Ok(args)
    }

    // primary_expr ::= "(" expr ")"
    //                  | identifier
    //                  | string
    //                  | number
    fn primary_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        // "(" expr ")"
        if self.consume_punctuator("(").is_some()
            && let Some(node) = self.expr()?
        {
            self.expect_punctuator(")")?;
            return Ok(Some(node));
        }

        if let Some(name) = self.consume_ident() {
            let node = Node::from(NodeKind::Identifier { name: name.clone() });
            return Ok(Some(Box::new(node)));
        }

        if let Some(string) = self.consume_string() {
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
