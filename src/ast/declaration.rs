use crate::ast::Ast;
use crate::errors::CompileError;
use crate::node::Node;
use crate::types::{
    Declaration, DeclarationSpecifier, FunctionKind, StorageClassKind, Type, TypeKind,
    TypeQualifierKind, TypeSpecifierQualifier,
};

impl Ast {
    // declaration ::= declaration_specifiers init_declarator_list ";"
    pub(super) fn declaration(&mut self) -> Result<Option<Vec<Declaration>>, CompileError> {
        let specifiers = self.declaration_specifiers()?;
        if specifiers.is_empty() {
            return Ok(None);
        }
        let base_ty =
            Type::from_ds(&specifiers).ok_or_else(|| CompileError::InvalidDeclaration {
                msg: "無効な型指定子です".to_string(),
            })?;
        let vars = self.init_declarator_list(&base_ty)?;
        if vars.is_empty() {
            return Ok(None);
        }
        self.expect_punctuator(";")?;
        Ok(Some(vars))
    }

    // declaration_specifiers ::= declaration_specifier+
    pub(super) fn declaration_specifiers(
        &mut self,
    ) -> Result<Vec<DeclarationSpecifier>, CompileError> {
        let mut specifiers = Vec::new();
        while let Some(specifier) = self.declaration_specifier()? {
            specifiers.push(specifier);
        }
        Ok(specifiers)
    }

    // declaration_specifier ::= storage_class_specifier | type_specifier_qualifier | function_specifier
    pub(super) fn declaration_specifier(
        &mut self,
    ) -> Result<Option<DeclarationSpecifier>, CompileError> {
        if let Some(storage_class_specifier) = self.storage_class_specifier() {
            return Ok(Some(DeclarationSpecifier::StorageClassSpecifier(
                storage_class_specifier,
            )));
        }
        if let Some(type_specifier_qualifier) = self.type_specifier_qualifier()? {
            return Ok(Some(DeclarationSpecifier::TypeSpecifierQualifier(
                type_specifier_qualifier,
            )));
        }
        if let Some(function_specifier) = self.function_specifier() {
            return Ok(Some(DeclarationSpecifier::FunctionSpecifier(
                function_specifier,
            )));
        }
        Ok(None)
    }

    // init_declarator_list ::= init_declarator ("," init_declarator)*
    fn init_declarator_list(&mut self, base_ty: &Type) -> Result<Vec<Declaration>, CompileError> {
        let mut declarations = Vec::new();
        if let Some(declaration) = self.init_declarator(base_ty)? {
            declarations.push(declaration);
        }
        while self.consume_punctuator(",").is_some() {
            if let Some(declaration) = self.init_declarator(base_ty)? {
                declarations.push(declaration);
            }
        }
        Ok(declarations)
    }

    // init_declarator ::= declarator
    //                     | declarator "=" initializer
    fn init_declarator(&mut self, base_ty: &Type) -> Result<Option<Declaration>, CompileError> {
        if let Ok(mut declaration) = self.declarator(base_ty) {
            if self.consume_punctuator("=").is_some() {
                // TODO: 代入時の型チェック
                declaration.init = self.initializer()?; // initializerを設定
                match declaration.ty.kind {
                    // サイズ不明な配列型の場合、初期化子の要素数でサイズを決定
                    TypeKind::Array { ref base, ref size } if *size == 0 => {
                        declaration.ty = Type::from(
                            &TypeKind::Array {
                                base: base.clone(),
                                size: declaration.init.len(),
                            },
                            declaration.ty.is_const,
                        );
                    }
                    _ => {}
                }
            }
            return Ok(Some(declaration));
        }
        Ok(None)
    }

    // storage_class_specifier ::= "auto" | "extern" | "register" | "static" | "typedef"
    fn storage_class_specifier(&mut self) -> Option<StorageClassKind> {
        StorageClassKind::all()
            .into_iter()
            .find(|specifier| self.consume_keyword(&specifier.to_string()).is_some())
    }

    // type_specifier ::= "void" | "char" | "short" | "int" | "long" | "float" | "double" | struct_or_union_specifier
    fn type_specifier(&mut self) -> Result<Option<TypeKind>, CompileError> {
        if let Some(ty) = self.struct_or_union_specifier()? {
            return Ok(Some(ty));
        }
        Ok(TypeKind::all()
            .into_iter()
            .find(|specifier| self.consume_keyword(&specifier.to_string()).is_some()))
    }

    // struct_or_union_specifier ::= "struct" ident? "{" struct_declaration_list "}"
    fn struct_or_union_specifier(&mut self) -> Result<Option<TypeKind>, CompileError> {
        if self.consume_keyword("struct").is_some() {
            let struct_name = if let Some(name) = self.consume_ident() {
                name
            } else {
                "".to_string()
            };
            self.expect_punctuator("{")?;
            let members = self.struct_declaration_list()?;
            self.expect_punctuator("}")?;
            return Ok(Some(TypeKind::Struct {
                name: struct_name,
                members,
            }));
        }
        Ok(None)
    }

    // struct_declaration_list ::= struct_declaration+
    fn struct_declaration_list(&mut self) -> Result<Vec<Declaration>, CompileError> {
        let mut members: Vec<Declaration> = Vec::new();
        while let Some(member_list) = self.struct_declaration()? {
            members.extend(member_list);
        }
        Ok(members)
    }

    // struct_declaration ::= specifier_qualifier_list struct_declarator_list? ";"
    fn struct_declaration(&mut self) -> Result<Option<Vec<Declaration>>, CompileError> {
        let specifiers = self.specifier_qualifier_list()?;
        if specifiers.is_empty() {
            return Ok(None);
        }
        let base_ty =
            Type::from_tsq(&specifiers).ok_or_else(|| CompileError::InvalidDeclaration {
                msg: "無効な型指定子です".to_string(),
            })?;
        let members = self.struct_declarator_list(&base_ty)?;
        self.expect_punctuator(";")?;
        if members.is_empty() {
            return Ok(None);
        }
        Ok(Some(members))
    }

    // struct_declarator_list ::= struct_declarator ("," struct_declarator)*
    fn struct_declarator_list(&mut self, base_ty: &Type) -> Result<Vec<Declaration>, CompileError> {
        let mut members = Vec::new();
        if let Some(member) = self.struct_declarator(base_ty)? {
            members.push(member);
        }
        while self.consume_punctuator(",").is_some() {
            if let Some(member) = self.struct_declarator(base_ty)? {
                members.push(member);
            }
        }
        Ok(members)
    }

    // struct_declarator ::= declarator
    fn struct_declarator(&mut self, base_ty: &Type) -> Result<Option<Declaration>, CompileError> {
        if let Ok(declaration) = self.declarator(base_ty) {
            return Ok(Some(declaration));
        }
        Ok(None)
    }

    // specifier_qualifier_list ::= type_specifier_qualifier+
    fn specifier_qualifier_list(&mut self) -> Result<Vec<TypeSpecifierQualifier>, CompileError> {
        let mut specifiers = Vec::new();
        while let Some(specifier) = self.type_specifier_qualifier()? {
            specifiers.push(specifier);
        }
        Ok(specifiers)
    }

    // type_specifier_qualifier ::= type_specifier | type_qualifier
    fn type_specifier_qualifier(&mut self) -> Result<Option<TypeSpecifierQualifier>, CompileError> {
        if let Some(specifier) = self.type_specifier()? {
            return Ok(Some(TypeSpecifierQualifier::TypeSpecifier(specifier)));
        }
        if let Some(qualifier) = self.type_qualifier() {
            return Ok(Some(TypeSpecifierQualifier::TypeQualifier(qualifier)));
        }
        Ok(None)
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
    fn pointer(&mut self, base_ty: &Type) -> Type {
        while self.consume_punctuator("*").is_some() {
            let ptr_type = Type::from(
                &TypeKind::Ptr {
                    to: Box::new(base_ty.clone()),
                },
                false,
            );
            return self.pointer(&ptr_type);
        }
        self.type_qualifier_list(); // 現状は型修飾子を無視
        base_ty.clone()
    }

    // declarator ::= pointer? direct_declarator
    pub(super) fn declarator(&mut self, base_ty: &Type) -> Result<Declaration, CompileError> {
        let ty = self.pointer(base_ty);
        self.direct_declarator(&ty)
    }

    // direct_declarator ::= "(" declarator ")"
    //                       | identifier
    //                       | direct_declarator "[" type_qualifier_list? assignment_expression? "]"
    //                       | direct_declarator "(" parameter_type_list ")"
    fn direct_declarator(&mut self, base_ty: &Type) -> Result<Declaration, CompileError> {
        let name = if self.consume_punctuator("(").is_some() {
            let inner_var = self.declarator(base_ty)?;
            self.expect_punctuator(")")?;
            inner_var.name
        } else if let Some(name) = self.consume_ident() {
            name
        } else {
            return Err(CompileError::InvalidDeclaration {
                msg: "識別子または括弧で囲まれた宣言子が必要です".to_string(),
            });
        };

        let final_ty = self.parse_postfix_declarators(base_ty)?;
        Ok(Declaration {
            name,
            ty: final_ty,
            init: Vec::new(),
        })
    }

    // 右結合で解析
    fn parse_postfix_declarators(&mut self, base_ty: &Type) -> Result<Type, CompileError> {
        // "[" type_qualifier_list? assignment_expression? "]"
        if self.consume_punctuator("[").is_some() {
            self.type_qualifier_list(); // 現状は型修飾子を無視
            // TODO: assign_exprに置き換え
            let array_size = if self.peek_punctuator("]") {
                0
            } else {
                let size_expr =
                    self.assign_expr()?
                        .ok_or_else(|| CompileError::InvalidDeclaration {
                            msg: "配列のサイズが必要です".to_string(),
                        })?;
                size_expr.eval_const_expr()? as usize
            };
            self.expect_punctuator("]")?;
            let inner_ty = self.parse_postfix_declarators(base_ty)?;
            Ok(Type::from(
                &TypeKind::Array {
                    base: Box::new(inner_ty),
                    size: array_size,
                },
                false,
            ))
        }
        // "(" parameter_type_list ")"
        else if self.consume_punctuator("(").is_some() {
            let params = if self.consume_punctuator(")").is_some() {
                // パラメータが0個の場合
                Vec::new()
            } else {
                // パラメータが1個以上の場合
                let params = self.parameter_type_list()?;
                self.expect_punctuator(")")?;
                params
            };
            let inner_ty = self.parse_postfix_declarators(base_ty)?;
            Ok(Type::from(
                &TypeKind::Func {
                    return_ty: Box::new(inner_ty),
                    params,
                },
                false,
            ))
        } else {
            Ok(base_ty.clone())
        }
    }

    // parameter_type_list ::= parameter_list
    fn parameter_type_list(&mut self) -> Result<Vec<Declaration>, CompileError> {
        self.parameter_list()
    }

    // parameter_list ::= parameter_declaration ("," parameter_declaration)*
    fn parameter_list(&mut self) -> Result<Vec<Declaration>, CompileError> {
        let mut params = Vec::new();
        let param = self.parameter_declaration()?;
        params.push(param);
        while self.consume_punctuator(",").is_some() {
            let param = self.parameter_declaration()?;
            params.push(param);
        }
        Ok(params)
    }

    // parameter_declaration ::= declaration_specifiers declarator
    fn parameter_declaration(&mut self) -> Result<Declaration, CompileError> {
        let specifiers = self.declaration_specifiers()?;
        if !specifiers.is_empty() {
            let base_kind =
                Type::from_ds(&specifiers).ok_or_else(|| CompileError::InvalidDeclaration {
                    msg: "無効な型指定子です".to_string(),
                })?;
            if let Ok(declaration) = self.declarator(&base_kind) {
                return Ok(declaration);
            }
        }
        Err(CompileError::InvalidDeclaration {
            msg: "無効なパラメータ宣言です".to_string(),
        })
    }

    // type_name ::= specifier_qualifier_list abstract_declarator?
    pub(super) fn type_name(&mut self) -> Result<Type, CompileError> {
        let specifiers = self.specifier_qualifier_list()?;
        if specifiers.is_empty() {
            return Err(CompileError::InvalidDeclaration {
                msg: "無効な型名です".to_string(),
            });
        }
        let base_ty =
            Type::from_tsq(&specifiers).ok_or_else(|| CompileError::InvalidDeclaration {
                msg: "無効な型指定子です".to_string(),
            })?;
        if let Ok(abstract_ty) = self.abstract_declarator(&base_ty) {
            return Ok(abstract_ty);
        }
        Ok(base_ty)
    }

    // abstract_declarator ::= pointer // 未実装
    //                         | pointer? direct_abstract_declarator
    fn abstract_declarator(&mut self, base_ty: &Type) -> Result<Type, CompileError> {
        let ty = self.pointer(base_ty);
        self.direct_abstract_declarator(&ty)
    }

    // direct_abstract_declarator ::= "(" abstract_declarator ")"
    //                                | direct_abstract_declarator "[" type_qualifier_list? assignment_expression? "]"
    //                                | direct_abstract_declarator "(" parameter_type_list ")"
    fn direct_abstract_declarator(&mut self, base_ty: &Type) -> Result<Type, CompileError> {
        let mut current_ty = if self.consume_punctuator("(").is_some() {
            let inner_ty = self.abstract_declarator(base_ty)?;
            self.expect_punctuator(")")?;
            inner_ty
        } else {
            base_ty.clone()
        };
        current_ty = self.parse_abstract_postfix_declarators(&current_ty)?;
        Ok(current_ty)
    }

    // 右結合で解析
    fn parse_abstract_postfix_declarators(&mut self, base_ty: &Type) -> Result<Type, CompileError> {
        // "[" type_qualifier_list? assignment_expression? "]"
        if self.consume_punctuator("[").is_some() {
            self.type_qualifier_list(); // 現状は型修飾子を無視
            let array_size = self.expect_number()? as usize; // TODO: assign_exprに置き換え
            self.expect_punctuator("]")?;
            let inner_ty = self.parse_abstract_postfix_declarators(base_ty)?;
            Ok(Type::from(
                &TypeKind::Array {
                    base: Box::new(inner_ty),
                    size: array_size,
                },
                false,
            ))
        }
        // "(" parameter_type_list ")"
        else if self.consume_punctuator("(").is_some() {
            let params = if self.consume_punctuator(")").is_some() {
                // パラメータが0個の場合
                Vec::new()
            } else {
                // パラメータが1個以上の場合
                let params = self.parameter_type_list()?;
                self.expect_punctuator(")")?;
                params
            };
            let inner_ty = self.parse_abstract_postfix_declarators(base_ty)?;
            Ok(Type::from(
                &TypeKind::Func {
                    return_ty: Box::new(inner_ty),
                    params,
                },
                false,
            ))
        } else {
            Ok(base_ty.clone())
        }
    }

    // initializer ::= assignment_expr
    //                 | "{" initializer_list "}"
    //                 | "{" initializer_list "," "}" // 未対応（initializer_listの処理と重複して問題が発生）
    fn initializer(&mut self) -> Result<Vec<Node>, CompileError> {
        if self.consume_punctuator("{").is_some() {
            let init_list = self.initializer_list()?;
            self.expect_punctuator("}")?;
            return Ok(init_list);
        }
        Ok(vec![*self.assign_expr()?.ok_or_else(|| {
            CompileError::InvalidDeclaration {
                msg: "初期化式が必要です".to_string(),
            }
        })?])
    }

    // initializer_list ::= initializer ("," initializer)*
    fn initializer_list(&mut self) -> Result<Vec<Node>, CompileError> {
        let mut init_list = Vec::new();
        init_list.extend(self.initializer()?);
        while self.consume_punctuator(",").is_some() {
            init_list.extend(self.initializer()?);
        }
        Ok(init_list)
    }
}
