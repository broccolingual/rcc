use crate::ast::{Ast, Var};
use crate::types::Type;

impl Ast {
    // declaration ::= declarator ";"
    pub(super) fn declaration(&mut self) -> Option<Result<Var, Var>> {
        // declaratorのみパース出来た場合は，ErrとしてVarを返す
        if let Ok(var) = self.declarator() {
            if self.consume_punctuator(";") {
                return Some(Ok(var));
            }
            return Some(Err(var));
        }
        None
    }

    // declaration_specifiers ::= declaration_specifier+
    fn declaration_specifiers(&mut self) -> Vec<String> {
        let mut specifiers = Vec::new();
        while let Some(specifier) = self.declaration_specifier() {
            specifiers.push(specifier);
        }
        specifiers
    }

    // declaration_specifier ::= storage_class_specifier | type_specifier_qualifier | function_specifier
    fn declaration_specifier(&mut self) -> Option<String> {
        if let Some(storage_class_specifier) = self.storage_class_specifier() {
            return Some(storage_class_specifier);
        }
        if let Some(type_specifier_qualifier) = self.type_specifier_qualifier() {
            return Some(type_specifier_qualifier);
        }
        if let Some(function_specifier) = self.function_specifier() {
            return Some(function_specifier);
        }
        None
    }

    // init_declarator ::= declarator
    fn init_declarator(&mut self) -> Option<Var> {
        unimplemented!("init_declaratorは未実装です");
    }

    // storage_class_specifier ::= "auto" | "constexpr" | "extern" | "register" | "static" | "thread_local" | "typedef"
    fn storage_class_specifier(&mut self) -> Option<String> {
        for specifier in [
            "auto",
            "constexpr",
            "extern",
            "register",
            "static",
            "thread_local",
            "typedef",
        ] {
            if self.consume_keyword(specifier) {
                return Some(specifier.to_string());
            }
        }
        None
    }

    // type_specifier ::= "void" | "char" | "short" | "int" | "long" | "float" | "double" | "signed" | "unsigned" | "bool"
    fn type_specifier(&mut self) -> Option<String> {
        for specifier in [
            "void", "char", "short", "int", "long", "float", "double", "signed", "unsigned", "bool",
        ] {
            if self.consume_keyword(specifier) {
                return Some(specifier.to_string());
            }
        }
        None
    }

    // function_specifier ::= "inline"
    fn function_specifier(&mut self) -> Option<String> {
        for specifier in ["inline"] {
            if self.consume_keyword(specifier) {
                return Some(specifier.to_string());
            }
        }
        None
    }

    // specifier_qualifier_list ::= type_specifier_qualifier+
    fn specifier_qualifier_list(&mut self) -> Vec<String> {
        let mut specifiers = Vec::new();
        while let Some(specifier) = self.type_specifier_qualifier() {
            specifiers.push(specifier);
        }
        specifiers
    }

    // type_specifier_qualifier ::= type_specifier | type_qualifier
    fn type_specifier_qualifier(&mut self) -> Option<String> {
        if let Some(specifier) = self.type_specifier() {
            return Some(specifier);
        }
        if let Some(qualifier) = self.type_qualifier() {
            return Some(qualifier);
        }
        None
    }

    // type_qualifier ::= "const" | "volatile" | "restrict"
    fn type_qualifier(&mut self) -> Option<String> {
        for qualifier in ["const", "volatile", "restrict"] {
            if self.consume_keyword(qualifier) {
                return Some(qualifier.to_string());
            }
        }
        None
    }

    // type_qualifier_list ::= type_qualifier*
    fn type_qualifier_list(&mut self) -> Vec<String> {
        let mut qualifiers = Vec::new();
        while let Some(qualifier) = self.type_qualifier() {
            qualifiers.push(qualifier);
        }
        qualifiers
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
    pub(super) fn declarator(&mut self) -> Result<Var, &str> {
        if self.consume_keyword("int") {
            // ポインタを処理
            let ty = self.pointer(Type::new_int());

            // 変数名を取得
            return self.direct_declarator(ty);
        }
        Err("declaratorのパースに失敗しました")
    }
}
