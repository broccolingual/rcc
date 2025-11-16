use crate::ast::{Ast, Var};
use crate::node::Node;
use crate::types::{
    DeclarationSpecifier, FunctionKind, StorageClassKind, Type, TypeKind, TypeQualifierKind,
    TypeSpecifierQualifier,
};

impl Ast {
    // declaration ::= declaration_specifiers init_declarator_list ";"
    pub(super) fn declaration(&mut self) -> Option<Vec<Var>> {
        let specifiers = self.declaration_specifiers();
        if specifiers.is_empty() {
            return None;
        }
        let base_kind = Type::from(&specifiers).unwrap().kind;
        let vars = self.init_declarator_list(base_kind);
        if vars.is_empty() {
            return None;
        }
        self.expect_punctuator(";").unwrap();
        Some(vars)
    }

    // declaration_specifiers ::= declaration_specifier+
    pub(super) fn declaration_specifiers(&mut self) -> Vec<DeclarationSpecifier> {
        let mut specifiers = Vec::new();
        while let Some(specifier) = self.declaration_specifier() {
            specifiers.push(specifier);
        }
        specifiers
    }

    // declaration_specifier ::= storage_class_specifier | type_specifier_qualifier | function_specifier
    pub(super) fn declaration_specifier(&mut self) -> Option<DeclarationSpecifier> {
        if let Some(storage_class_specifier) = self.storage_class_specifier() {
            return Some(DeclarationSpecifier::StorageClassSpecifier(
                storage_class_specifier,
            ));
        }
        if let Some(type_specifier_qualifier) = self.type_specifier_qualifier() {
            return Some(DeclarationSpecifier::TypeSpecifierQualifier(
                type_specifier_qualifier,
            ));
        }
        if let Some(function_specifier) = self.function_specifier() {
            return Some(DeclarationSpecifier::FunctionSpecifier(function_specifier));
        }
        None
    }

    // init_declarator_list ::= init_declarator ("," init_declarator)*
    fn init_declarator_list(&mut self, base_kind: TypeKind) -> Vec<Var> {
        let mut vars = Vec::new();
        if let Some(var) = self.init_declarator(base_kind.clone()) {
            vars.push(*var);
        }
        while self.consume_punctuator(",") {
            if let Some(var) = self.init_declarator(base_kind.clone()) {
                vars.push(*var);
            }
        }
        vars
    }

    // init_declarator ::= declarator
    //                     | declarator "=" initializer
    fn init_declarator(&mut self, base_kind: TypeKind) -> Option<Box<Var>> {
        if let Ok(mut var) = self.declarator(base_kind) {
            if self.consume_punctuator("=") {
                if let Some(init) = self.initializer() {
                    let mut init = Some(init);
                    self.assign_types(&mut init); // initializerの型を設定
                    if &var.ty != init.as_ref().unwrap().ty.as_ref().unwrap() {
                        panic!(
                            "initializerの型が変数の型と一致しません {:?} <= {:?}",
                            var.ty,
                            init.as_ref().unwrap().ty.as_ref().unwrap(),
                        );
                    } // 型チェック
                    var.init = init; // initializerを設定
                } else {
                    panic!("initializerのパースに失敗しました");
                }
            }
            return Some(var);
        }
        None
    }

    // storage_class_specifier ::= "auto" | "constexpr" | "extern" | "register" | "static" | "thread_local" | "typedef"
    fn storage_class_specifier(&mut self) -> Option<StorageClassKind> {
        StorageClassKind::all()
            .into_iter()
            .find(|specifier| self.consume_keyword(&specifier.to_string()))
    }

    // type_specifier ::= "void" | "char" | "short" | "int" | "long" | "float" | "double" | "bool"
    fn type_specifier(&mut self) -> Option<TypeKind> {
        TypeKind::all()
            .into_iter()
            .find(|specifier| self.consume_keyword(&specifier.to_string()))
    }

    // specifier_qualifier_list ::= type_specifier_qualifier+
    #[allow(dead_code)]
    fn specifier_qualifier_list(&mut self) -> Vec<TypeSpecifierQualifier> {
        let mut specifiers = Vec::new();
        while let Some(specifier) = self.type_specifier_qualifier() {
            specifiers.push(specifier);
        }
        specifiers
    }

    // type_specifier_qualifier ::= type_specifier | type_qualifier
    fn type_specifier_qualifier(&mut self) -> Option<TypeSpecifierQualifier> {
        if let Some(specifier) = self.type_specifier() {
            return Some(TypeSpecifierQualifier::TypeSpecifier(specifier));
        }
        if let Some(qualifier) = self.type_qualifier() {
            return Some(TypeSpecifierQualifier::TypeQualifier(qualifier));
        }
        None
    }

    // type_qualifier ::= "const" | "volatile" | "restrict"
    fn type_qualifier(&mut self) -> Option<TypeQualifierKind> {
        TypeQualifierKind::all()
            .into_iter()
            .find(|qualifier| self.consume_keyword(&qualifier.to_string()))
    }

    // function_specifier ::= "inline"
    fn function_specifier(&mut self) -> Option<FunctionKind> {
        FunctionKind::all()
            .into_iter()
            .find(|specifier| self.consume_keyword(&specifier.to_string()))
    }

    // type_qualifier_list ::= type_qualifier*
    fn type_qualifier_list(&mut self) -> Vec<TypeQualifierKind> {
        let mut qualifiers = Vec::new();
        while let Some(qualifier) = self.type_qualifier() {
            qualifiers.push(qualifier);
        }
        qualifiers
    }

    // pointer ::= "*" type_qualifier_list* pointer?
    #[allow(clippy::never_loop)]
    fn pointer(&mut self, base_ty: Box<Type>) -> Box<Type> {
        while self.consume_punctuator("*") {
            return self.pointer(Box::new(Type::new_ptr(&base_ty)));
        }
        self.type_qualifier_list(); // 現状は型修飾子を無視
        base_ty
    }

    // declarator ::= pointer? direct_declarator
    pub(super) fn declarator(&mut self, base_kind: TypeKind) -> Result<Box<Var>, &str> {
        let ty = self.pointer(Box::new(Type::new(base_kind)));
        self.direct_declarator(ty)
    }

    // direct_declarator ::= "(" declarator ")"
    //                       | ident
    //                       | array_declarator
    //                       | function_declarator
    fn direct_declarator(&mut self, ty: Box<Type>) -> Result<Box<Var>, &str> {
        let mut var = if self.consume_punctuator("(") {
            if let Ok(v) = self.declarator(ty.kind.clone()) {
                self.expect_punctuator(")").unwrap();
                v
            } else {
                return Err("direct_declarator: parentheses declarator failed");
            }
        } else if let Some(name) = self.consume_ident() {
            Box::new(Var::new(&name, *ty.clone()))
        } else {
            return Err("direct_declaratorのパースに失敗しました");
        };

        loop {
            // array_declarator
            if self.consume_punctuator("[") {
                let array_size = self.expect_number().unwrap() as usize;
                self.expect_punctuator("]").unwrap();
                // TODO: 多次元配列の場合，逆順で定義されてしまう
                let array_ty = Type::new_array(&var.ty, array_size);
                var = Box::new(Var::new(&var.name, array_ty));
                continue;
            }
            // function_declarator
            if self.consume_punctuator("(") {
                let params = self.parameter_type_list();
                self.expect_punctuator(")").unwrap();
                let func_ty = Type::new_func(&var.ty, params.to_vec());
                var = Box::new(Var::new(&var.name, func_ty));
                continue;
            }
            break;
        }
        Ok(var)
    }

    // parameter_type_list ::= parameter_list
    fn parameter_type_list(&mut self) -> Vec<Var> {
        self.parameter_list()
    }

    // parameter_list ::= parameter_declaration ("," parameter_declaration)*
    fn parameter_list(&mut self) -> Vec<Var> {
        let mut params = Vec::new();
        if let Ok(param) = self.parameter_declaration() {
            params.push(*param);
        }
        while self.consume_punctuator(",") {
            if let Ok(param) = self.parameter_declaration() {
                params.push(*param);
            }
        }
        params
    }

    // parameter_declaration ::= declaration_specifiers declarator
    fn parameter_declaration(&mut self) -> Result<Box<Var>, &str> {
        let specifiers = self.declaration_specifiers();
        if !specifiers.is_empty() {
            let base_kind = Type::from(&specifiers).unwrap().kind;
            if let Ok(var) = self.declarator(base_kind) {
                return Ok(var);
            }
        }
        Err("parameter_declarationのパースに失敗しました")
    }

    // initializer ::= assignment_expr
    //                 | braced_initializer // TODO: 未実装
    fn initializer(&mut self) -> Option<Box<Node>> {
        self.assign_expr()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeKind;
    use crate::parser::Tokenizer;

    fn preproc(input: &str) -> Ast {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize(input).unwrap();
        Ast::new(&tokens)
    }

    #[test]
    fn test_declaration() {
        let input = "int a;";
        let mut ast = preproc(input);
        let vars = ast.declaration().unwrap();
        let var = &vars[0];
        assert_eq!(var.name, "a");
        assert_eq!(var.ty.kind, TypeKind::Int);

        let input = "int *p;";
        let mut ast = preproc(input);
        let vars = ast.declaration().unwrap();
        let var = &vars[0];
        assert_eq!(var.name, "p");
        assert_eq!(var.ty.kind, TypeKind::Ptr);
        assert_eq!(var.ty.ptr_to.as_ref().unwrap().kind, TypeKind::Int);

        let input = "int **p;";
        let mut ast = preproc(input);
        let vars = ast.declaration().unwrap();
        let var = &vars[0];
        assert_eq!(var.name, "p");
        assert_eq!(var.ty.kind, TypeKind::Ptr);
        assert_eq!(var.ty.ptr_to.as_ref().unwrap().kind, TypeKind::Ptr);
        assert_eq!(
            var.ty
                .ptr_to
                .as_ref()
                .unwrap()
                .ptr_to
                .as_ref()
                .unwrap()
                .kind,
            TypeKind::Int
        );

        let input = "int arr[10];";
        let mut ast = preproc(input);
        let vars = ast.declaration().unwrap();
        let var = &vars[0];
        assert_eq!(var.name, "arr");
        assert_eq!(var.ty.kind, TypeKind::Array);
        assert_eq!(var.ty.array_size, 10);
        assert_eq!(var.ty.ptr_to.as_ref().unwrap().kind, TypeKind::Int);

        // TODO: 多次元配列の要素数の宣言が逆順になる問題の修正
        // let input = "int arr[3][5];";
        // let mut ast = preproc(input);
        // let vars = ast.declaration().unwrap();
        // let var = &vars[0];
        // assert_eq!(var.name, "arr");
        // assert_eq!(var.ty.kind, TypeKind::Array);
        // assert_eq!(var.ty.array_size, 3);
        // let inner_ty = var.ty.ptr_to.as_ref().unwrap();
        // assert_eq!(inner_ty.kind, TypeKind::Array);
        // assert_eq!(inner_ty.array_size, 5);
        // assert_eq!(inner_ty.ptr_to.as_ref().unwrap().kind, TypeKind::Int);

        let input = "int *arr[10];";
        let mut ast = preproc(input);
        let vars = ast.declaration().unwrap();
        let var = &vars[0];
        assert_eq!(var.name, "arr");
        assert_eq!(var.ty.kind, TypeKind::Array);
        assert_eq!(var.ty.array_size, 10);
        let inner_ty = var.ty.ptr_to.as_ref().unwrap();
        assert_eq!(inner_ty.kind, TypeKind::Ptr);
        assert_eq!(inner_ty.ptr_to.as_ref().unwrap().kind, TypeKind::Int);

        let input = "int a, b;";
        let mut ast = preproc(input);
        let vars = ast.declaration().unwrap();
        assert!(vars.len() == 2);
        assert_eq!(vars[0].name, "a");
        assert_eq!(vars[1].name, "b");
        assert_eq!(vars[0].ty.kind, TypeKind::Int);
        assert_eq!(vars[1].ty.kind, TypeKind::Int);

        let input = "int a = 3;";
        let mut ast = preproc(input);
        let vars = ast.declaration().unwrap();
        let var = &vars[0];
        assert_eq!(var.name, "a");
        assert_eq!(var.ty.kind, TypeKind::Int);
        assert!(var.init.is_some());
        let init = var.init.as_ref().unwrap();
        assert_eq!(init.kind, NodeKind::Number);
        assert_eq!(init.val, 3);

        let input = "int a = 3, b = 5;";
        let mut ast = preproc(input);
        let vars = ast.declaration().unwrap();
        assert!(vars.len() == 2);
        let var_a = &vars[0];
        let var_b = &vars[1];
        assert_eq!(var_a.name, "a");
        assert_eq!(var_b.name, "b");
        assert_eq!(var_a.ty.kind, TypeKind::Int);
        assert_eq!(var_b.ty.kind, TypeKind::Int);
        assert!(var_a.init.is_some());
        assert!(var_b.init.is_some());
        let init_a = var_a.init.as_ref().unwrap();
        let init_b = var_b.init.as_ref().unwrap();
        assert_eq!(init_a.kind, NodeKind::Number);
        assert_eq!(init_a.val, 3);
        assert_eq!(init_b.kind, NodeKind::Number);
        assert_eq!(init_b.val, 5);
    }
}
