use crate::parser::{Token, TokenKind};

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
    Ternary,      // ?:
    Assign,       // =
    AddAssign,    // +=
    SubAssign,    // -=
    MulAssign,    // *=
    DivAssign,    // /=
    ShlAssign,    // <<=
    ShrAssign,    // >>=
    BitAndAssign, // &=
    BitOrAssign,  // |=
    BitXorAssign, // ^=
    PreInc,       // ++pre
    PreDec,       // --pre
    PostInc,      // post++
    PostDec,      // post--
    If,           // if
    While,        // while
    For,          // for
    Do,           // do
    Block,        // {}
    Call,         // 関数呼び出し
    Label,        // ラベル
    Break,        // break
    Continue,     // continue
    LVar,         // ローカル変数
    Return,       // return
    Num,          // 整数
}

#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,
    // pub next: Option<Box<Node>>, // 次のノードへのポインタ
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub val: i64,                // kindがNumのときに使う
    pub offset: i64,             // kindがLVarのときに使う
    pub cond: Option<Box<Node>>, // if, while文の条件式
    pub then: Option<Box<Node>>, // if, while文のthen節
    pub els: Option<Box<Node>>,  // if文のelse節
    pub init: Option<Box<Node>>, // for文の初期化式
    pub inc: Option<Box<Node>>,  // for文の更新式
    pub body: Vec<Box<Node>>,    // ブロック内のstatementリスト
    pub label_name: String,      // ラベル名
    pub func_name: String,       // 関数名
    pub args: Vec<Box<Node>>,    // 関数呼び出しの引数リスト
}

impl Node {
    pub fn new(kind: NodeKind, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Self {
        Node {
            kind,
            lhs,
            rhs,
            val: 0,
            offset: 0,
            cond: None,
            then: None,
            els: None,
            init: None,
            inc: None,
            body: Vec::new(),
            label_name: String::new(),
            func_name: String::new(),
            args: Vec::new(),
        }
    }

    pub fn new_num(val: i64) -> Self {
        Node {
            kind: NodeKind::Num,
            lhs: None,
            rhs: None,
            val,
            offset: 0,
            cond: None,
            then: None,
            els: None,
            init: None,
            inc: None,
            body: Vec::new(),
            label_name: String::new(),
            func_name: String::new(),
            args: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LVar {
    name: String,
    pub offset: i64,
}

impl LVar {
    pub fn new(name: &str, offset: i64) -> Self {
        LVar {
            name: name.to_string(),
            offset,
        }
    }
}

pub struct Function {
    pub name: String,
    pub nodes: Vec<Box<Node>>,
    pub args: Vec<LVar>,
}

impl Function {
    pub fn new(name: String) -> Self {
        Function {
            name,
            nodes: Vec::new(),
            args: Vec::new(),
        }
    }
}

pub struct Ast {
    pub tokens: Vec<Token>,
    pub funcs: Vec<Box<Function>>,
    pub locals: Vec<LVar>,
}

impl Ast {
    pub fn new(tokens: Vec<Token>) -> Self {
        Ast {
            tokens,
            funcs: Vec::new(),
            locals: Vec::new(),
        }
    }

    fn find_lvar(&mut self, name: &str) -> Option<&mut LVar> {
        self.locals.iter_mut().find(|lvar| lvar.name == name)
    }

    fn consume(&mut self, op: &str) -> bool {
        let current_token = self.tokens.first();
        if current_token.is_none()
            || current_token.unwrap().kind != TokenKind::Reserved
            || current_token.unwrap().input != op
        {
            return false;
        }
        self.tokens.remove(0);
        true
    }

    fn consume_ident(&mut self) -> Option<String> {
        let current_token = self.tokens.first();
        if current_token.is_none() || current_token.unwrap().kind != TokenKind::Ident {
            return None;
        }
        let name = current_token.unwrap().input.clone();
        self.tokens.remove(0);
        Some(name)
    }

    fn expect(&mut self, op: &str) -> Result<(), &str> {
        let current_token = self.tokens.first();
        if current_token.is_none()
            || current_token.unwrap().kind != TokenKind::Reserved
            || current_token.unwrap().input != op
        {
            return Err("予期せぬトークンです");
        }
        self.tokens.remove(0);
        Ok(())
    }

    fn expect_number(&mut self) -> Result<i64, &str> {
        let current_token = self.tokens.first();
        if current_token.is_none() || current_token.unwrap().kind != TokenKind::Num {
            return Err("数値ではありません");
        }
        let val = current_token.unwrap().val;
        self.tokens.remove(0);
        Ok(val)
    }

    fn new_lvar(&mut self, name: &str) -> LVar {
        // 新しいローカル変数のオフセットを計算
        let offset = self.locals.first().map_or(8, |last| last.offset + 8);
        let lvar = LVar::new(name, offset);
        // ローカル変数リストの先頭に追加
        self.locals.insert(0, lvar.clone());
        lvar
    }

    fn at_eof(&self) -> bool {
        self.tokens.is_empty() || self.tokens.first().unwrap().kind == TokenKind::EOF
    }

    // program ::= function*
    pub fn program(&mut self) {
        while !self.at_eof() {
            if let Some(func) = self.function() {
                self.funcs.push(func);
            }
        }
    }

    fn function(&mut self) -> Option<Box<Function>> {
        // 関数宣言(形指定なし)
        let func_name = self.consume_ident().unwrap();
        let mut func = Function::new(func_name);
        self.expect("(").unwrap();

        // 引数のパース
        while let Some(arg_name) = self.consume_ident() {
            // 現状、引数の型情報は無視
            let lvar = self.new_lvar(&arg_name);
            func.args.push(lvar);

            if !self.consume(",") {
                break;
            }
        }

        self.expect(")").unwrap();

        if self.consume(";") {
            // 関数プロトタイプ宣言
            return None;
        }

        // 関数本体のパース
        self.expect("{").unwrap();
        while !self.consume("}") {
            if let Some(stmt) = self.stmt() {
                func.nodes.push(stmt);
            } else {
                panic!("関数の文のパースに失敗しました");
            }
        }
        Some(Box::new(func))
    }

    // stmt ::= "return" expr? ";"
    //          | "if" "(" expr ")" stmt ("else" stmt)?
    //          | "while" "(" expr ")" stmt
    //          | "for" "(" expr? ";" expr? ";" expr? ")" stmt
    //          | "do" stmt "while" "(" expr ")" ";"
    //          | "{" stmt* "}"
    //          | "break" ";"
    //          | "continue" ";"
    //          | expr_stmt
    fn stmt(&mut self) -> Option<Box<Node>> {
        let mut node: Option<Box<Node>>;

        if self.consume("return") {
            if self.consume(";") {
                node = Some(Box::new(Node::new(NodeKind::Return, None, None)));
                return node;
            }

            node = Some(Box::new(Node::new(NodeKind::Return, self.expr(), None)));
            self.expect(";").unwrap();
            return node;
        }

        // selection statement
        if self.consume("if") {
            node = Some(Box::new(Node::new(NodeKind::If, None, None)));
            self.expect("(").unwrap();
            node.as_mut().unwrap().cond = self.expr();
            self.expect(")").unwrap();
            node.as_mut().unwrap().then = self.stmt();
            if self.consume("else") {
                node.as_mut().unwrap().els = self.stmt();
            }
            return node;
        }

        // iteration statement
        if self.consume("while") {
            node = Some(Box::new(Node::new(NodeKind::While, None, None)));
            self.expect("(").unwrap();
            node.as_mut().unwrap().cond = self.expr();
            self.expect(")").unwrap();
            node.as_mut().unwrap().then = self.stmt();
            return node;
        }

        if self.consume("for") {
            node = Some(Box::new(Node::new(NodeKind::For, None, None)));
            self.expect("(").unwrap();
            // 初期化式
            if !self.consume(";") {
                node.as_mut().unwrap().init = self.expr();
                self.expect(";").unwrap();
            }
            // 条件式
            if !self.consume(";") {
                node.as_mut().unwrap().cond = self.expr();
                self.expect(";").unwrap();
            }
            // 更新式
            if !self.consume(")") {
                node.as_mut().unwrap().inc = self.expr();
                self.expect(")").unwrap();
            }
            node.as_mut().unwrap().then = self.stmt();
            return node;
        }

        if self.consume("do") {
            node = Some(Box::new(Node::new(NodeKind::Do, None, None)));
            node.as_mut().unwrap().then = self.stmt();
            self.expect("while").unwrap();
            self.expect("(").unwrap();
            node.as_mut().unwrap().cond = self.expr();
            self.expect(")").unwrap();
            self.expect(";").unwrap();
            return node;
        }

        // compound statement
        if self.consume("{") {
            let mut node = Some(Box::new(Node::new(NodeKind::Block, None, None)));
            while !self.consume("}") {
                if let Some(stmt) = self.stmt() {
                    node.as_mut().unwrap().body.push(stmt);
                } else {
                    panic!("ブロック内の文のパースに失敗しました");
                }
            }
            return node;
        }

        if self.consume("break") {
            self.expect(";").unwrap();
            return Some(Box::new(Node::new(NodeKind::Break, None, None)));
        }

        if self.consume("continue") {
            self.expect(";").unwrap();
            return Some(Box::new(Node::new(NodeKind::Continue, None, None)));
        }

        self.expr_stmt()
    }

    // expr_stmt ::= expr ";"
    fn expr_stmt(&mut self) -> Option<Box<Node>> {
        let node = self.expr();
        self.expect(";").unwrap();
        node
    }

    // expr ::= assign_expr
    fn expr(&mut self) -> Option<Box<Node>> {
        self.assign_expr()
    }

    // assign_expr ::= conditional_expr (("=" | "+=" | "-=" | "*=" | "/=" | "<<=" | ">>=" | "&=" | "|=" | "^=") assign_expr)?
    fn assign_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.conditional_expr();

        if self.consume("=") {
            node = Some(Box::new(Node::new(
                NodeKind::Assign,
                node,
                self.assign_expr(),
            )));
        }

        if self.consume("+=") {
            node = Some(Box::new(Node::new(
                NodeKind::AddAssign,
                node,
                self.assign_expr(),
            )));
        }

        if self.consume("-=") {
            node = Some(Box::new(Node::new(
                NodeKind::SubAssign,
                node,
                self.assign_expr(),
            )));
        }

        if self.consume("*=") {
            node = Some(Box::new(Node::new(
                NodeKind::MulAssign,
                node,
                self.assign_expr(),
            )));
        }

        if self.consume("/=") {
            node = Some(Box::new(Node::new(
                NodeKind::DivAssign,
                node,
                self.assign_expr(),
            )));
        }

        if self.consume("<<=") {
            node = Some(Box::new(Node::new(
                NodeKind::ShlAssign,
                node,
                self.assign_expr(),
            )));
        }

        if self.consume(">>=") {
            node = Some(Box::new(Node::new(
                NodeKind::ShrAssign,
                node,
                self.assign_expr(),
            )));
        }

        if self.consume("&=") {
            node = Some(Box::new(Node::new(
                NodeKind::BitAndAssign,
                node,
                self.assign_expr(),
            )));
        }

        if self.consume("|=") {
            node = Some(Box::new(Node::new(
                NodeKind::BitOrAssign,
                node,
                self.assign_expr(),
            )));
        }

        if self.consume("^=") {
            node = Some(Box::new(Node::new(
                NodeKind::BitXorAssign,
                node,
                self.assign_expr(),
            )));
        }
        node
    }

    // conditional_expr ::= logical_or_expr
    //                      | logical_or_expr "?" expr ":" conditional_expr
    fn conditional_expr(&mut self) -> Option<Box<Node>> {
        let node = self.logical_or_expr();
        if self.consume("?") {
            let mut ternary_node = Node::new(NodeKind::Ternary, None, None);
            ternary_node.cond = node;
            ternary_node.then = self.expr();
            self.expect(":").unwrap();
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
            if self.consume("||") {
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
            if self.consume("&&") {
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
            if self.consume("|") {
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
            if self.consume("^") {
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
            if self.consume("&") {
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
            if self.consume("==") {
                // equal
                node = Some(Box::new(Node::new(
                    NodeKind::Eq,
                    node,
                    self.relational_expr(),
                )));
            } else if self.consume("!=") {
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
            if self.consume("<") {
                // less than
                node = Some(Box::new(Node::new(NodeKind::Lt, node, self.shift_expr())));
            } else if self.consume("<=") {
                // less than or equal
                node = Some(Box::new(Node::new(NodeKind::Le, node, self.shift_expr())));
            } else if self.consume(">") {
                // greater than
                node = Some(Box::new(Node::new(NodeKind::Lt, self.shift_expr(), node)));
            } else if self.consume(">=") {
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
            if self.consume("<<") {
                // left shift
                node = Some(Box::new(Node::new(NodeKind::Shl, node, self.add_expr())));
            } else if self.consume(">>") {
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
            if self.consume("+") {
                // addition
                node = Some(Box::new(Node::new(NodeKind::Add, node, self.mul_expr())));
            } else if self.consume("-") {
                // subtraction
                node = Some(Box::new(Node::new(NodeKind::Sub, node, self.mul_expr())));
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
            if self.consume("*") {
                // multiplication
                node = Some(Box::new(Node::new(NodeKind::Mul, node, self.cast_expr())));
            } else if self.consume("/") {
                // division
                node = Some(Box::new(Node::new(NodeKind::Div, node, self.cast_expr())));
            } else if self.consume("%") {
                // remainder
                node = Some(Box::new(Node::new(NodeKind::Rem, node, self.cast_expr())));
            } else {
                return node;
            }
        }
    }

    // cast_expr ::= unary_expr
    fn cast_expr(&mut self) -> Option<Box<Node>> {
        self.unary_expr()
    }

    // unary_expr ::= postfix_expr
    //                | ("++" | "--") unary_expr
    //                | ("+" | "-" | "!" | "~") cast_expr
    fn unary_expr(&mut self) -> Option<Box<Node>> {
        if self.consume("++") {
            // pre-increment
            return Some(Box::new(Node::new(
                NodeKind::PreInc,
                self.unary_expr(),
                None,
            )));
        }
        if self.consume("--") {
            // pre-decrement
            return Some(Box::new(Node::new(
                NodeKind::PreDec,
                self.unary_expr(),
                None,
            )));
        }
        if self.consume("+") {
            // unary plus
            return self.cast_expr();
        }
        if self.consume("-") {
            // unary minus
            return Some(Box::new(Node::new(
                NodeKind::Sub,
                Some(Box::new(Node::new_num(0))),
                self.cast_expr(),
            )));
        }
        if self.consume("!") {
            // logical not
            return Some(Box::new(Node::new(
                NodeKind::LogicalNot,
                self.cast_expr(),
                None,
            )));
        }
        if self.consume("~") {
            // bitwise not
            return Some(Box::new(Node::new(
                NodeKind::BitNot,
                self.cast_expr(),
                None,
            )));
        }
        self.postfix_expr()
    }

    // postfix_expr ::= primary_expr
    //                  | postfix_expr ("++" | "--")
    fn postfix_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.primary_expr();

        loop {
            if self.consume("++") {
                // post-increment
                node = Some(Box::new(Node::new(NodeKind::PostInc, node, None)));
            } else if self.consume("--") {
                // post-decrement
                node = Some(Box::new(Node::new(NodeKind::PostDec, node, None)));
            } else {
                return node;
            }
        }
    }

    // primary_expr ::= "(" expr ")"
    //                  | ident
    //                  | num
    fn primary_expr(&mut self) -> Option<Box<Node>> {
        if self.consume("(") {
            let node = self.expr();
            self.expect(")").unwrap();
            return node;
        }
        let token = self.consume_ident();
        if let Some(name) = token {
            // 関数呼び出し
            if self.consume("(") {
                let mut node = Node::new(NodeKind::Call, None, None);
                node.func_name = name;

                // 引数リストをパース
                if self.consume(")") {
                    // 引数なし
                } else {
                    // 引数あり
                    loop {
                        if let Some(arg) = self.assign_expr() {
                            node.args.push(arg);
                        } else {
                            panic!("関数呼び出しの引数のパースに失敗しました");
                        }

                        if self.consume(",") {
                            continue;
                        } else {
                            break;
                        }
                    }
                    self.expect(")").unwrap();
                }

                return Some(Box::new(node));
            }

            // ローカル変数ノードを作成
            let mut node = Node::new(NodeKind::LVar, None, None);
            let lvar = self.find_lvar(&name);
            if let Some(lvar) = lvar {
                node.offset = lvar.offset; // 既存のローカル変数のオフセットを設定
            } else {
                let new_lvar = self.new_lvar(&name);
                node.offset = new_lvar.offset; // 新しいローカル変数のオフセットを設定
            }
            return Some(Box::new(node));
        }
        Some(Box::new(Node::new_num(self.expect_number().unwrap())))
    }
}
