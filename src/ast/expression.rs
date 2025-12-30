use crate::ast::Ast;
use crate::errors::CompileError;
use crate::node::{BinaryOp, Node, NodeKind, UnaryOp};
use core::str::FromStr;

impl<'a> Ast<'a> {
    // const_expr ::= cond_expr
    #[allow(dead_code)]
    pub(super) fn const_expr(&mut self) -> Result<i64, CompileError> {
        let node = self
            .cond_expr()?
            .ok_or_else(|| CompileError::InvalidExpression {
                msg: "定数式がありません".to_string(),
            })?;
        node.eval_const_expr() // 定数式を評価
    }

    // expr ::= assign_expr
    pub(super) fn expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        self.assign_expr()
    }

    // assign_expr ::= cond_expr
    //                 | ("=" | "*=" | "/=" | "%=" | "+=" | "-=" | "<<=" | ">>=" | "&=" | "^=" | "|=") assign_expr
    pub(super) fn assign_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node = self.cond_expr()?;
        let assign_op_str_list = [
            "=", "*=", "/=", "%=", "+=", "-=", "<<=", ">>=", "&=", "^=", "|=",
        ];
        for assign_op_str in &assign_op_str_list {
            if let Some(token) = self.consume_punctuator(assign_op_str)
                && let Ok(kind) = BinaryOp::from_str(assign_op_str)
            {
                let span = token.span;
                let lhs = node.ok_or_else(|| CompileError::InvalidExpression {
                    msg: "代入式の左辺値がありません".to_string(),
                })?;
                if let NodeKind::Var { name, .. } = &lhs.kind
                    && lhs.ty.is_const
                {
                    return Err(CompileError::ReadOnlyLvalue { name: name.clone() });
                }
                let rhs = self
                    .assign_expr()?
                    .ok_or_else(|| CompileError::InvalidExpression {
                        msg: "代入式の右辺値がありません".to_string(),
                    })?;
                node = Some(Box::new(Node::new_assign(kind, lhs, rhs, span)));
                break;
            }
        }
        Ok(node)
    }

    // cond_expr ::= logical_or_expr
    //               | logical_or_expr "?" expr ":" cond_expr
    fn cond_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let node = self.logical_or_expr()?;
        if let Some(token) = self.consume_punctuator("?") {
            let span = token.span;
            let cond = node.ok_or_else(|| CompileError::InvalidExpression {
                msg: "三項演算子の条件式がありません".to_string(),
            })?;
            let then = self
                .expr()?
                .ok_or_else(|| CompileError::InvalidExpression {
                    msg: "三項演算子のthenの場合の式がありません".to_string(),
                })?;
            self.expect_punctuator(":")?;
            let els = self
                .cond_expr()?
                .ok_or_else(|| CompileError::InvalidExpression {
                    msg: "三項演算子のelseの場合の式がありません".to_string(),
                })?;
            return Ok(Some(Box::new(Node::new_ternary(cond, then, els, span)?)));
        }
        Ok(node)
    }

    // logical_or_expr ::= logical_and_expr
    //                     | logical_or_expr "||" logical_and_expr
    fn logical_or_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node = self.logical_and_expr()?;

        loop {
            if let Some(token) = self.consume_punctuator("||") {
                let span = token.span;
                // logical or
                let lhs = node.ok_or_else(|| CompileError::InvalidExpression {
                    msg: "'||'演算子の左辺値がありません".to_string(),
                })?;
                let rhs =
                    self.logical_and_expr()?
                        .ok_or_else(|| CompileError::InvalidExpression {
                            msg: "'||'演算子の右辺値がありません".to_string(),
                        })?;
                node = Some(Box::new(Node::new_logical_or(lhs, rhs, span)?));
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
            if let Some(token) = self.consume_punctuator("&&") {
                let span = token.span;
                // logical and
                let lhs = node.ok_or_else(|| CompileError::InvalidExpression {
                    msg: "'&&'演算子の左辺値がありません".to_string(),
                })?;
                let rhs =
                    self.inclusive_or_expr()?
                        .ok_or_else(|| CompileError::InvalidExpression {
                            msg: "'&&'演算子の右辺値がありません".to_string(),
                        })?;
                node = Some(Box::new(Node::new_logical_and(lhs, rhs, span)?));
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
            if let Some(token) = self.consume_punctuator("|") {
                let span = token.span;
                // bitwise or
                let lhs = node.ok_or_else(|| CompileError::InvalidExpression {
                    msg: "'|'演算子の左辺値がありません".to_string(),
                })?;
                let rhs =
                    self.exclusive_or_expr()?
                        .ok_or_else(|| CompileError::InvalidExpression {
                            msg: "'|'演算子の右辺値がありません".to_string(),
                        })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::BitOr, lhs, rhs, span)?));
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
            if let Some(token) = self.consume_punctuator("^") {
                let span = token.span;
                // bitwise xor
                let lhs = node.ok_or_else(|| CompileError::InvalidExpression {
                    msg: "'^'演算子の左辺値がありません".to_string(),
                })?;
                let rhs = self
                    .and_expr()?
                    .ok_or_else(|| CompileError::InvalidExpression {
                        msg: "'^'演算子の右辺値がありません".to_string(),
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
    //              | and_expr "&" equality_expr
    fn and_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node = self.equality_expr()?;

        loop {
            if let Some(token) = self.consume_punctuator("&") {
                let span = token.span;
                //bitwise and
                let lhs = node.ok_or_else(|| CompileError::InvalidExpression {
                    msg: "'&'演算子の左辺値がありません".to_string(),
                })?;
                let rhs = self
                    .equality_expr()?
                    .ok_or_else(|| CompileError::InvalidExpression {
                        msg: "'&'演算子の右辺値がありません".to_string(),
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
    //                   | equality_expr ("==" | "!=") relational_expr
    fn equality_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node = self.relational_expr()?;

        loop {
            if let Some(token) = self.consume_punctuator("==") {
                let span = token.span;
                // equal
                let lhs = node.ok_or_else(|| CompileError::InvalidExpression {
                    msg: "'=='演算子の左辺値がありません".to_string(),
                })?;
                let rhs =
                    self.relational_expr()?
                        .ok_or_else(|| CompileError::InvalidExpression {
                            msg: "'=='演算子の右辺値がありません".to_string(),
                        })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Eq, lhs, rhs, span)?));
            } else if let Some(token) = self.consume_punctuator("!=") {
                let span = token.span;
                // not equal
                let lhs = node.ok_or_else(|| CompileError::InvalidExpression {
                    msg: "'!='演算子の左辺値がありません".to_string(),
                })?;
                let rhs =
                    self.relational_expr()?
                        .ok_or_else(|| CompileError::InvalidExpression {
                            msg: "'!='演算子の右辺値がありません".to_string(),
                        })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Ne, lhs, rhs, span)?));
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
            if let Some(token) = self.consume_punctuator("<") {
                let span = token.span;
                // less than
                let lhs = node.ok_or_else(|| CompileError::InvalidExpression {
                    msg: "'<'演算子の左辺値がありません".to_string(),
                })?;
                let rhs = self
                    .shift_expr()?
                    .ok_or_else(|| CompileError::InvalidExpression {
                        msg: "'<'演算子の右辺値がありません".to_string(),
                    })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Lt, lhs, rhs, span)?));
            } else if let Some(token) = self.consume_punctuator("<=") {
                let span = token.span;
                // less than or equal
                let lhs = node.ok_or_else(|| CompileError::InvalidExpression {
                    msg: "'<='演算子の左辺値がありません".to_string(),
                })?;
                let rhs = self
                    .shift_expr()?
                    .ok_or_else(|| CompileError::InvalidExpression {
                        msg: "'<='演算子の右辺値がありません".to_string(),
                    })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Le, lhs, rhs, span)?));
            } else if let Some(token) = self.consume_punctuator(">") {
                let span = token.span;
                // greater than
                let lhs = self
                    .shift_expr()?
                    .ok_or_else(|| CompileError::InvalidExpression {
                        msg: "'>'演算子の左辺値がありません".to_string(),
                    })?;
                let rhs = node.ok_or_else(|| CompileError::InvalidExpression {
                    msg: "'>'演算子の右辺値がありません".to_string(),
                })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Lt, lhs, rhs, span)?));
            } else if let Some(token) = self.consume_punctuator(">=") {
                let span = token.span;
                // greater than or equal
                let lhs = self
                    .shift_expr()?
                    .ok_or_else(|| CompileError::InvalidExpression {
                        msg: "'>='演算子の左辺値がありません".to_string(),
                    })?;
                let rhs = node.ok_or_else(|| CompileError::InvalidExpression {
                    msg: "'>='演算子の右辺値がありません".to_string(),
                })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Le, lhs, rhs, span)?));
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
            if let Some(token) = self.consume_punctuator("<<") {
                let span = token.span;
                // left shift
                let lhs = node.ok_or_else(|| CompileError::InvalidExpression {
                    msg: "'<<'演算子の左辺値がありません".to_string(),
                })?;
                let rhs = self
                    .add_expr()?
                    .ok_or_else(|| CompileError::InvalidExpression {
                        msg: "'<<'演算子の右辺値がありません".to_string(),
                    })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Shl, lhs, rhs, span)?));
            } else if let Some(token) = self.consume_punctuator(">>") {
                let span = token.span;
                // right shift
                let lhs = node.ok_or_else(|| CompileError::InvalidExpression {
                    msg: "'>>'演算子の左辺値がありません".to_string(),
                })?;
                let rhs = self
                    .add_expr()?
                    .ok_or_else(|| CompileError::InvalidExpression {
                        msg: "'>>'演算子の右辺値がありません".to_string(),
                    })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Shr, lhs, rhs, span)?));
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
            if let Some(token) = self.consume_punctuator("+") {
                let span = token.span;
                // addition
                if let Some(n) = node.take() {
                    let rhs = self
                        .mul_expr()?
                        .ok_or_else(|| CompileError::InvalidExpression {
                            msg: "'+'演算子の右辺値がありません".to_string(),
                        })?;
                    node = Some(Box::new(Node::new_scaled_add(n, rhs, span)?));
                }
            } else if let Some(token) = self.consume_punctuator("-") {
                let span = token.span;
                // subtraction
                if let Some(n) = node.take() {
                    let rhs = self
                        .mul_expr()?
                        .ok_or_else(|| CompileError::InvalidExpression {
                            msg: "'-'演算子の右辺値がありません".to_string(),
                        })?;
                    node = Some(Box::new(Node::new_scaled_sub(n, rhs, span)?));
                }
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
            if let Some(token) = self.consume_punctuator("*") {
                let span = token.span;
                // multiplication
                let lhs = node.ok_or_else(|| CompileError::InvalidExpression {
                    msg: "'*'演算子の左辺値がありません".to_string(),
                })?;
                let rhs = self
                    .cast_expr()?
                    .ok_or_else(|| CompileError::InvalidExpression {
                        msg: "'*'演算子の右辺値がありません".to_string(),
                    })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Mul, lhs, rhs, span)?));
            } else if let Some(token) = self.consume_punctuator("/") {
                let span = token.span;
                // division
                let lhs = node.ok_or_else(|| CompileError::InvalidExpression {
                    msg: "'/'演算子の左辺値がありません".to_string(),
                })?;
                let rhs = self
                    .cast_expr()?
                    .ok_or_else(|| CompileError::InvalidExpression {
                        msg: "'/'演算子の右辺値がありません".to_string(),
                    })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Div, lhs, rhs, span)?));
            } else if let Some(token) = self.consume_punctuator("%") {
                let span = token.span;
                // remainder
                let lhs = node.ok_or_else(|| CompileError::InvalidExpression {
                    msg: "'%'演算子の左辺値がありません".to_string(),
                })?;
                let rhs = self
                    .cast_expr()?
                    .ok_or_else(|| CompileError::InvalidExpression {
                        msg: "'%'演算子の右辺値がありません".to_string(),
                    })?;
                node = Some(Box::new(Node::new_binary(BinaryOp::Rem, lhs, rhs, span)?));
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
        if let Some(token) = self.consume_punctuator("++") {
            let span = token.span;
            // pre-increment
            let node = self
                .unary_expr()?
                .ok_or_else(|| CompileError::InvalidExpression {
                    msg: "単項'++'の後に式がありません".to_string(),
                })?;
            return Ok(Some(Box::new(Node::new_scaled_increment(
                node, true, span,
            )?)));
        }
        if let Some(token) = self.consume_punctuator("--") {
            let span = token.span;
            // pre-decrement
            let node = self
                .unary_expr()?
                .ok_or_else(|| CompileError::InvalidExpression {
                    msg: "単項'--'の後に式がありません".to_string(),
                })?;
            return Ok(Some(Box::new(Node::new_scaled_decrement(
                node, true, span,
            )?)));
        }

        if self.consume_punctuator("+").is_some() {
            // unary plus
            return self.cast_expr();
        }
        if let Some(token) = self.consume_punctuator("-") {
            let span = token.span;
            // unary minus
            let expr = self
                .cast_expr()?
                .ok_or_else(|| CompileError::InvalidExpression {
                    msg: "単項'-'の後に式がありません".to_string(),
                })?;
            return Ok(Some(Box::new(Node::new_binary(
                BinaryOp::Sub,
                Box::new(Node::new_num(0, span)),
                expr,
                span,
            )?)));
        }
        if let Some(token) = self.consume_punctuator("&") {
            let span = token.span;
            // address-of
            let expr = self
                .cast_expr()?
                .ok_or_else(|| CompileError::InvalidExpression {
                    msg: "単項'&'の後に式がありません".to_string(),
                })?;
            return Ok(Some(Box::new(Node::new_unary(UnaryOp::Addr, expr, span)?)));
        }
        if let Some(token) = self.consume_punctuator("*") {
            let span = token.span;
            // dereference
            let expr = self
                .cast_expr()?
                .ok_or_else(|| CompileError::InvalidExpression {
                    msg: "単項'*'の後に式がありません".to_string(),
                })?;
            return Ok(Some(Box::new(Node::new_unary(UnaryOp::Deref, expr, span)?)));
        }
        if let Some(token) = self.consume_punctuator("~") {
            let span = token.span;
            // bitwise not
            let expr = self
                .cast_expr()?
                .ok_or_else(|| CompileError::InvalidExpression {
                    msg: "単項'~'の後に式がありません".to_string(),
                })?;
            return Ok(Some(Box::new(Node::new_unary(
                UnaryOp::BitNot,
                expr,
                span,
            )?)));
        }
        if let Some(token) = self.consume_punctuator("!") {
            let span = token.span;
            // logical not
            let expr = self
                .cast_expr()?
                .ok_or_else(|| CompileError::InvalidExpression {
                    msg: "単項'!'の後に式がありません".to_string(),
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
            if self.peek_punctuator("(") {
                let token_pos = self.token_pos;
                self.consume_punctuator("(");
                if let Ok(ty) = self.type_name() {
                    self.expect_punctuator(")")?;
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
            && let NodeKind::Identifier { name } = &n.kind
        {
            // 変数参照
            if let Ok(current_func) = self.get_current_func() {
                if let Some(symbol) = current_func.find_local_var(name) {
                    // ローカル変数ノードを作成
                    let node = Node::new_var(name, symbol.offset, &symbol.ty, true, n.span);
                    return Ok(Some(Box::new(node)));
                } else if let Some(symbol) = current_func.find_param(name) {
                    // 引数変数ノードを作成
                    let node = Node::new_var(name, symbol.offset, &symbol.ty, true, n.span);
                    return Ok(Some(Box::new(node)));
                }
            }
            if let Some(symbol) = self.find_global_var(name) {
                // グローバル変数ノードを作成
                let node = Node::new_var(name, 0, &symbol.ty, false, n.span);
                return Ok(Some(Box::new(node)));
            }
            Err(CompileError::UndefinedIdentifier { name: name.clone() })?;
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
            return Err(CompileError::InvalidExpression {
                msg: format!(
                    "型 '{:?}' は構造体ではないため、メンバアクセスできません",
                    obj.ty
                ),
            });
        }

        let member_decl = obj.ty.find_struct_member(member_name).ok_or_else(|| {
            CompileError::InvalidExpression {
                msg: format!("構造体に指定されたメンバ {} が存在しません", member_name),
            }
        })?;
        let member_offset = member_decl
            .offset
            .ok_or_else(|| CompileError::InternalError {
                msg: format!(
                    "構造体メンバ {:?} のオフセットが設定されていません",
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
    //                  | postfix_expr "[" expr "]"
    //                  | postfix_expr "(" argument_expr_list? ")"
    //                  | postfix_expr "." identifier
    //                  | postfix_expr "->" identifier
    //                  | postfix_expr ("++" | "--")
    fn postfix_expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node = self.primary_expr()?;

        loop {
            if let Some(token) = self.consume_punctuator("[") {
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
                self.expect_punctuator("]")?;
            } else if let Some(token) = self.consume_punctuator("(") {
                let span = token.span;
                // 関数呼び出し
                let args = self.argument_expr_list()?;
                self.expect_punctuator(")")?;
                let func_name = if let Some(n) = &node
                    && let NodeKind::Identifier { name } = &n.kind
                {
                    name
                } else {
                    return Err(CompileError::InternalError {
                        msg: "関数呼び出しの関数名のパースに失敗しました".to_string(),
                    });
                };
                let return_ty = self
                    .get_function_return_type(func_name)
                    .cloned()
                    .unwrap_or_default();
                node = Some(Box::new(Node::new_call(func_name, args, return_ty, span)));
            } else if let Some(token) = self.consume_punctuator(".") {
                let span = token.span;
                // 構造体のメンバアクセス
                node = self.resolve_ident_to_var(node)?; // 識別子を変数に割り当て
                let (member_name, _) =
                    self.consume_ident()
                        .ok_or_else(|| CompileError::InvalidExpression {
                            msg: "構造体メンバアクセスのメンバ名がありません".to_string(),
                        })?;
                let obj = node.ok_or_else(|| CompileError::InvalidExpression {
                    msg: "構造体オブジェクトがありません".to_string(),
                })?;
                node = Some(self.create_member_access_node(obj, &member_name, span)?);
            } else if let Some(token) = self.consume_punctuator("->") {
                let span = token.span;
                // 構造体ポインタのメンバアクセス
                // ptr->member は (*ptr).member と同等
                node = self.resolve_ident_to_var(node)?; // 識別子を変数に割り当て
                let (member_name, _) =
                    self.consume_ident()
                        .ok_or_else(|| CompileError::InvalidExpression {
                            msg: "構造体ポインタメンバアクセスのメンバ名がありません".to_string(),
                        })?;
                let ptr = node.ok_or_else(|| CompileError::InvalidExpression {
                    msg: "構造体ポインタがありません".to_string(),
                })?;
                // ポインタであることを確認
                if !(ptr.ty.is_ptr() || ptr.ty.is_array()) {
                    return Err(CompileError::InvalidExpression {
                        msg: format!(
                            "型 '{:?}' はポインタではないため、'->'演算子を使用できません",
                            ptr.ty
                        ),
                    });
                }
                // デリファレンスして構造体を取得
                let deref_node = Box::new(Node::new_unary(UnaryOp::Deref, ptr, span)?);
                node = Some(self.create_member_access_node(deref_node, &member_name, span)?);
            } else if let Some(token) = self.consume_punctuator("++") {
                let span = token.span;
                // post-increment
                node = self.resolve_ident_to_var(node)?; // 識別子を変数に割り当て
                let expr = node.ok_or_else(|| CompileError::InvalidExpression {
                    msg: "単項'++'の前に式がありません".to_string(),
                })?;
                node = Some(Box::new(Node::new_scaled_increment(expr, false, span)?));
            } else if let Some(token) = self.consume_punctuator("--") {
                let span = token.span;
                // post-decrement
                node = self.resolve_ident_to_var(node)?; // 識別子を変数に割り当て
                let expr = node.ok_or_else(|| CompileError::InvalidExpression {
                    msg: "単項'--'の前に式がありません".to_string(),
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

        while self.consume_punctuator(",").is_some() {
            if let Some(arg) = self.assign_expr()? {
                args.push(*arg);
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

        if let Some((name, token)) = self.consume_ident() {
            let span = token.span;
            return Ok(Some(Box::new(Node::new(
                NodeKind::Identifier { name },
                span,
            ))));
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
