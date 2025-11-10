use crate::node::{Node, NodeKind};
use crate::token::Token;

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
    pub body: Vec<Box<Node>>,
    pub args: Vec<LVar>,
}

impl Function {
    pub fn new(name: String) -> Self {
        Function {
            name,
            body: Vec::new(),
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

    fn consume(&mut self, token: &Token) -> bool {
        match self.tokens.first() {
            Some(t) if t == token => {
                self.tokens.remove(0);
                true
            }
            _ => false,
        }
    }

    fn consume_symbol(&mut self, sym: &str) -> bool {
        self.consume(&Token::Symbol(sym.to_string()))
    }

    fn consume_reserved(&mut self, word: &str) -> bool {
        self.consume(&Token::Reserved(word.to_string()))
    }

    fn consume_ident(&mut self) -> Option<String> {
        match self.tokens.first() {
            Some(Token::Ident(name)) => {
                let name_clone = name.clone();
                self.tokens.remove(0);
                Some(name_clone)
            }
            _ => None,
        }
    }

    fn expect(&mut self, token: &Token) -> Result<(), &str> {
        match self.tokens.first() {
            Some(t) if t == token => {
                self.tokens.remove(0);
                Ok(())
            }
            _ => Err("期待されたトークンではありません"),
        }
    }

    fn expect_symbol(&mut self, sym: &str) -> Result<(), &str> {
        self.expect(&Token::Symbol(sym.to_string()))
    }

    fn expect_reserved(&mut self, word: &str) -> Result<(), &str> {
        self.expect(&Token::Reserved(word.to_string()))
    }

    fn expect_number(&mut self) -> Result<i64, &str> {
        match self.tokens.first() {
            Some(Token::Num(val)) => {
                let val_clone = *val;
                self.tokens.remove(0);
                Ok(val_clone)
            }
            _ => Err("数値トークンではありません"),
        }
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
        self.tokens.is_empty() || matches!(self.tokens.first(), Some(Token::EOF))
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
        self.expect_symbol("(").unwrap();

        // 引数のパース
        while let Some(arg_name) = self.consume_ident() {
            // 現状、引数の型情報は無視
            let lvar = self.new_lvar(&arg_name);
            func.args.push(lvar);

            if !self.consume_symbol(",") {
                break;
            }
        }

        self.expect_symbol(")").unwrap();

        if self.consume_symbol(";") {
            // 関数プロトタイプ宣言
            return None;
        }

        // 関数本体のパース
        self.expect_symbol("{").unwrap();
        while !self.consume_symbol("}") {
            if let Some(stmt) = self.stmt() {
                func.body.push(stmt);
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

        if self.consume_reserved("return") {
            if self.consume_symbol(";") {
                node = Some(Box::new(Node::from(NodeKind::Return)));
                return node;
            }

            node = Some(Box::new(Node::new_unary(NodeKind::Return, self.expr())));
            self.expect_symbol(";").unwrap();
            return node;
        }

        // selection statement
        if self.consume_reserved("if") {
            node = Some(Box::new(Node::from(NodeKind::If)));
            self.expect_symbol("(").unwrap();
            node.as_mut().unwrap().cond = self.expr();
            self.expect_symbol(")").unwrap();
            node.as_mut().unwrap().then = self.stmt();
            if self.consume_reserved("else") {
                node.as_mut().unwrap().els = self.stmt();
            }
            return node;
        }

        // iteration statement
        if self.consume_reserved("while") {
            node = Some(Box::new(Node::from(NodeKind::While)));
            self.expect_symbol("(").unwrap();
            node.as_mut().unwrap().cond = self.expr();
            self.expect_symbol(")").unwrap();
            node.as_mut().unwrap().then = self.stmt();
            return node;
        }

        if self.consume_reserved("for") {
            node = Some(Box::new(Node::from(NodeKind::For)));
            self.expect_symbol("(").unwrap();
            // 初期化式
            if !self.consume_symbol(";") {
                node.as_mut().unwrap().init = self.expr();
                self.expect_symbol(";").unwrap();
            }
            // 条件式
            if !self.consume_symbol(";") {
                node.as_mut().unwrap().cond = self.expr();
                self.expect_symbol(";").unwrap();
            }
            // 更新式
            if !self.consume_symbol(")") {
                node.as_mut().unwrap().inc = self.expr();
                self.expect_symbol(")").unwrap();
            }
            node.as_mut().unwrap().then = self.stmt();
            return node;
        }

        if self.consume_reserved("do") {
            node = Some(Box::new(Node::from(NodeKind::Do)));
            node.as_mut().unwrap().then = self.stmt();
            self.expect_reserved("while").unwrap();
            self.expect_symbol("(").unwrap();
            node.as_mut().unwrap().cond = self.expr();
            self.expect_symbol(")").unwrap();
            self.expect_symbol(";").unwrap();
            return node;
        }

        // compound statement
        if self.consume_symbol("{") {
            let mut node = Some(Box::new(Node::from(NodeKind::Block)));
            while !self.consume_symbol("}") {
                if let Some(stmt) = self.stmt() {
                    node.as_mut().unwrap().body.push(stmt);
                } else {
                    panic!("ブロック内の文のパースに失敗しました");
                }
            }
            return node;
        }

        // break statement
        if self.consume_reserved("break") {
            self.expect_symbol(";").unwrap();
            return Some(Box::new(Node::from(NodeKind::Break)));
        }

        // continue statement
        if self.consume_reserved("continue") {
            self.expect_symbol(";").unwrap();
            return Some(Box::new(Node::from(NodeKind::Continue)));
        }

        self.expr_stmt()
    }

    // expr_stmt ::= expr ";"
    fn expr_stmt(&mut self) -> Option<Box<Node>> {
        let node = self.expr();
        self.expect_symbol(";").unwrap();
        node
    }

    // expr ::= assign_expr
    fn expr(&mut self) -> Option<Box<Node>> {
        self.assign_expr()
    }

    // assign_expr ::= conditional_expr (("=" | "+=" | "-=" | "*=" | "/=" | "<<=" | ">>=" | "&=" | "|=" | "^=") assign_expr)?
    fn assign_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.conditional_expr();

        if self.consume_symbol("=") {
            node = Some(Box::new(Node::new(
                NodeKind::Assign,
                node,
                self.assign_expr(),
            )));
        }

        if self.consume_symbol("+=") {
            node = Some(Box::new(Node::new(
                NodeKind::AddAssign,
                node,
                self.assign_expr(),
            )));
        }

        if self.consume_symbol("-=") {
            node = Some(Box::new(Node::new(
                NodeKind::SubAssign,
                node,
                self.assign_expr(),
            )));
        }

        if self.consume_symbol("*=") {
            node = Some(Box::new(Node::new(
                NodeKind::MulAssign,
                node,
                self.assign_expr(),
            )));
        }

        if self.consume_symbol("/=") {
            node = Some(Box::new(Node::new(
                NodeKind::DivAssign,
                node,
                self.assign_expr(),
            )));
        }

        if self.consume_symbol("<<=") {
            node = Some(Box::new(Node::new(
                NodeKind::ShlAssign,
                node,
                self.assign_expr(),
            )));
        }

        if self.consume_symbol(">>=") {
            node = Some(Box::new(Node::new(
                NodeKind::ShrAssign,
                node,
                self.assign_expr(),
            )));
        }

        if self.consume_symbol("&=") {
            node = Some(Box::new(Node::new(
                NodeKind::BitAndAssign,
                node,
                self.assign_expr(),
            )));
        }

        if self.consume_symbol("|=") {
            node = Some(Box::new(Node::new(
                NodeKind::BitOrAssign,
                node,
                self.assign_expr(),
            )));
        }

        if self.consume_symbol("^=") {
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
        if self.consume_symbol("?") {
            let mut ternary_node = Node::from(NodeKind::Ternary);
            ternary_node.cond = node;
            ternary_node.then = self.expr();
            self.expect_symbol(":").unwrap();
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
            if self.consume_symbol("||") {
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
            if self.consume_symbol("&&") {
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
            if self.consume_symbol("|") {
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
            if self.consume_symbol("^") {
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
            if self.consume_symbol("&") {
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
            if self.consume_symbol("==") {
                // equal
                node = Some(Box::new(Node::new(
                    NodeKind::Eq,
                    node,
                    self.relational_expr(),
                )));
            } else if self.consume_symbol("!=") {
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
            if self.consume_symbol("<") {
                // less than
                node = Some(Box::new(Node::new(NodeKind::Lt, node, self.shift_expr())));
            } else if self.consume_symbol("<=") {
                // less than or equal
                node = Some(Box::new(Node::new(NodeKind::Le, node, self.shift_expr())));
            } else if self.consume_symbol(">") {
                // greater than
                node = Some(Box::new(Node::new(NodeKind::Lt, self.shift_expr(), node)));
            } else if self.consume_symbol(">=") {
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
            if self.consume_symbol("<<") {
                // left shift
                node = Some(Box::new(Node::new(NodeKind::Shl, node, self.add_expr())));
            } else if self.consume_symbol(">>") {
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
            if self.consume_symbol("+") {
                // addition
                node = Some(Box::new(Node::new(NodeKind::Add, node, self.mul_expr())));
            } else if self.consume_symbol("-") {
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
            if self.consume_symbol("*") {
                // multiplication
                node = Some(Box::new(Node::new(NodeKind::Mul, node, self.cast_expr())));
            } else if self.consume_symbol("/") {
                // division
                node = Some(Box::new(Node::new(NodeKind::Div, node, self.cast_expr())));
            } else if self.consume_symbol("%") {
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
    //                | ("+" | "-" | "!" | "~" | "&" | "*") cast_expr
    fn unary_expr(&mut self) -> Option<Box<Node>> {
        if self.consume_symbol("++") {
            // pre-increment
            return Some(Box::new(Node::new_unary(
                NodeKind::PreInc,
                self.unary_expr(),
            )));
        }
        if self.consume_symbol("--") {
            // pre-decrement
            return Some(Box::new(Node::new_unary(
                NodeKind::PreDec,
                self.unary_expr(),
            )));
        }
        if self.consume_symbol("+") {
            // unary plus
            return self.cast_expr();
        }
        if self.consume_symbol("-") {
            // unary minus
            return Some(Box::new(Node::new(
                NodeKind::Sub,
                Some(Box::new(Node::new_num(0))),
                self.cast_expr(),
            )));
        }
        if self.consume_symbol("!") {
            // logical not
            return Some(Box::new(Node::new_unary(
                NodeKind::LogicalNot,
                self.cast_expr(),
            )));
        }
        if self.consume_symbol("~") {
            // bitwise not
            return Some(Box::new(Node::new_unary(
                NodeKind::BitNot,
                self.cast_expr(),
            )));
        }
        if self.consume_symbol("&") {
            // address-of
            return Some(Box::new(Node::new_unary(NodeKind::Addr, self.cast_expr())));
        }
        if self.consume_symbol("*") {
            // dereference
            return Some(Box::new(Node::new_unary(NodeKind::Deref, self.cast_expr())));
        }
        self.postfix_expr()
    }

    // postfix_expr ::= primary_expr
    //                  | postfix_expr ("++" | "--")
    fn postfix_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.primary_expr();

        loop {
            if self.consume_symbol("++") {
                // post-increment
                node = Some(Box::new(Node::new_unary(NodeKind::PostInc, node)));
            } else if self.consume_symbol("--") {
                // post-decrement
                node = Some(Box::new(Node::new_unary(NodeKind::PostDec, node)));
            } else {
                return node;
            }
        }
    }

    // primary_expr ::= "(" expr ")"
    //                  | ident
    //                  | num
    fn primary_expr(&mut self) -> Option<Box<Node>> {
        if self.consume_symbol("(") {
            let node = self.expr();
            self.expect_symbol(")").unwrap();
            return node;
        }
        let token = self.consume_ident();
        if let Some(name) = token {
            // 関数呼び出し
            if self.consume_symbol("(") {
                let mut node = Node::from(NodeKind::Call);
                node.func_name = name;

                // 引数リストをパース
                if self.consume_symbol(")") {
                    // 引数なし
                } else {
                    // 引数あり
                    loop {
                        if let Some(arg) = self.assign_expr() {
                            node.args.push(arg);
                        } else {
                            panic!("関数呼び出しの引数のパースに失敗しました");
                        }

                        if self.consume_symbol(",") {
                            continue;
                        } else {
                            break;
                        }
                    }
                    self.expect_symbol(")").unwrap();
                }

                return Some(Box::new(node));
            }

            // ローカル変数ノードを作成
            let mut node = Node::from(NodeKind::LVar);
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
