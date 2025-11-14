use core::{fmt, panic};

use crate::node::{Node, NodeKind};
use crate::token::Token;
use crate::types::{Type, TypeKind};

#[derive(Clone)]
pub struct Var {
    pub name: String,
    pub offset: i64,
    pub ty: Box<Type>,
}

impl Var {
    pub fn new(name: &str, ty: Type) -> Self {
        Var {
            name: name.to_string(),
            offset: 0,
            ty: Box::new(ty),
        }
    }
}

impl fmt::Debug for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Var {{ name: '{}', type: {:?}, offset: {} }}",
            self.name, self.ty, self.offset
        )
    }
}

pub struct Function {
    pub name: String,
    pub body: Vec<Box<Node>>,
    pub locals: Vec<Var>,
}

impl Function {
    pub fn new(name: &str) -> Self {
        Function {
            name: name.to_string(),
            body: Vec::new(),
            locals: Vec::new(),
        }
    }
}

impl Function {
    fn gen_lvar(&mut self, mut var: Var) -> Result<(), &str> {
        if self.find_lvar(&var.name).is_some() {
            return Err("同じ名前のローカル変数が既に存在します");
        }
        var.offset = if let Some(first_var) = self.locals.first() {
            first_var.offset + var.ty.size_of() as i64
        } else {
            var.ty.size_of() as i64
        };
        self.locals.insert(0, var);
        Ok(())
    }

    fn find_lvar(&mut self, name: &str) -> Option<&mut Var> {
        for var in &mut self.locals {
            if var.name == name {
                return Some(var);
            }
        }
        None
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Function {{ name: '{}', locals: {:?} }}",
            self.name, self.locals
        )
    }
}

pub struct Ast {
    tokens: Vec<Token>,
    pub globals: Vec<Var>,
    pub funcs: Vec<Box<Function>>,
    current_func: Option<Box<Function>>,
}

impl Ast {
    pub fn new(tokens: &[Token]) -> Self {
        Ast {
            tokens: tokens.to_vec(),
            globals: Vec::new(),
            funcs: Vec::new(),
            current_func: None,
        }
    }

    fn gen_gvar(&mut self, var: Var) -> Result<(), &str> {
        if self.find_gvar(&var.name).is_some() {
            return Err("同じ名前のグローバル変数が既に存在します");
        }
        self.globals.push(var);
        Ok(())
    }

    fn find_gvar(&mut self, name: &str) -> Option<&mut Var> {
        for var in &mut self.globals {
            if var.name == name {
                return Some(var);
            }
        }
        None
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

    fn consume_punctuator(&mut self, sym: &str) -> bool {
        self.consume(&Token::Punctuator(sym.to_string()))
    }

    fn consume_keyword(&mut self, word: &str) -> bool {
        self.consume(&Token::Keyword(word.to_string()))
    }

    fn consume_ident(&mut self) -> Option<String> {
        match self.tokens.first() {
            Some(Token::Identifier(name)) => {
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

    fn expect_punctuator(&mut self, sym: &str) -> Result<(), &str> {
        self.expect(&Token::Punctuator(sym.to_string()))
    }

    fn expect_reserved(&mut self, word: &str) -> Result<(), &str> {
        self.expect(&Token::Keyword(word.to_string()))
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
    //                          | declaration
    fn external_declaration(&mut self) -> Option<Box<Function>> {
        if let Some(result) = self.declaration() {
            match result {
                Ok(var) => {
                    // グローバル変数宣言
                    self.gen_gvar(var).unwrap();
                    return None;
                }
                Err(var) => {
                    // 関数定義
                    if let Ok(func) = self.func_def(var) {
                        return Some(func);
                    }
                    panic!("関数定義のパースに失敗しました");
                }
            }
        }
        panic!("external_declarationのパースに失敗しました");
    }

    // pointer ::= "*" pointer?
    fn pointer(&mut self, ty: Type) -> Type {
        while self.consume_punctuator("*") {
            return self.pointer(Type::new_ptr(&ty));
        }
        ty
    }

    // direct_declarator ::= ident
    //                       | ident "[" number "]"
    fn direct_declarator(&mut self, ty: Type) -> Result<Var, &str> {
        if let Some(name) = self.consume_ident() {
            let new_ty;
            if self.consume_punctuator("[") {
                // 配列型の処理
                let array_size = self.expect_number().unwrap() as usize;
                self.expect_punctuator("]").unwrap();
                let array_ty = Type::new_array(&ty, array_size);
                new_ty = array_ty;
            } else {
                // 通常の変数型の場合
                new_ty = ty;
            }
            return Ok(Var::new(&name, new_ty));
        }
        Err("direct_declaratorのパースに失敗しました")
    }

    // declarator ::= pointer? direct_declarator
    fn declarator(&mut self) -> Result<Var, &str> {
        if self.consume_keyword("int") {
            // ポインタを処理
            let ty = self.pointer(Type::new_int());

            // 変数名を取得
            return self.direct_declarator(ty);
        }
        Err("declaratorのパースに失敗しました")
    }

    // func_def ::= "(" (declarator ("," declarator)*)? ")" compound_stmt
    fn func_def(&mut self, global_info: Var) -> Result<Box<Function>, &str> {
        if self.consume_punctuator("(") {
            // 関数の引数のパース（型情報もパース）
            let mut func = Function::new(&global_info.name);
            loop {
                if let Ok(param) = self.declarator() {
                    func.gen_lvar(param).unwrap();
                }
                if !self.consume_punctuator(",") {
                    break;
                }
            }
            self.expect_punctuator(")").unwrap();
            if self.consume_punctuator(";") {
                // 関数プロトタイプ宣言
                return Err("関数プロトタイプ宣言はサポートされていません");
            }

            self.current_func = Some(Box::new(func)); // 現在の関数を設定

            // 関数本体のパース
            if let Some(node) = self.compound_stmt() {
                self.current_func.as_mut().unwrap().body = node.body;
                return Ok(self.current_func.take().unwrap());
            }
            return Err("関数本体のパースに失敗しました");
        }
        Err("関数の引数リストのパースに失敗しました")
    }

    // declaration ::= declarator ";"
    fn declaration(&mut self) -> Option<Result<Var, Var>> {
        // declaratorのみパース出来た場合は，ErrとしてVarを返す
        if let Ok(var) = self.declarator() {
            if self.consume_punctuator(";") {
                return Some(Ok(var));
            }
            return Some(Err(var));
        }
        None
    }

    // TODO: case文, default文の実装
    fn labeled_stmt(&mut self) -> Option<Box<Node>> {
        if let Some(label_name) = self.consume_ident() {
            if self.consume_punctuator(":") {
                let mut node = Node::new_unary(NodeKind::Label, self.stmt());
                node.label_name = label_name;
                return Some(Box::new(node));
            } else {
                // ラベル名ではなかった場合、トークンを元に戻す
                self.tokens.insert(0, Token::Identifier(label_name));
            }
        }
        None
    }

    // compound_stmt ::= "{" declaration* stmt* "}"
    fn compound_stmt(&mut self) -> Option<Box<Node>> {
        if self.consume_punctuator("{") {
            let mut node = Node::from(NodeKind::Block);
            while !self.consume_punctuator("}") {
                if let Some(Ok(var)) = self.declaration() {
                    self.current_func.as_mut().unwrap().gen_lvar(var).unwrap();
                    continue;
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
        if self.consume_keyword("if") {
            let mut node = Node::from(NodeKind::If);
            self.expect_punctuator("(").unwrap();
            node.cond = self.expr();
            self.expect_punctuator(")").unwrap();
            node.then = self.stmt();
            if self.consume_keyword("else") {
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
        if self.consume_keyword("while") {
            let mut node = Node::from(NodeKind::While);
            self.expect_punctuator("(").unwrap();
            node.cond = self.expr();
            self.expect_punctuator(")").unwrap();
            node.then = self.stmt();
            return Some(Box::new(node));
        }

        if self.consume_keyword("do") {
            let mut node = Node::from(NodeKind::Do);
            node.then = self.stmt();
            self.expect_reserved("while").unwrap();
            self.expect_punctuator("(").unwrap();
            node.cond = self.expr();
            self.expect_punctuator(")").unwrap();
            self.expect_punctuator(";").unwrap();
            return Some(Box::new(node));
        }

        if self.consume_keyword("for") {
            let mut node = Node::from(NodeKind::For);
            self.expect_punctuator("(").unwrap();
            // 初期化式
            if !self.consume_punctuator(";") {
                node.init = self.expr();
                self.expect_punctuator(";").unwrap();
            }
            // 条件式
            if !self.consume_punctuator(";") {
                node.cond = self.expr();
                self.expect_punctuator(";").unwrap();
            }
            // 更新式
            if !self.consume_punctuator(")") {
                node.inc = self.expr();
                self.expect_punctuator(")").unwrap();
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
        if self.consume_keyword("goto") {
            let mut node = Node::from(NodeKind::Goto);
            node.label_name = self.consume_ident().unwrap();
            self.expect_punctuator(";").unwrap();
            return Some(Box::new(node));
        }

        if self.consume_keyword("continue") {
            let node = Node::from(NodeKind::Continue);
            self.expect_punctuator(";").unwrap();
            return Some(Box::new(node));
        }

        if self.consume_keyword("break") {
            let node = Node::from(NodeKind::Break);
            self.expect_punctuator(";").unwrap();
            return Some(Box::new(node));
        }

        if self.consume_keyword("return") {
            if self.consume_punctuator(";") {
                return Some(Box::new(Node::from(NodeKind::Return)));
            }

            let node = Node::new_unary(NodeKind::Return, self.expr());
            self.expect_punctuator(";").unwrap();
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
        if self.consume_punctuator(";") {
            return Some(Box::new(Node::from(NodeKind::Nop)));
        } else {
            let expr_node = self.expr();
            self.expect_punctuator(";").unwrap();
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
                let mut rhs = self.mul_expr();
                if let Some(ty) = &node.as_ref().unwrap().ty {
                    if ty.kind == TypeKind::Ptr || ty.kind == TypeKind::Array {
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
            } else if self.consume_punctuator("-") {
                // subtraction
                let mut rhs = self.mul_expr();
                if let Some(ty) = &node.as_ref().unwrap().ty {
                    if ty.kind == TypeKind::Ptr || ty.kind == TypeKind::Array {
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
    fn cast_expr(&mut self) -> Option<Box<Node>> {
        self.unary_expr()
    }

    // unary_expr ::= postfix_expr
    //                | ("++" | "--") unary_expr
    //                | ( "&" | "*" | "+" | "-" | "~" | "!") cast_expr
    //                | sizeof unary_expr
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

        let unary_ops = ["&", "*", "~", "!"];
        for op in &unary_ops {
            if self.consume_punctuator(op) {
                let kind = NodeKind::from_str(op).unwrap();
                return Some(Box::new(Node::new_unary(kind, self.cast_expr())));
            }
        }

        if self.consume_keyword("sizeof") {
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
    //                  | num
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
                let mut node = Node::new_lvar(&lvar.name, lvar.offset, &lvar.ty);

                // 配列の場合はポインタ型に変換
                if self.consume_punctuator("[") {
                    let add = Node::new(NodeKind::Add, Some(Box::new(node)), self.expr());
                    node = Node::new_unary(NodeKind::Deref, Some(Box::new(add)));
                    self.expect_punctuator("]").unwrap();
                }
                return Some(Box::new(node));
            } else if let Some(gvar) = self.find_gvar(&name) {
                // グローバル変数ノードを作成
                let mut node = Node::new_gvar(&gvar.name, &gvar.ty);

                // 配列の場合はポインタ型に変換
                if self.consume_punctuator("[") {
                    let add = Node::new(NodeKind::Add, Some(Box::new(node)), self.expr());
                    node = Node::new_unary(NodeKind::Deref, Some(Box::new(add)));
                    self.expect_punctuator("]").unwrap();
                }
                return Some(Box::new(node));
            }
            panic!("未定義の関数もしくは変数です: {}", name);
        }
        Some(Box::new(Node::new_num(self.expect_number().unwrap())))
    }
}
