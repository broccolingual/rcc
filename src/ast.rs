use core::fmt;

use crate::node::{Node, NodeKind};
use crate::token::Token;
use crate::types::{Type, TypeKind};

#[derive(Clone)]
pub struct LVar {
    name: String,
    pub offset: i64,
    pub ty: Box<Type>,
}

impl LVar {
    pub fn new(name: &str, offset: i64, ty: Type) -> Self {
        LVar {
            name: name.to_string(),
            offset,
            ty: Box::new(ty),
        }
    }
}

impl fmt::Debug for LVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LVar {{ name: '{}', type: {:?}, offset: {} }}",
            self.name, self.ty, self.offset
        )
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

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Function {{ name: '{}', args: {:?} }}",
            self.name, self.args
        )
    }
}

pub struct Ast {
    tokens: Vec<Token>,
    pub funcs: Vec<Box<Function>>,
    pub locals: Vec<LVar>,
}

impl Ast {
    pub fn new(tokens: &[Token]) -> Self {
        Ast {
            tokens: tokens.to_vec(),
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

    fn consume_type(&mut self) -> Option<Token> {
        match self.tokens.first() {
            Some(Token::Type(ty)) => {
                let ty_clone = ty.clone();
                self.tokens.remove(0);
                Some(Token::Type(ty_clone))
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

    fn at_eof(&self) -> bool {
        self.tokens.is_empty() || matches!(self.tokens.first(), Some(Token::EOF))
    }

    // translation_unit ::= external_declaration*
    pub fn translation_unit(&mut self) {
        while !self.at_eof() {
            if let Some(func) = self.external_declaration() {
                self.funcs.push(func);
            }
        }
    }

    // external_declaration ::= func_def
    fn external_declaration(&mut self) -> Option<Box<Function>> {
        self.func_def()
    }

    // pointer ::= "*" pointer?
    fn pointer(&mut self, ty: Type) -> Type {
        while self.consume_symbol("*") {
            return self.pointer(Type::new_ptr(&ty));
        }
        ty
    }

    // direct_declarator ::= ident
    fn direct_declarator(&mut self, ty: Type) -> Option<Box<Node>> {
        if let Some(var_name) = self.consume_ident() {
            let offset = self.locals.first().map_or(8, |last| last.offset + 8);
            let mut node_var = Node::from(NodeKind::LVar);
            node_var.name = var_name.clone(); // 変数名を設定
            node_var.offset = offset; // 新しいローカル変数のオフセットを設定
            node_var.ty = Some(Box::new(ty.clone()));
            self.locals.insert(0, LVar::new(&var_name, offset, ty)); // ローカル変数リストの先頭に追加
            return Some(Box::new(node_var));
        } else {
            panic!("識別子のパースに失敗しました");
        }
    }

    // declarator ::= pointer? direct_declarator
    fn declarator(&mut self) -> Option<Box<Node>> {
        if let Some(tok) = self.consume_type() {
            // ポインタを処理
            if tok != Token::Type(TypeKind::Int) {
                panic!("現在サポートされている型はintのみです");
            }
            let ty = self.pointer(Type::new_int());

            // 変数名を取得
            if let Some(node_var) = self.direct_declarator(ty) {
                // let lvar = self.locals.first().unwrap().clone();
                return Some(node_var);
            } else {
                panic!("変数名のパースに失敗しました");
            }
        }
        None
    }

    // func_def ::= declarator "(" (declarator ("," declarator)*)? ")" compound_stmt
    fn func_def(&mut self) -> Option<Box<Function>> {
        // 関数名と戻り値の型のパース
        let func_name;
        if let Some(var_node) = self.declarator() {
            func_name = var_node.name;
        } else {
            panic!("関数名と戻り値の型のパースに失敗しました");
        }
        let mut func = Function::new(func_name);

        // 引数のパース（型情報もパース）
        self.expect_symbol("(").unwrap();
        loop {
            if let Some(var_node) = self.declarator() {
                let lvar = LVar::new(
                    &var_node.name,
                    var_node.offset,
                    (*var_node.ty.unwrap()).clone(),
                );
                func.args.push(lvar);
            } else {
                break;
            }
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
        if let Some(body_node) = self.compound_stmt() {
            func.body = body_node.body;
        } else {
            panic!("関数本体のパースに失敗しました");
        }
        Some(Box::new(func))
    }

    fn declaration(&mut self) -> Option<Box<Node>> {
        // variable declaration
        if let Some(var_node) = self.declarator() {
            self.expect_symbol(";").unwrap();
            return Some(var_node);
        }
        None
    }

    // TODO: case文, default文の実装
    fn labeled_stmt(&mut self) -> Option<Box<Node>> {
        if let Some(label_name) = self.consume_ident() {
            if self.consume_symbol(":") {
                let mut node = Node::new_unary(NodeKind::Label, self.stmt());
                node.label_name = label_name;
                return Some(Box::new(node));
            } else {
                // ラベル名ではなかった場合、トークンを元に戻す
                self.tokens.insert(0, Token::Ident(label_name));
            }
        }
        None
    }

    // compound_stmt ::= "{" declaration* stmt* "}"
    fn compound_stmt(&mut self) -> Option<Box<Node>> {
        if self.consume_symbol("{") {
            let mut node = Node::from(NodeKind::Block);
            while !self.consume_symbol("}") {
                if let Some(decl) = self.declaration() {
                    node.body.push(decl);
                } else if let Some(stmt) = self.stmt() {
                    node.body.push(stmt);
                } else {
                    panic!("ブロック内の文のパースに失敗しました");
                }
            }
            return Some(Box::new(node));
        }
        None
    }

    // TODO: switch文の実装
    // selection_stmt ::= "if" "(" expr ")" stmt ("else" stmt)?
    fn selection_stmt(&mut self) -> Option<Box<Node>> {
        if self.consume_reserved("if") {
            let mut node = Node::from(NodeKind::If);
            self.expect_symbol("(").unwrap();
            node.cond = self.expr();
            self.expect_symbol(")").unwrap();
            node.then = self.stmt();
            if self.consume_reserved("else") {
                node.els = self.stmt();
            }
            return Some(Box::new(node));
        }
        None
    }

    // iteration_stmt ::= "while" "(" expr ")" stmt
    //                    | "do" stmt "while" "(" expr ")" ";"
    //                    | "for" "(" expr? ";" expr? ";" expr? ")" stmt
    fn iteration_stmt(&mut self) -> Option<Box<Node>> {
        if self.consume_reserved("while") {
            let mut node = Node::from(NodeKind::While);
            self.expect_symbol("(").unwrap();
            node.cond = self.expr();
            self.expect_symbol(")").unwrap();
            node.then = self.stmt();
            return Some(Box::new(node));
        }

        if self.consume_reserved("do") {
            let mut node = Node::from(NodeKind::Do);
            node.then = self.stmt();
            self.expect_reserved("while").unwrap();
            self.expect_symbol("(").unwrap();
            node.cond = self.expr();
            self.expect_symbol(")").unwrap();
            self.expect_symbol(";").unwrap();
            return Some(Box::new(node));
        }

        if self.consume_reserved("for") {
            let mut node = Node::from(NodeKind::For);
            self.expect_symbol("(").unwrap();
            // 初期化式
            if !self.consume_symbol(";") {
                node.init = self.expr();
                self.expect_symbol(";").unwrap();
            }
            // 条件式
            if !self.consume_symbol(";") {
                node.cond = self.expr();
                self.expect_symbol(";").unwrap();
            }
            // 更新式
            if !self.consume_symbol(")") {
                node.inc = self.expr();
                self.expect_symbol(")").unwrap();
            }
            node.then = self.stmt();
            return Some(Box::new(node));
        }
        None
    }

    // jump_stmt ::= "goto" ident ";"
    //               | "continue" ";"
    //               | "break" ";"
    //               | "return" expr? ";"
    fn jump_stmt(&mut self) -> Option<Box<Node>> {
        if self.consume_reserved("goto") {
            let mut node = Node::from(NodeKind::Goto);
            node.label_name = self.consume_ident().unwrap();
            self.expect_symbol(";").unwrap();
            return Some(Box::new(node));
        }

        if self.consume_reserved("continue") {
            let node = Node::from(NodeKind::Continue);
            self.expect_symbol(";").unwrap();
            return Some(Box::new(node));
        }

        if self.consume_reserved("break") {
            let node = Node::from(NodeKind::Break);
            self.expect_symbol(";").unwrap();
            return Some(Box::new(node));
        }

        if self.consume_reserved("return") {
            if self.consume_symbol(";") {
                return Some(Box::new(Node::from(NodeKind::Return)));
            }

            let node = Node::new_unary(NodeKind::Return, self.expr());
            self.expect_symbol(";").unwrap();
            return Some(Box::new(node));
        }
        None
    }

    // stmt ::= labeled_stmt
    //          | expr_stmt
    //          | compound_stmt
    //          | selection_stmt
    //          | iteration_stmt
    //          | jump_stmt
    fn stmt(&mut self) -> Option<Box<Node>> {
        // labeled statement
        if let Some(node) = self.labeled_stmt() {
            return Some(node);
        }

        // selection statement
        if let Some(node) = self.selection_stmt() {
            return Some(node);
        }

        // iteration statement
        if let Some(node) = self.iteration_stmt() {
            return Some(node);
        }

        // compound statement
        if let Some(node) = self.compound_stmt() {
            return Some(node);
        }

        // jump statement
        if let Some(node) = self.jump_stmt() {
            return Some(node);
        }

        self.expr_stmt()
    }

    // expr_stmt ::= expr? ";"
    fn expr_stmt(&mut self) -> Option<Box<Node>> {
        if self.consume_symbol(";") {
            return Some(Box::new(Node::from(NodeKind::Nop)));
        } else {
            let expr_node = self.expr();
            self.expect_symbol(";").unwrap();
            return expr_node;
        };
    }

    // expr ::= assign_expr
    fn expr(&mut self) -> Option<Box<Node>> {
        self.assign_expr()
    }

    // assign_expr ::= conditional_expr (("=" | "*=" | "/=" | "%=" | "+=" | "-=" | "<<=" | ">>=" | "&=" | "^=" | "|=") assign_expr)?
    fn assign_expr(&mut self) -> Option<Box<Node>> {
        let mut node = self.conditional_expr();
        let assignment_ops = [
            "=", "*=", "/=", "%=", "+=", "-=", "<<=", ">>=", "&=", "^=", "|=",
        ];
        for op in &assignment_ops {
            if self.consume_symbol(op) {
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
                let mut rhs = self.mul_expr();
                if let Some(ty) = &node.as_ref().unwrap().ty {
                    if ty.is_ptr() {
                        // ポインタ加算の場合、スケーリングを考慮
                        let size = ty.ptr_to.as_ref().unwrap().size_of();
                        rhs = Some(Box::new(Node::new(
                            NodeKind::Mul,
                            rhs,
                            Some(Box::new(Node::new_num(size))),
                        )));
                    }
                }
                node = Some(Box::new(Node::new(NodeKind::Add, node, rhs)));
            } else if self.consume_symbol("-") {
                // subtraction
                let mut rhs = self.mul_expr();
                if let Some(ty) = &node.as_ref().unwrap().ty {
                    if ty.is_ptr() {
                        // ポインタ減算の場合、スケーリングを考慮
                        let size = ty.ptr_to.as_ref().unwrap().size_of();
                        rhs = Some(Box::new(Node::new(
                            NodeKind::Mul,
                            rhs,
                            Some(Box::new(Node::new_num(size))),
                        )));
                    }
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
    //                | ( "&" | "*" | "+" | "-" | "~" | "!") cast_expr
    //                | sizeof unary_expr
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

        let unary_ops = ["&", "*", "~", "!"];
        for op in &unary_ops {
            if self.consume_symbol(op) {
                let kind = NodeKind::from_str(op).unwrap();
                return Some(Box::new(Node::new_unary(kind, self.cast_expr())));
            }
        }

        if self.consume_reserved("sizeof") {
            let node = self.unary_expr();
            if let Some(ty) = &node.as_ref().unwrap().ty {
                let size = ty.size_of();
                return Some(Box::new(Node::new_num(size)));
            } else {
                panic!("sizeofのパースに失敗しました");
            }
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
                node.name = name; // 変数名を設定
                node.offset = lvar.offset; // 既存のローカル変数のオフセットを設定
                node.ty = Some(Box::new(*lvar.ty.clone())); // 変数の型情報を設定
            } else {
                panic!("未定義の変数です: {}", name);
            }
            return Some(Box::new(node));
        }
        Some(Box::new(Node::new_num(self.expect_number().unwrap())))
    }
}
