use crate::ast::Ast;
use crate::errors::CompileError;
use crate::function::Func;
use crate::node::{Node, NodeKind};
use crate::types::{
    Decl, DeclSpec, FuncKind, MemberDecl, StorageClassKind, Type, TypeKind, TypeQualKind,
    TypeSpecQual,
};

impl Ast<'_> {
    // external_decl ::= func_def | decl
    // func_def      ::= decl_specs declarator compound_stmt
    // decl          ::= decl_specs init_declarator_list ";"
    pub(super) fn external_decl(&mut self) -> Result<(), CompileError> {
        let specs = self.decl_specs()?;
        if specs.is_empty() {
            let span = self.get_prev_token_span().unwrap_or((0, 0));
            return Err(CompileError::InvalidDecl {
                msg: "外部宣言のパースに失敗しました。型指定子が必要です".to_string(),
                span,
            });
        }

        let base_ty = Type::from_ds(specs).ok_or_else(|| {
            let span = self.get_prev_token_span().unwrap_or((0, 0));
            CompileError::InvalidDecl {
                msg: "無効な型指定子です".to_string(),
                span,
            }
        })?;

        let token_pos = self.token_pos; // 関数定義でなかった場合にバックトラックするために保存
        let first_decl = self.declarator(&base_ty)?;

        // 関数型の場合の分岐
        if let TypeKind::Func { params, return_ty } = &first_decl.ty.kind {
            if self.peek_punct("{") {
                // 関数定義: declarator compound_stmt
                let mut func = Func::new(&first_decl.name);
                for param_decl in params.clone() {
                    func.register_param(param_decl)?;
                }
                func.return_ty = *return_ty.clone();

                self.current_func = Some(func);
                let func_body = self.compound_stmt()?.ok_or_else(|| {
                    let span = self.get_prev_token_span().unwrap_or((0, 0));
                    CompileError::InvalidDecl {
                        msg: "関数本体が必要です".to_string(),
                        span,
                    }
                })?;

                func = self
                    .current_func
                    .take()
                    .ok_or_else(|| CompileError::InternalError {
                        msg: "現在の関数が設定されていません".to_string(),
                    })?;

                if let NodeKind::Block { body } = func_body.kind {
                    func.body = body;
                } else {
                    let span = func_body.span;
                    return Err(CompileError::InvalidDecl {
                        msg: "関数本体がブロックではありません。'{' と '}' で囲まれた複合文が必要です".to_string(),
                        span,
                    });
                }

                self.current_func = None;
                self.funcs.push(func);
                return Ok(());
            } else if self.consume_punct(";").is_some() {
                // 関数プロトタイプ宣言: declarator ";"
                // TODO: 現状では何もせず無視（将来的には関数テーブルに登録するなど）
                return Ok(());
            } else {
                let span = self.get_prev_token_span().unwrap_or((0, 0));
                return Err(CompileError::InvalidDecl {
                    msg: "関数宣言には ';' または '{' が必要です".to_string(),
                    span,
                });
            }
        }

        // グローバル変数宣言の場合: init_declarator_list ";"
        self.token_pos = token_pos; // バックトラックして再度パース
        let decls = self.init_declarator_list(&base_ty)?;
        self.expect_punct(";")?;

        // グローバル変数として登録
        for decl in decls {
            self.register_global_var(decl)?;
        }

        Ok(())
    }

    // decl ::= decl_specs init_declarator_list ";"
    pub(super) fn decl(&mut self) -> Result<Option<Vec<Decl>>, CompileError> {
        let specs = self.decl_specs()?;
        if specs.is_empty() {
            return Ok(None);
        }
        let base_ty = Type::from_ds(specs).ok_or_else(|| {
            let span = self.get_prev_token_span().unwrap_or((0, 0));
            CompileError::InvalidDecl {
                msg: "無効な型指定子です".to_string(),
                span,
            }
        })?;
        let vars = self.init_declarator_list(&base_ty)?;
        if vars.is_empty() {
            return Ok(None);
        }
        self.expect_punct(";")?;
        Ok(Some(vars))
    }

    // decl_specs ::= decl_spec+
    pub(super) fn decl_specs(&mut self) -> Result<Vec<DeclSpec>, CompileError> {
        let mut specs = Vec::new();
        while let Some(spec) = self.decl_spec()? {
            specs.push(spec);
        }
        Ok(specs)
    }

    // decl_spec ::= storage_class_spec | type_spec_qual | func_spec
    pub(super) fn decl_spec(&mut self) -> Result<Option<DeclSpec>, CompileError> {
        if let Some(storage_class_spec) = self.storage_class_spec() {
            return Ok(Some(DeclSpec::StorageClassSpec(storage_class_spec)));
        }
        if let Some(type_spec_qual) = self.type_spec_qual()? {
            return Ok(Some(DeclSpec::TypeSpecQual(type_spec_qual)));
        }
        if let Some(func_spec) = self.func_spec() {
            return Ok(Some(DeclSpec::FuncSpec(func_spec)));
        }
        Ok(None)
    }

    // init_declarator_list ::= init_declarator ("," init_declarator)*
    fn init_declarator_list(&mut self, base_ty: &Type) -> Result<Vec<Decl>, CompileError> {
        let mut decls = Vec::new();
        if let Some(decl) = self.init_declarator(base_ty)? {
            decls.push(decl);
        }
        while self.consume_punct(",").is_some() {
            if let Some(decl) = self.init_declarator(base_ty)? {
                decls.push(decl);
            }
        }
        Ok(decls)
    }

    // init_declarator ::= declarator
    //                   | declarator "=" initializer
    fn init_declarator(&mut self, base_ty: &Type) -> Result<Option<Decl>, CompileError> {
        if let Ok(mut decl) = self.declarator(base_ty) {
            if self.consume_punct("=").is_some() {
                // TODO: 代入時の型チェック
                decl.init = self.initializer()?; // initializerを設定
                match decl.ty.kind {
                    // サイズ不明な配列型の場合、初期化子の要素数でサイズを決定
                    TypeKind::Array { ref base, ref size } if *size == 0 => {
                        decl.ty = Type::from(
                            TypeKind::Array {
                                base: base.clone(),
                                size: decl.init.len(),
                            },
                            decl.ty.is_const,
                        );
                    }
                    _ => {}
                }
            }
            return Ok(Some(decl));
        }
        Ok(None)
    }

    // storage_class_spec ::= "auto" | "extern" | "register" | "static" | "typedef"
    fn storage_class_spec(&mut self) -> Option<StorageClassKind> {
        StorageClassKind::all()
            .into_iter()
            .find(|spec| self.consume_keyword(&spec.to_string()).is_some())
    }

    // type_spec ::= "void" | "char" | "short" | "int" | "long" | "float" | "double" | struct_or_union_spec
    fn type_spec(&mut self) -> Result<Option<TypeKind>, CompileError> {
        if let Some(ty) = self.struct_or_union_spec()? {
            return Ok(Some(ty));
        }
        Ok(TypeKind::all()
            .into_iter()
            .find(|spec| self.consume_keyword(&spec.to_string()).is_some()))
    }

    // struct_or_union_spec ::= "struct" ident? "{" struct_decl_list "}"
    //                        | "struct" ident
    fn struct_or_union_spec(&mut self) -> Result<Option<TypeKind>, CompileError> {
        if let Some(struct_token) = self.consume_keyword("struct") {
            let mut span = struct_token.span;
            let struct_name = if let Some((name, ident_token)) = self.consume_ident() {
                span = ident_token.span;
                name
            } else {
                String::new()
            };
            if self.consume_punct("{").is_some() {
                let members = self.struct_decl_list()?;
                self.expect_punct("}")?;
                let struct_ty = Type::from(
                    TypeKind::Struct {
                        name: struct_name.clone(),
                        members,
                    },
                    false,
                );
                // 構造体タグを登録
                if !struct_name.is_empty() {
                    if let Some(func) = self.current_func.as_mut() {
                        func.register_struct_tag(struct_name, struct_ty.clone(), span)?;
                    } else {
                        self.register_struct_tag(struct_name, struct_ty.clone(), span)?;
                    }
                }
                return Ok(Some(struct_ty.kind));
            } else if !struct_name.is_empty() {
                // 既存の構造体タグを検索
                // 現在の関数のスコープ内を優先して検索（無い場合はグローバルスコープを検索）
                let struct_ty = self
                    .current_func
                    .as_ref()
                    .and_then(|f| f.find_struct_tag(&struct_name))
                    .or_else(|| self.find_struct_tag(&struct_name));
                if let Some(ty) = struct_ty {
                    return Ok(Some(ty.kind.clone()));
                } else {
                    let span = self.get_prev_token_span().unwrap_or((0, 0));
                    return Err(CompileError::InvalidDecl {
                        msg: format!("未宣言の構造体タグ: '{}'", struct_name),
                        span,
                    });
                }
            } else {
                let span = self.get_prev_token_span().unwrap_or((0, 0));
                return Err(CompileError::InvalidDecl {
                    msg: "無名構造体には定義が必要です".to_string(),
                    span,
                });
            }
        }
        Ok(None)
    }

    // struct_decl_list ::= struct_decl+
    fn struct_decl_list(&mut self) -> Result<Vec<MemberDecl>, CompileError> {
        let mut members: Vec<MemberDecl> = Vec::new();
        while let Some(member_list) = self.struct_decl()? {
            members.extend(member_list);
        }
        Ok(members)
    }

    // struct_decl ::= spec_qual_list struct_declarator_list? ";"
    fn struct_decl(&mut self) -> Result<Option<Vec<MemberDecl>>, CompileError> {
        let specs = self.spec_qual_list()?;
        if specs.is_empty() {
            return Ok(None);
        }
        let base_ty = Type::from_tsq(specs).ok_or_else(|| {
            let span = self.get_prev_token_span().unwrap_or((0, 0));
            CompileError::InvalidDecl {
                msg: "無効な型指定子です".to_string(),
                span,
            }
        })?;
        let members = self.struct_declarator_list(&base_ty)?;
        self.expect_punct(";")?;
        if members.is_empty() {
            return Ok(None);
        }
        Ok(Some(members))
    }

    // struct_declarator_list ::= struct_declarator ("," struct_declarator)*
    fn struct_declarator_list(&mut self, base_ty: &Type) -> Result<Vec<MemberDecl>, CompileError> {
        let mut members = Vec::new();
        if let Some(member) = self.struct_declarator(base_ty)? {
            members.push(member);
        }
        while self.consume_punct(",").is_some() {
            if let Some(member) = self.struct_declarator(base_ty)? {
                members.push(member);
            }
        }
        Ok(members)
    }

    // struct_declarator ::= declarator
    fn struct_declarator(&mut self, base_ty: &Type) -> Result<Option<MemberDecl>, CompileError> {
        if let Ok(decl) = self.declarator(base_ty) {
            return Ok(Some(decl.into()));
        }
        Ok(None)
    }

    // spec_qual_list ::= type_spec_qual+
    fn spec_qual_list(&mut self) -> Result<Vec<TypeSpecQual>, CompileError> {
        let mut specs = Vec::new();
        while let Some(spec) = self.type_spec_qual()? {
            specs.push(spec);
        }
        Ok(specs)
    }

    // type_spec_qual ::= type_spec | type_qual
    fn type_spec_qual(&mut self) -> Result<Option<TypeSpecQual>, CompileError> {
        if let Some(spec) = self.type_spec()? {
            return Ok(Some(TypeSpecQual::TypeSpec(spec)));
        }
        if let Some(qual) = self.type_qual() {
            return Ok(Some(TypeSpecQual::TypeQual(qual)));
        }
        Ok(None)
    }

    // type_qual ::= "const" | "volatile" | "restrict"
    fn type_qual(&mut self) -> Option<TypeQualKind> {
        TypeQualKind::all()
            .into_iter()
            .find(|qual| self.consume_keyword(&qual.to_string()).is_some())
    }

    // func_spec ::= "inline"
    fn func_spec(&mut self) -> Option<FuncKind> {
        FuncKind::all()
            .into_iter()
            .find(|spec| self.consume_keyword(&spec.to_string()).is_some())
    }

    // type_qual_list ::= type_qual*
    fn type_qual_list(&mut self) -> Vec<TypeQualKind> {
        let mut quals = Vec::new();
        while let Some(qual) = self.type_qual() {
            quals.push(qual);
        }
        quals
    }

    // ptr ::= "*" type_qual_list* ptr?
    #[allow(clippy::never_loop)]
    fn ptr(&mut self, base_ty: &Type) -> Type {
        while self.consume_punct("*").is_some() {
            let ptr_type = Type::from(
                TypeKind::Ptr {
                    to: Box::new(base_ty.clone()),
                },
                false,
            );
            return self.ptr(&ptr_type);
        }
        self.type_qual_list(); // 現状は型修飾子を無視
        base_ty.clone()
    }

    // declarator ::= ptr? direct_declarator
    pub(super) fn declarator(&mut self, base_ty: &Type) -> Result<Decl, CompileError> {
        let ty = self.ptr(base_ty);
        self.direct_declarator(&ty)
    }

    // direct_declarator ::= "(" declarator ")"
    //                     | ident
    //                     | direct_declarator "[" type_qual_list? assign_expr? "]"
    //                     | direct_declarator "(" param_type_list ")"
    fn direct_declarator(&mut self, base_ty: &Type) -> Result<Decl, CompileError> {
        let span;
        let name = if let Some(token) = self.consume_punct("(") {
            span = token.span;
            let inner_var = self.declarator(base_ty)?;
            self.expect_punct(")")?;
            inner_var.name
        } else if let Some((name, token)) = self.consume_ident() {
            span = token.span;
            name
        } else {
            let span = self.get_prev_token_span().unwrap_or((0, 0));
            return Err(CompileError::InvalidDecl {
                msg: "識別子または括弧で囲まれた宣言子が必要です".to_string(),
                span,
            });
        };

        let final_ty = self.parse_postfix_declarators(base_ty)?;
        Ok(Decl {
            name,
            ty: final_ty,
            init: Vec::new(),
            span,
        })
    }

    // 右結合で解析
    fn parse_postfix_declarators(&mut self, base_ty: &Type) -> Result<Type, CompileError> {
        // "[" type_qual_list? assign_expr? "]"
        if self.consume_punct("[").is_some() {
            self.type_qual_list(); // 現状は型修飾子を無視
            let array_size = if self.peek_punct("]") {
                0
            } else {
                self.assign_expr()?
                    .ok_or_else(|| {
                        let span = self.get_prev_token_span().unwrap_or((0, 0));
                        CompileError::InvalidDecl {
                            msg: "配列のサイズが必要です".to_string(),
                            span,
                        }
                    })?
                    .eval_const_expr()? as usize
            };
            self.expect_punct("]")?;
            let inner_ty = self.parse_postfix_declarators(base_ty)?;
            Ok(Type::from(
                TypeKind::Array {
                    base: Box::new(inner_ty),
                    size: array_size,
                },
                false,
            ))
        }
        // "(" param_type_list ")"
        else if self.consume_punct("(").is_some() {
            let params = if self.peek_punct(")") {
                // パラメータが0個の場合
                self.expect_punct(")")?;
                Vec::new()
            } else {
                // パラメータが1個以上の場合
                let params = self.param_type_list()?;
                self.expect_punct(")")?;
                params
            };
            let inner_ty = self.parse_postfix_declarators(base_ty)?;
            Ok(Type::from(
                TypeKind::Func {
                    return_ty: Box::new(inner_ty),
                    params,
                },
                false,
            ))
        } else {
            Ok(base_ty.clone())
        }
    }

    // param_type_list ::= param_list
    fn param_type_list(&mut self) -> Result<Vec<Decl>, CompileError> {
        self.param_list()
    }

    // param_list ::= param_decl ("," param_decl)*
    fn param_list(&mut self) -> Result<Vec<Decl>, CompileError> {
        let mut params = Vec::new();
        let param = self.param_decl()?;
        params.push(param);
        while self.consume_punct(",").is_some() {
            let param = self.param_decl()?;
            params.push(param);
        }
        Ok(params)
    }

    // param_decl ::= decl_specs declarator
    fn param_decl(&mut self) -> Result<Decl, CompileError> {
        let specs = self.decl_specs()?;
        if !specs.is_empty() {
            let base_kind = Type::from_ds(specs).ok_or_else(|| {
                let span = self.get_prev_token_span().unwrap_or((0, 0));
                CompileError::InvalidDecl {
                    msg: "無効な型指定子です".to_string(),
                    span,
                }
            })?;
            if let Ok(decl) = self.declarator(&base_kind) {
                return Ok(decl);
            }
        }
        let span = self.get_prev_token_span().unwrap_or((0, 0));
        Err(CompileError::InvalidDecl {
            msg: "無効なパラメータ宣言です".to_string(),
            span,
        })
    }

    // type_name ::= spec_qual_list abst_declarator?
    pub(super) fn type_name(&mut self) -> Result<Type, CompileError> {
        let specs = self.spec_qual_list()?;
        if specs.is_empty() {
            let span = self.get_prev_token_span().unwrap_or((0, 0));
            return Err(CompileError::InvalidDecl {
                msg: "無効な型名です".to_string(),
                span,
            });
        }
        let base_ty = Type::from_tsq(specs).ok_or_else(|| {
            let span = self.get_prev_token_span().unwrap_or((0, 0));
            CompileError::InvalidDecl {
                msg: "無効な型指定子です".to_string(),
                span,
            }
        })?;
        if let Ok(abst_ty) = self.abst_declarator(&base_ty) {
            return Ok(abst_ty);
        }
        Ok(base_ty)
    }

    // abst_declarator ::= ptr // TODO: 未実装
    //                   | ptr? direct_abst_declarator
    fn abst_declarator(&mut self, base_ty: &Type) -> Result<Type, CompileError> {
        let ty = self.ptr(base_ty);
        self.direct_abst_declarator(&ty)
    }

    // direct_abst_declarator ::= "(" abst_declarator ")"
    //                          | direct_abst_declarator "[" type_qual_list? assign_expr? "]"
    //                          | direct_abst_declarator "(" param_type_list ")"
    fn direct_abst_declarator(&mut self, base_ty: &Type) -> Result<Type, CompileError> {
        let mut current_ty = if self.consume_punct("(").is_some() {
            let inner_ty = self.abst_declarator(base_ty)?;
            self.expect_punct(")")?;
            inner_ty
        } else {
            base_ty.clone()
        };
        current_ty = self.parse_abst_postfix_declarators(&current_ty)?;
        Ok(current_ty)
    }

    // 右結合で解析
    fn parse_abst_postfix_declarators(&mut self, base_ty: &Type) -> Result<Type, CompileError> {
        // "[" type_qual_list? assign_expr? "]"
        if self.consume_punct("[").is_some() {
            self.type_qual_list(); // 現状は型修飾子を無視
            let array_size = if self.peek_punct("]") {
                0
            } else {
                self.assign_expr()?
                    .ok_or_else(|| {
                        let span = self.get_prev_token_span().unwrap_or((0, 0));
                        CompileError::InvalidDecl {
                            msg: "配列のサイズが必要です".to_string(),
                            span,
                        }
                    })?
                    .eval_const_expr()? as usize
            };
            self.expect_punct("]")?;
            let inner_ty = self.parse_abst_postfix_declarators(base_ty)?;
            Ok(Type::from(
                TypeKind::Array {
                    base: Box::new(inner_ty),
                    size: array_size,
                },
                false,
            ))
        }
        // "(" param_type_list ")"
        else if self.consume_punct("(").is_some() {
            let params = if self.peek_punct(")") {
                // パラメータが0個の場合
                self.expect_punct(")")?;
                Vec::new()
            } else {
                // パラメータが1個以上の場合
                let params = self.param_type_list()?;
                self.expect_punct(")")?;
                params
            };
            let inner_ty = self.parse_abst_postfix_declarators(base_ty)?;
            Ok(Type::from(
                TypeKind::Func {
                    return_ty: Box::new(inner_ty),
                    params,
                },
                false,
            ))
        } else {
            Ok(base_ty.clone())
        }
    }

    // initializer ::= assign_expr
    //               | "{" initializer_list "}"
    //               | "{" initializer_list "," "}" // TODO: 未対応（initializer_listの処理と重複して問題が発生）
    pub(super) fn initializer(&mut self) -> Result<Vec<Node>, CompileError> {
        if self.consume_punct("{").is_some() {
            let init_list = self.initializer_list()?;
            self.expect_punct("}")?;
            return Ok(init_list);
        }
        Ok(vec![*self.assign_expr()?.ok_or_else(|| {
            let span = self.get_prev_token_span().unwrap_or((0, 0));
            CompileError::InvalidDecl {
                msg: "初期化式が必要です。'= 定数式' の形式で初期値を指定してください".to_string(),
                span,
            }
        })?])
    }

    // initializer_list ::= initializer ("," initializer)*
    fn initializer_list(&mut self) -> Result<Vec<Node>, CompileError> {
        let mut init_list = Vec::new();
        init_list.extend(self.initializer()?);
        while self.consume_punct(",").is_some() {
            init_list.extend(self.initializer()?);
        }
        Ok(init_list)
    }
}
