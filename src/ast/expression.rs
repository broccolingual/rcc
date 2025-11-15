use core::str::FromStr;

use crate::ast::Ast;
use crate::node::{Node, NodeKind};
use crate::types::{Type, TypeKind};

impl Ast {
    // constant_expr ::= conditional_expr
    #[allow(dead_code)]
    pub(super) fn constant_expr(&mut self) -> Option<Box<Node>> {
        self.conditional_expr()
    }

    // expr ::= assign_expr
    pub(super) fn expr(&mut self) -> Option<Box<Node>> {
        self.assign_expr()
    }

    // assign_expr ::= conditional_expr
    //                 | ("=" | "*=" | "/=" | "%=" | "+=" | "-=" | "<<=" | ">>=" | "&=" | "^=" | "|=") assign_expr
    fn assign_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.conditional_expr();
        let assignment_ops = [
            "=", "*=", "/=", "%=", "+=", "-=", "<<=", ">>=", "&=", "^=", "|=",
        ];
        for op in &assignment_ops {
            if self.consume_punctuator(op) {
                let kind = NodeKind::from_str(op).unwrap();
                node = Some(Box::new(Node::new(kind, node, self.assign_expr())));
                break;
            }
        }
        node
    }

    // conditional_expr ::= logical_or_expr
    //                      | logical_or_expr "?" expr ":" conditional_expr
    fn conditional_expr(&mut self) -> Option<Box<Node>> {
        let node = self.logical_or_expr();
        if self.consume_punctuator("?") {
            let mut ternary_node = Node::from(NodeKind::Ternary);
            ternary_node.cond = node;
            ternary_node.then = self.expr();
            self.expect_punctuator(":").unwrap();
            ternary_node.els = self.conditional_expr();
            return Some(Box::new(ternary_node));
        }
        node
    }

    // logical_or_expr ::= logical_and_expr
    //                     | logical_or_expr "||" logical_and_expr
    fn logical_or_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.logical_and_expr();

        loop {
            if self.consume_punctuator("||") {
                // logical or
                node = Some(Box::new(Node::new(
                    NodeKind::LogicalOr,
                    node,
                    self.logical_and_expr(),
                )));
            } else {
                return node;
            }
        }
    }

    // logical_and_expr ::= inclusive_or_expr
    //                      | logical_and_expr "&&" inclusive_or_expr
    fn logical_and_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.inclusive_or_expr();

        loop {
            if self.consume_punctuator("&&") {
                // logical and
                node = Some(Box::new(Node::new(
                    NodeKind::LogicalAnd,
                    node,
                    self.inclusive_or_expr(),
                )));
            } else {
                return node;
            }
        }
    }

    // inclusive_or_expr ::= exclusive_or_expr
    //                       | inclusive_or_expr "|" exclusive_or_expr
    fn inclusive_or_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.exclusive_or_expr();

        loop {
            if self.consume_punctuator("|") {
                // bitwise or
                node = Some(Box::new(Node::new(
                    NodeKind::BitOr,
                    node,
                    self.exclusive_or_expr(),
                )));
            } else {
                return node;
            }
        }
    }

    // exclusive_or_expr ::= and_expr
    //                       | exclusive_or_expr "^" and_expr
    fn exclusive_or_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.and_expr();

        loop {
            if self.consume_punctuator("^") {
                // bitwise xor
                node = Some(Box::new(Node::new(NodeKind::BitXor, node, self.and_expr())));
            } else {
                return node;
            }
        }
    }

    // and_expr ::= equality_expr
    //              | and_expr "&" equality_expr
    fn and_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.equality_expr();

        loop {
            if self.consume_punctuator("&") {
                //bitwise and
                node = Some(Box::new(Node::new(
                    NodeKind::BitAnd,
                    node,
                    self.equality_expr(),
                )));
            } else {
                return node;
            }
        }
    }

    // equality_expr ::= relational_expr
    //                   | equality_expr ("==" | "!=") relational_expr
    fn equality_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.relational_expr();

        loop {
            if self.consume_punctuator("==") {
                // equal
                node = Some(Box::new(Node::new(
                    NodeKind::Eq,
                    node,
                    self.relational_expr(),
                )));
            } else if self.consume_punctuator("!=") {
                // not equal
                node = Some(Box::new(Node::new(
                    NodeKind::Ne,
                    node,
                    self.relational_expr(),
                )));
            } else {
                return node;
            }
        }
    }

    // relational_expr ::= shift_expr
    //                     | relational_expr ("<" | "<=" | ">" | ">=") shift_expr
    fn relational_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.shift_expr();

        loop {
            if self.consume_punctuator("<") {
                // less than
                node = Some(Box::new(Node::new(NodeKind::Lt, node, self.shift_expr())));
            } else if self.consume_punctuator("<=") {
                // less than or equal
                node = Some(Box::new(Node::new(NodeKind::Le, node, self.shift_expr())));
            } else if self.consume_punctuator(">") {
                // greater than
                node = Some(Box::new(Node::new(NodeKind::Lt, self.shift_expr(), node)));
            } else if self.consume_punctuator(">=") {
                // greater than or equal
                node = Some(Box::new(Node::new(NodeKind::Le, self.shift_expr(), node)));
            } else {
                return node;
            }
        }
    }

    // shift_expr ::= add_expr
    //                | shift_expr ("<<" | ">>") add_expr
    fn shift_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.add_expr();

        loop {
            if self.consume_punctuator("<<") {
                // left shift
                node = Some(Box::new(Node::new(NodeKind::Shl, node, self.add_expr())));
            } else if self.consume_punctuator(">>") {
                // right shift
                node = Some(Box::new(Node::new(NodeKind::Shr, node, self.add_expr())));
            } else {
                return node;
            }
        }
    }

    // add_expr ::= mul_expr
    //              | add_expr ("+" | "-") mul_expr
    fn add_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.mul_expr();

        loop {
            if self.consume_punctuator("+") {
                // addition
                self.assign_types(&mut node); // lhs
                let mut rhs = self.mul_expr();
                self.assign_types(&mut rhs); // rhs
                if let Some(ty) = &node.as_ref().unwrap().ty
                    && (ty.kind == TypeKind::Ptr || ty.kind == TypeKind::Array)
                {
                    // ポインタ加算の場合、スケーリングを考慮
                    let size = ty.ptr_to.as_ref().unwrap().size_of();
                    rhs = Some(Box::new(Node::new(
                        NodeKind::Mul,
                        rhs,
                        Some(Box::new(Node::new_num(size))),
                    )));
                }
                node = Some(Box::new(Node::new(NodeKind::Add, node, rhs)));
            } else if self.consume_punctuator("-") {
                // subtraction
                self.assign_types(&mut node); // lhs
                let mut rhs = self.mul_expr();
                self.assign_types(&mut rhs); // rhs
                if let Some(ty) = &node.as_ref().unwrap().ty
                    && (ty.kind == TypeKind::Ptr || ty.kind == TypeKind::Array)
                {
                    // ポインタ減算の場合、スケーリングを考慮
                    let size = ty.ptr_to.as_ref().unwrap().size_of();
                    rhs = Some(Box::new(Node::new(
                        NodeKind::Mul,
                        rhs,
                        Some(Box::new(Node::new_num(size))),
                    )));
                }
                node = Some(Box::new(Node::new(NodeKind::Sub, node, rhs)));
            } else {
                return node;
            }
        }
    }

    // mul_expr ::= cast_expr
    //              | mul_expr ("*" | "/" | "%") cast_expr
    fn mul_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.cast_expr();

        loop {
            if self.consume_punctuator("*") {
                // multiplication
                node = Some(Box::new(Node::new(NodeKind::Mul, node, self.cast_expr())));
            } else if self.consume_punctuator("/") {
                // division
                node = Some(Box::new(Node::new(NodeKind::Div, node, self.cast_expr())));
            } else if self.consume_punctuator("%") {
                // remainder
                node = Some(Box::new(Node::new(NodeKind::Rem, node, self.cast_expr())));
            } else {
                return node;
            }
        }
    }

    // cast_expr ::= unary_expr
    //               | "(" type_name ")" cast_expr // 未実装
    fn cast_expr(&mut self) -> Option<Box<Node>> {
        self.unary_expr()
    }

    // unary_expr ::= postfix_expr
    //                | ("++" | "--") unary_expr
    //                | ( "&" | "*" | "+" | "-" | "~" | "!") cast_expr
    //                | sizeof unary_expr
    //                | sizeof "(" type_name ")" // 未実装
    fn unary_expr(&mut self) -> Option<Box<Node>> {
        if self.consume_punctuator("++") {
            // pre-increment
            return Some(Box::new(Node::new_unary(
                NodeKind::PreInc,
                self.unary_expr(),
            )));
        }
        if self.consume_punctuator("--") {
            // pre-decrement
            return Some(Box::new(Node::new_unary(
                NodeKind::PreDec,
                self.unary_expr(),
            )));
        }

        if self.consume_punctuator("+") {
            // unary plus
            return self.cast_expr();
        }
        if self.consume_punctuator("-") {
            // unary minus
            return Some(Box::new(Node::new(
                NodeKind::Sub,
                Some(Box::new(Node::new_num(0))),
                self.cast_expr(),
            )));
        }
        if self.consume_punctuator("&") {
            // address-of
            let mut node = Some(Box::new(Node::new_unary(NodeKind::Addr, self.cast_expr())));
            self.assign_types(&mut node);
            if node.is_some() && node.as_ref().unwrap().ty.is_none() {
                panic!("&演算子の型情報が設定されていません: {:?}", node);
            }
            return node;
        }
        if self.consume_punctuator("*") {
            // dereference
            let mut node = Some(Box::new(Node::new_unary(NodeKind::Deref, self.cast_expr())));
            self.assign_types(&mut node);
            if node.is_some() && node.as_ref().unwrap().ty.is_none() {
                panic!("*演算子の型情報が設定されていません: {:?}", node);
            }
            return node;
        }
        if self.consume_punctuator("~") {
            // bitwise not
            return Some(Box::new(Node::new_unary(
                NodeKind::BitNot,
                self.cast_expr(),
            )));
        }
        if self.consume_punctuator("!") {
            // logical not
            return Some(Box::new(Node::new_unary(
                NodeKind::LogicalNot,
                self.cast_expr(),
            )));
        }

        if self.consume_keyword("sizeof") {
            // sizeof unary_expr
            let mut node = self.unary_expr();
            self.assign_types(&mut node);
            if node.is_some() && node.as_ref().unwrap().ty.is_none() {
                panic!("sizeofの型情報が設定されていません: {:?}", node);
            }
            let size = node.as_ref().unwrap().ty.as_ref().unwrap().actual_size_of();
            return Some(Box::new(Node::new_num(size)));
        }

        self.postfix_expr()
    }

    // postfix_expr ::= primary_expr
    //                  | postfix_expr ("++" | "--")
    fn postfix_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.primary_expr();

        loop {
            if self.consume_punctuator("++") {
                // post-increment
                node = Some(Box::new(Node::new_unary(NodeKind::PostInc, node)));
            } else if self.consume_punctuator("--") {
                // post-decrement
                node = Some(Box::new(Node::new_unary(NodeKind::PostDec, node)));
            } else {
                return node;
            }
        }
    }

    // primary_expr ::= "(" expr ")"
    //                  | ident "(" (assign_expr ("," assign_expr)*)? ")"
    //                  | ident
    //                  | integer_constant
    fn primary_expr(&mut self) -> Option<Box<Node>> {
        if self.consume_punctuator("(") {
            let node = self.expr();
            self.expect_punctuator(")").unwrap();
            return node;
        }
        let token = self.consume_ident();
        if let Some(name) = token {
            // 関数呼び出し
            if self.consume_punctuator("(") {
                let mut node = Node::from(NodeKind::Call);
                node.func_name = name;

                // 引数リストをパース
                if self.consume_punctuator(")") {
                    // 引数なし
                } else {
                    // 引数あり
                    loop {
                        if let Some(arg) = self.assign_expr() {
                            node.args.insert(0, arg);
                        } else {
                            panic!("関数呼び出しの引数のパースに失敗しました");
                        }

                        if self.consume_punctuator(",") {
                            continue;
                        } else {
                            break;
                        }
                    }
                    self.expect_punctuator(")").unwrap();
                }

                return Some(Box::new(node));
            }

            // 変数参照
            let lvar = self.current_func.as_mut().unwrap().find_lvar(&name);
            if let Some(lvar) = lvar {
                // ローカル変数ノードを作成
                let node = Node::new_lvar(&lvar.name, lvar.offset, &lvar.ty);

                // TODO: 多次元配列への対応
                // 配列の場合はポインタ型に変換
                if self.consume_punctuator("[") {
                    let add = Node::new(NodeKind::Add, Some(Box::new(node)), self.expr());
                    let mut n = Some(Box::new(Node::new_unary(
                        NodeKind::Deref,
                        Some(Box::new(add)),
                    )));
                    self.assign_types(&mut n);
                    self.expect_punctuator("]").unwrap();
                    return n;
                }
                return Some(Box::new(node));
            } else if let Some(gvar) = self.find_gvar(&name) {
                // グローバル変数ノードを作成
                let node = Node::new_gvar(&gvar.name, &gvar.ty);

                // TODO: 多次元配列への対応
                // 配列の場合はポインタ型に変換
                if self.consume_punctuator("[") {
                    let add = Node::new(NodeKind::Add, Some(Box::new(node)), self.expr());
                    let mut n = Some(Box::new(Node::new_unary(
                        NodeKind::Deref,
                        Some(Box::new(add)),
                    )));
                    self.assign_types(&mut n);
                    self.expect_punctuator("]").unwrap();
                    return n;
                }
                return Some(Box::new(node));
            }
            panic!("未定義の関数もしくは変数です: {}", name);
        }
        Some(Box::new(Node::new_num(self.expect_number().unwrap())))
    }

    fn assign_types(&mut self, node: &mut Option<Box<Node>>) {
        if node.is_none() {
            return;
        }
        let node = node.as_mut().unwrap();
        self.assign_types(&mut node.lhs);
        self.assign_types(&mut node.rhs);

        match node.kind {
            NodeKind::Num => {
                // 数値リテラルの型はすでに設定されているはず
            }
            NodeKind::LVar => {
                // ローカル変数の型はすでに設定されているはず
            }
            NodeKind::GVar => {
                // グローバル変数の型はすでに設定されているはず
            }
            NodeKind::Add | NodeKind::Sub | NodeKind::Mul | NodeKind::Div | NodeKind::Rem => {
                // 二項演算子の型は左辺から決定
                node.ty = node.lhs.as_ref().unwrap().ty.clone();
            }
            NodeKind::Addr => {
                // アドレス演算子の型はポインタ型にする
                let base_ty = node.lhs.as_ref().unwrap().ty.as_ref().unwrap();
                let ptr_ty = Type::new_ptr(base_ty);
                node.ty = Some(Box::new(ptr_ty));
            }
            NodeKind::Deref => {
                // デリファレンス演算子の型はポインタの指す型にする
                let ptr_ty = node.lhs.as_ref().unwrap().ty.as_ref().unwrap();
                if let Some(pointee_ty) = &ptr_ty.ptr_to {
                    node.ty = Some(Box::new((**pointee_ty).clone()));
                } else {
                    panic!(
                        "ポインタ型ではないものをデリファレンスしようとしました: {:?}",
                        node
                    );
                }
            }
            _ => {
                // その他のノードはとりあえずNoneにする
                node.ty = None;
            }
        }
    }
}
