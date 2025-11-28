use crate::ast::{Ast, Var};
use crate::errors::CompileError;
use crate::node::Node;
use crate::types::{
    DeclarationSpecifier, FunctionKind, StorageClassKind, Type, TypeKind, TypeQualifierKind,
    TypeSpecifierQualifier,
};

impl Ast {
    // declaration ::= declaration_specifiers init_declarator_list ";"
    pub(super) fn declaration(&mut self) -> Result<Option<Vec<Var>>, CompileError> {
        let specifiers = self.declaration_specifiers();
        if specifiers.is_empty() {
            return Ok(None);
        }
        let base_type = Type::from(&specifiers).unwrap();
        let vars = self.init_declarator_list(base_type)?;
        if vars.is_empty() {
            return Ok(None);
        }
        self.expect_punctuator(";")?;
        Ok(Some(vars))
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
    fn init_declarator_list(&mut self, base_type: Type) -> Result<Vec<Var>, CompileError> {
        let mut vars = Vec::new();
        if let Some(var) = self.init_declarator(base_type.clone())? {
            vars.push(*var);
        }
        while self.consume_punctuator(",").is_some() {
            if let Some(var) = self.init_declarator(base_type.clone())? {
                vars.push(*var);
            }
        }
        Ok(vars)
    }

    // init_declarator ::= declarator
    //                     | declarator "=" initializer
    fn init_declarator(&mut self, base_type: Type) -> Result<Option<Box<Var>>, CompileError> {
        if let Ok(mut var) = self.declarator(base_type) {
            if self.consume_punctuator("=").is_some() {
                if let Some(init) = self.initializer()? {
                    // TODO: 数字を代入する際の扱いを考える
                    // init.assign_types()?; // initializerの型を設定
                    // if let Some(ty) = &init.ty
                    //     && ty != &var.ty
                    // {
                    //     return Err(AstError::TypeError(format!(
                    //         "initializerの型が変数の型と一致しません {} != {}",
                    //         var.ty, ty
                    //     )));
                    // }
                    var.init = Some(init); // initializerを設定
                } else {
                    return Err(CompileError::InvalidInitializer {
                        msg: "無効な初期化子です".to_string(),
                    });
                }
            }
            return Ok(Some(var));
        }
        Ok(None)
    }

    // storage_class_specifier ::= "auto" | "constexpr" | "extern" | "register" | "static" | "thread_local" | "typedef"
    fn storage_class_specifier(&mut self) -> Option<StorageClassKind> {
        StorageClassKind::all()
            .into_iter()
            .find(|specifier| self.consume_keyword(&specifier.to_string()).is_some())
    }

    // type_specifier ::= "void" | "char" | "short" | "int" | "long" | "float" | "double"
    fn type_specifier(&mut self) -> Option<TypeKind> {
        TypeKind::all()
            .into_iter()
            .find(|specifier| self.consume_keyword(&specifier.to_string()).is_some())
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
            .find(|qualifier| self.consume_keyword(&qualifier.to_string()).is_some())
    }

    // function_specifier ::= "inline"
    fn function_specifier(&mut self) -> Option<FunctionKind> {
        FunctionKind::all()
            .into_iter()
            .find(|specifier| self.consume_keyword(&specifier.to_string()).is_some())
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
        while self.consume_punctuator("*").is_some() {
            return self.pointer(Box::new(Type::new(&TypeKind::Ptr { to: base_ty })));
        }
        self.type_qualifier_list(); // 現状は型修飾子を無視
        base_ty
    }

    // declarator ::= pointer? direct_declarator
    pub(super) fn declarator(&mut self, base_type: Type) -> Result<Box<Var>, CompileError> {
        let ty = self.pointer(Box::new(base_type));
        self.direct_declarator(ty)
    }

    // direct_declarator ::= "(" declarator ")"
    //                       | ident
    //                       | array_declarator
    //                       | function_declarator
    fn direct_declarator(&mut self, ty: Box<Type>) -> Result<Box<Var>, CompileError> {
        let mut var = if self.consume_punctuator("(").is_some() {
            if let Ok(v) = self.declarator(*ty.clone()) {
                self.expect_punctuator(")")?;
                v
            } else {
                return Err(CompileError::InvalidDeclaration {
                    msg: "無効な宣言子です".to_string(),
                });
            }
        } else if let Some(name) = self.consume_ident() {
            Box::new(Var::new(&name, *ty.clone()))
        } else {
            return Err(CompileError::InvalidDeclaration {
                msg: "無効な宣言子です".to_string(),
            });
        };

        loop {
            // array_declarator
            if self.consume_punctuator("[").is_some() {
                let array_size = self.expect_number()? as usize;
                self.expect_punctuator("]")?;
                // TODO: 多次元配列の場合，逆順で定義されてしまう
                let array_ty = Type::new(&TypeKind::Array {
                    base: var.ty,
                    size: array_size,
                });
                var = Box::new(Var::new(&var.name, array_ty));
                continue;
            }
            // function_declarator
            if self.consume_punctuator("(").is_some() {
                // パラメータが0個の場合
                if self.consume_punctuator(")").is_some() {
                    let func_ty = Type::new(&TypeKind::Func {
                        return_ty: var.ty,
                        params: Vec::new(),
                    });
                    var = Box::new(Var::new(&var.name, func_ty));
                    continue;
                }

                // パラメータが1個以上の場合
                let params = self.parameter_type_list()?;
                self.expect_punctuator(")")?;
                let func_ty = Type::new(&TypeKind::Func {
                    return_ty: var.ty,
                    params,
                });
                var = Box::new(Var::new(&var.name, func_ty));
                continue;
            }
            break;
        }
        Ok(var)
    }

    // parameter_type_list ::= parameter_list
    fn parameter_type_list(&mut self) -> Result<Vec<Var>, CompileError> {
        self.parameter_list()
    }

    //
    // parameter_list ::= parameter_declaration ("," parameter_declaration)*
    fn parameter_list(&mut self) -> Result<Vec<Var>, CompileError> {
        let mut params = Vec::new();
        let param = self.parameter_declaration()?;
        params.push(*param);
        while self.consume_punctuator(",").is_some() {
            let param = self.parameter_declaration()?;
            params.push(*param);
        }
        Ok(params)
    }

    // parameter_declaration ::= declaration_specifiers declarator
    fn parameter_declaration(&mut self) -> Result<Box<Var>, CompileError> {
        let specifiers = self.declaration_specifiers();
        if !specifiers.is_empty() {
            let base_kind = Type::from(&specifiers).unwrap();
            if let Ok(var) = self.declarator(base_kind) {
                return Ok(var);
            }
        }
        Err(CompileError::InvalidDeclaration {
            msg: "無効なパラメータ宣言です".to_string(),
        })
    }

    // initializer ::= assignment_expr
    //                 | braced_initializer
    fn initializer(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        if let Some(init) = self.braced_initializer()? {
            return Ok(Some(init));
        }
        self.assign_expr()
    }

    // braced_initializer ::= "{" "}" // TODO: その他未実装
    fn braced_initializer(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        if self.consume_punctuator("{").is_some() {
            self.expect_punctuator("}")?;
            return Ok(None);
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::node::NodeKind;

    fn preproc(input: &str) -> Ast {
        let lexer = Lexer::new();
        let tokens = lexer.tokenize(input).unwrap();
        Ast::new(&tokens)
    }

    #[test]
    fn test_declaration() {
        let input = "int a;";
        let mut ast = preproc(input);
        let vars = ast.declaration().unwrap().unwrap();
        let var = &vars[0];
        assert_eq!(var.name, "a");
        assert_eq!(*var.ty, Type::new(&TypeKind::Int));

        let input = "int *p;";
        let mut ast = preproc(input);
        let vars = ast.declaration().unwrap().unwrap();
        let var = &vars[0];
        assert_eq!(var.name, "p");
        assert_eq!(
            *var.ty,
            Type::new(&TypeKind::Ptr {
                to: Box::new(Type::new(&TypeKind::Int))
            })
        );

        let input = "int **p;";
        let mut ast = preproc(input);
        let vars = ast.declaration().unwrap().unwrap();
        let var = &vars[0];
        assert_eq!(var.name, "p");
        assert_eq!(
            *var.ty,
            Type::new(&TypeKind::Ptr {
                to: Box::new(Type::new(&TypeKind::Ptr {
                    to: Box::new(Type::new(&TypeKind::Int))
                }))
            })
        );

        let input = "int arr[10];";
        let mut ast = preproc(input);
        let vars = ast.declaration().unwrap().unwrap();
        let var = &vars[0];
        assert_eq!(var.name, "arr");
        assert_eq!(
            *var.ty,
            Type::new(&TypeKind::Array {
                base: Box::new(Type::new(&TypeKind::Int)),
                size: 10
            })
        );

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
        let vars = ast.declaration().unwrap().unwrap();
        let var = &vars[0];
        assert_eq!(var.name, "arr");
        assert_eq!(
            *var.ty,
            Type::new(&TypeKind::Array {
                base: Box::new(Type::new(&TypeKind::Ptr {
                    to: Box::new(Type::new(&TypeKind::Int))
                })),
                size: 10
            })
        );

        let input = "int a, b;";
        let mut ast = preproc(input);
        let vars = ast.declaration().unwrap().unwrap();
        assert!(vars.len() == 2);
        assert_eq!(vars[0].name, "a");
        assert_eq!(vars[1].name, "b");
        assert_eq!(*vars[0].ty, Type::new(&TypeKind::Int));
        assert_eq!(*vars[1].ty, Type::new(&TypeKind::Int));

        let input = "int a = 3;";
        let mut ast = preproc(input);
        let vars = ast.declaration().unwrap().unwrap();
        let var = &vars[0];
        assert_eq!(var.name, "a");
        assert_eq!(*var.ty, Type::new(&TypeKind::Int));
        assert!(var.init.is_some());
        let init = var.init.as_ref().unwrap();
        assert_eq!(init.kind, NodeKind::Number { val: 3 });

        let input = "int a = 3, b = 5;";
        let mut ast = preproc(input);
        let vars = ast.declaration().unwrap().unwrap();
        assert!(vars.len() == 2);
        let var_a = &vars[0];
        let var_b = &vars[1];
        assert_eq!(var_a.name, "a");
        assert_eq!(var_b.name, "b");
        assert_eq!(*var_a.ty, Type::new(&TypeKind::Int));
        assert_eq!(*var_b.ty, Type::new(&TypeKind::Int));
        assert!(var_a.init.is_some());
        assert!(var_b.init.is_some());
        let init_a = var_a.init.as_ref().unwrap();
        let init_b = var_b.init.as_ref().unwrap();
        assert_eq!(init_a.kind, NodeKind::Number { val: 3 });
        assert_eq!(init_b.kind, NodeKind::Number { val: 5 });
    }
}
