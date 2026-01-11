use super::Ast;
use crate::decl::{Decl, MemberDecl};
use crate::errors::CompileError;
use crate::func::{Func, LocalVar};
use crate::node::{Node, NodeKind};
use crate::types::{
    DeclSpec, FuncKind, StorageClassKind, TypeAttr, TypeKind, TypeQualKind, TypeRef, TypeSpecQual,
};
use crate::utils::Span;

impl Ast<'_> {
    // external_decl ::= func_def | decl
    // func_def      ::= decl_specs declarator compound_stmt
    // decl          ::= decl_specs init_declarator_list? ";"
    pub(super) fn external_decl(&mut self) -> Result<(), CompileError> {
        let decl_specs = self.decl_specs()?;
        if decl_specs.is_empty() {
            return Err(CompileError::InvalidDecl {
                msg: "外部宣言のパースに失敗しました。型指定子が必要です".to_string(),
                span: self.current_span(),
            });
        }

        let base_ty = TypeRef::from_ds(&decl_specs).ok_or_else(|| CompileError::InvalidDecl {
            msg: format!("無効な型指定子です: {:?}", decl_specs),
            span: self.current_span(),
        })?;

        let token_pos = self.token_pos; // 関数定義でなかった場合にバックトラックするために保存
        if let Ok(decl) = self.declarator(&base_ty) {
            // 関数定義または関数プロトタイプ宣言の場合
            if let TypeKind::Func { .. } = &decl.ty.kind() {
                if self.peek_punct("{") {
                    return self.func_def(decl); // 関数定義をパース
                } else if self.consume_punct(";").is_some() {
                    self.register_func_symbol(&decl.name, decl.ty, false); // プロトタイプ宣言を登録
                    return Ok(());
                } else {
                    return Err(CompileError::InvalidDecl {
                        msg: "関数の本体またはセミコロンが必要です".to_string(),
                        span: decl.span,
                    });
                }
            }
        }
        self.token_pos = token_pos; // バックトラックして再度パース

        // グローバル変数宣言の場合: init_declarator_list? ";"
        let decls = self.init_declarator_list(&base_ty)?;
        self.expect_punct(";")?;

        // グローバル変数として登録
        for decl in decls {
            if decl.ty.is_typedef() {
                self.register_typedef(&decl.name, decl.ty, decl.span)?;
            } else {
                self.register_var(&decl, None)?;
            }
        }
        Ok(())
    }

    // 関数定義をパース
    fn func_def(&mut self, decl: Decl) -> Result<(), CompileError> {
        let TypeKind::Func { params, return_ty, .. } = &decl.ty.kind() else {
            return Err(CompileError::InvalidDecl {
                msg: "関数型が必要です".to_string(),
                span: decl.span,
            });
        };

        // プロトタイプ宣言を検証
        self.validate_func_prototype(&decl)?;

        // 関数シンボルを登録または更新
        if let Some(symbol) = self.find_symbol_mut(&decl.name) {
            symbol.set_defined(true);
        } else {
            self.register_func_symbol(&decl.name, decl.ty, true);
        }

        // 関数本体をパース
        let func_id = self.register_func_def(Func::new(&decl.name));
        self.current_func = Some(func_id);
        self.push_scope(); // 引数スコープに入る

        // パラメータを登録
        for param_decl in params {
            let symbol_id = self.register_var(param_decl, Some(func_id))?;
            self.get_current_func_mut()?.params.push(LocalVar::new(symbol_id));
        }

        // 戻り値の型を設定
        self.get_current_func_mut()?.return_ty = *return_ty;

        // 関数本体をパース
        let func_body = self.compound_stmt()?.ok_or_else(|| CompileError::InvalidDecl {
            msg: "関数本体が必要です".to_string(),
            span: self.current_span(),
        })?;

        let NodeKind::Block { body } = func_body.kind else {
            return Err(CompileError::InvalidDecl {
                msg: "関数本体がブロックではありません。'{' と '}' で囲まれた複合文が必要です"
                    .to_string(),
                span: func_body.span,
            });
        };

        self.get_current_func_mut()?.body = body;
        self.pop_scope(); // 引数スコープを抜ける
        self.calc_current_func_offset()?; // ローカル変数のオフセットを計算
        self.current_func = None;

        Ok(())
    }

    // プロトタイプ宣言を検証
    fn validate_func_prototype(&mut self, decl: &Decl) -> Result<(), CompileError> {
        if let Some(symbol) = self.find_symbol_mut(&decl.name) {
            if !symbol.is_func() {
                return Err(CompileError::InvalidDecl {
                    msg: format!("'{}' は関数ではありません", decl.name),
                    span: decl.span,
                });
            }
            if symbol.is_defined() {
                return Err(CompileError::InvalidDecl {
                    msg: format!("関数 '{}' が既に定義されています", decl.name),
                    span: decl.span,
                });
            }
            // TODO: 抽象宣言子を考慮して型チェックを実装
            // プロトタイプ宣言と定義の型が一致するか確認
        }
        Ok(())
    }

    // decl ::= decl_specs init_declarator_list? ";"
    pub(super) fn decl(&mut self) -> Result<Option<Vec<Decl>>, CompileError> {
        let decl_specs = self.decl_specs()?;
        if decl_specs.is_empty() {
            return Ok(None);
        }
        let base_ty = TypeRef::from_ds(&decl_specs).ok_or_else(|| CompileError::InvalidDecl {
            msg: format!("無効な型指定子です: {:?}", decl_specs),
            span: self.current_span(),
        })?;
        let decls = self.init_declarator_list(&base_ty)?;
        self.expect_punct(";")?;
        Ok(Some(decls))
    }

    // decl_specs ::= decl_spec+
    pub(super) fn decl_specs(&mut self) -> Result<Vec<DeclSpec>, CompileError> {
        let mut decl_specs = Vec::new();
        while let Some(decl_spec) = self.decl_spec()? {
            decl_specs.push(decl_spec);
        }
        Ok(decl_specs)
    }

    // decl_spec ::= storage_class_spec | type_spec | type_qual | func_spec
    pub(super) fn decl_spec(&mut self) -> Result<Option<DeclSpec>, CompileError> {
        if let Some(storage_class_spec) = self.storage_class_spec() {
            return Ok(Some(DeclSpec::StorageClassSpec(storage_class_spec)));
        }
        if let Some(type_spec) = self.type_spec()? {
            return Ok(Some(DeclSpec::TypeSpec(type_spec)));
        }
        if let Some(type_qual) = self.type_qual() {
            return Ok(Some(DeclSpec::TypeQual(type_qual)));
        }
        if let Some(func_spec) = self.func_spec() {
            return Ok(Some(DeclSpec::FuncSpec(func_spec)));
        }
        Ok(None)
    }

    // init_declarator_list ::= init_declarator ("," init_declarator)*
    fn init_declarator_list(&mut self, base_ty: &TypeRef) -> Result<Vec<Decl>, CompileError> {
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
    fn init_declarator(&mut self, base_ty: &TypeRef) -> Result<Option<Decl>, CompileError> {
        if let Ok(mut decl) = self.declarator(base_ty) {
            if self.consume_punct("=").is_some() {
                // TODO: 代入時の型チェック
                decl.init = self.initializer()?; // initializerを設定

                if decl.ty.is_incomplete() && decl.ty.is_array() {
                    // サイズ不明な配列型の場合、初期化子の要素数でサイズを決定
                    let len = decl.init.len();
                    decl.ty.complete_array(len);
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

    // type_spec ::= "void" | "char" | "short" | "int" | "long" | "float" | "double" | struct_or_union_spec | enum_spec | typedef_name
    fn type_spec(&mut self) -> Result<Option<TypeRef>, CompileError> {
        if let Some(ty_kind) = self.struct_or_union_spec()? {
            return Ok(Some(TypeRef::register(ty_kind, TypeAttr::default(), None)));
        }
        if let Some(ty_kind) = self.enum_spec()? {
            return Ok(Some(TypeRef::register(ty_kind, TypeAttr::default(), None)));
        }
        if let Some(typedef_name) = self.peek_ident()
            && let Some(ty) = self.find_typedef(&typedef_name).map(|symbol| symbol.ty)
        {
            self.consume_ident(); // トークンを消費
            return Ok(Some(ty));
        }
        Ok(TypeKind::all()
            .into_iter()
            .find(|spec| self.consume_keyword(&spec.to_string()).is_some())
            .map(|ty_kind| TypeRef::register(ty_kind, TypeAttr::default(), None)))
    }

    // struct_or_union_spec ::= ("struct" | "union") ident? "{" struct_decl_list "}"
    //                        | ("struct" | "union") ident
    fn struct_or_union_spec(&mut self) -> Result<Option<TypeKind>, CompileError> {
        // struct
        if let Some(span) = self.consume_keyword("struct") {
            let struct_name =
                if let Some((name, _)) = self.consume_ident() { name } else { String::new() };
            // 構造体定義: struct ident? { ... }
            if self.peek_punct("{") {
                return Ok(Some(self.parse_struct_or_union_definition(struct_name, span, true)?));
            }

            // 構造体参照または前方宣言: struct ident
            if !struct_name.is_empty() {
                return Ok(Some(self.parse_struct_or_union_reference(struct_name, true)));
            }

            // 無名構造体の前方宣言はエラー
            return Err(CompileError::InvalidDecl {
                msg: "無名構造体には定義が必要です".to_string(),
                span: self.current_span(),
            });
        }

        // union
        if let Some(span) = self.consume_keyword("union") {
            let union_name =
                if let Some((name, _)) = self.consume_ident() { name } else { String::new() };
            // 共用体定義: union ident? { ... }
            if self.peek_punct("{") {
                return Ok(Some(self.parse_struct_or_union_definition(union_name, span, false)?));
            }

            // 共用体参照または前方宣言: union ident
            if !union_name.is_empty() {
                return Ok(Some(self.parse_struct_or_union_reference(union_name, false)));
            }

            // 無名共用体の前方宣言はエラー
            return Err(CompileError::InvalidDecl {
                msg: "無名共用体には定義が必要です".to_string(),
                span: self.current_span(),
            });
        }
        Ok(None)
    }

    // 構造体/共用体定義をパース
    fn parse_struct_or_union_definition(
        &mut self,
        name: String,
        span: Span,
        is_struct: bool,
    ) -> Result<TypeKind, CompileError> {
        // メンバをパースする前に未完成型を登録
        let incomplete_ty = TypeRef::register(
            if is_struct {
                TypeKind::Struct { name: name.clone(), members: Vec::new() }
            } else {
                TypeKind::Union { name: name.clone(), members: Vec::new() }
            },
            TypeAttr::default(),
            None,
        );

        // タグの重複チェックと登録
        if !name.is_empty() {
            self.validate_and_register_tag(&name, incomplete_ty, span)?;
        }

        // メンバをパース
        self.expect_punct("{")?;
        let members = self.struct_decl_list()?;
        self.expect_punct("}")?;

        if members.is_empty() {
            return Err(CompileError::InvalidDecl {
                msg: "構造体/共用体には少なくとも1つのメンバが必要です".to_string(),
                span,
            });
        }

        // 型を完成させる
        let completed_ty = if incomplete_ty.is_incomplete() {
            incomplete_ty.complete_struct_or_union(members)
        } else {
            // 入れ子になっている場合
            TypeRef::register(
                if is_struct {
                    TypeKind::Struct { name: name.clone(), members }
                } else {
                    TypeKind::Union { name: name.clone(), members }
                },
                TypeAttr::default(),
                None,
            )
        };

        // タグを更新
        if !name.is_empty() {
            self.register_tag(&name, completed_ty);
        }

        Ok(completed_ty.kind())
    }

    // 構造体参照または前方宣言をパース
    fn parse_struct_or_union_reference(&mut self, name: String, is_struct: bool) -> TypeKind {
        // 既存のタグを検索
        if let Some(tag) = self.find_tag(&name) {
            return tag.ty.kind();
        }

        // 前方宣言として未完成型を登録
        let incomplete_ty = TypeRef::register(
            if is_struct {
                TypeKind::Struct { name: name.clone(), members: Vec::new() }
            } else {
                TypeKind::Union { name: name.clone(), members: Vec::new() }
            },
            TypeAttr::default(),
            None,
        );
        self.register_tag(&name, incomplete_ty);
        incomplete_ty.kind()
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
        let type_spec_quals = self.spec_qual_list()?;
        if type_spec_quals.is_empty() {
            return Ok(None);
        }
        let base_ty =
            TypeRef::from_tsq(&type_spec_quals).ok_or_else(|| CompileError::InvalidDecl {
                msg: format!("無効な型指定子です: {:?}", type_spec_quals),
                span: self.current_span(),
            })?;
        let members = self.struct_declarator_list(&base_ty)?;
        self.expect_punct(";")?;
        Ok(Some(members))
    }

    // struct_declarator_list ::= struct_declarator ("," struct_declarator)*
    fn struct_declarator_list(
        &mut self,
        base_ty: &TypeRef,
    ) -> Result<Vec<MemberDecl>, CompileError> {
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
    fn struct_declarator(&mut self, base_ty: &TypeRef) -> Result<Option<MemberDecl>, CompileError> {
        if let Ok(decl) = self.declarator(base_ty) {
            return Ok(Some(decl.into()));
        }
        Ok(None)
    }

    // enum_spec ::= "enum" ident? "{" enum_list "}"
    //             | "enum" ident? "{" enum_list "," "}"
    //             | "enum" ident
    fn enum_spec(&mut self) -> Result<Option<TypeKind>, CompileError> {
        if let Some(span) = self.consume_keyword("enum") {
            let enum_name =
                if let Some((name, _)) = self.consume_ident() { name } else { String::new() };
            // 列挙体定義: enum ident? { ... }
            if self.peek_punct("{") {
                return Ok(Some(self.parse_enum_definition(enum_name, span)?));
            }

            // 列挙体参照: enum ident
            if !enum_name.is_empty() {
                return Ok(Some(self.parse_enum_reference(enum_name, span)?));
            }

            // 無名列挙体の前方宣言はエラー
            return Err(CompileError::InvalidDecl {
                msg: "無名列挙体には定義が必要です".to_string(),
                span: self.current_span(),
            });
        }
        Ok(None)
    }

    // 列挙体定義をパース
    fn parse_enum_definition(
        &mut self,
        enum_name: String,
        span: Span,
    ) -> Result<TypeKind, CompileError> {
        // タグの重複チェックと登録
        let int_ty = TypeRef::register(TypeKind::Int, TypeAttr::default(), None);
        if !enum_name.is_empty() {
            self.validate_and_register_tag(&enum_name, int_ty, span)?;
        }

        // メンバをパース
        self.expect_punct("{")?;
        let variants = self.enum_list()?;
        self.expect_punct("}")?;

        // 列挙定数をシンボルとして登録
        for (name, val) in &variants {
            self.register_enum_const_symbol(name, *val, span)?;
        }

        Ok(TypeKind::Int) // 列挙体の型は int として扱う
    }

    // 列挙体定義または参照をパース
    fn parse_enum_reference(
        &mut self,
        enum_name: String,
        span: Span,
    ) -> Result<TypeKind, CompileError> {
        // 既存の列挙体タグを検索
        if let Some(tag) = self.find_tag(&enum_name) {
            Ok(tag.ty.kind())
        } else {
            Err(CompileError::InvalidDecl {
                msg: format!("列挙体 '{}' が見つかりません", enum_name),
                span,
            })
        }
    }

    // enum_list ::= enumerator ( "," enumerator )*
    fn enum_list(&mut self) -> Result<Vec<(String, i64)>, CompileError> {
        let mut variants_with_opt = Vec::new();
        let variant = self.enumerator()?.ok_or_else(|| CompileError::InvalidDecl {
            msg: "列挙体には少なくとも1つの列挙定数が必要です".to_string(),
            span: self.current_span(),
        })?;
        variants_with_opt.push(variant);
        while self.consume_punct(",").is_some() {
            if self.peek_punct("}") {
                // 末尾カンマに対応
                break;
            }
            if let Some(variant) = self.enumerator()? {
                variants_with_opt.push(variant);
            } else {
                // カンマの後に識別子がない場合はエラー
                return Err(CompileError::InvalidDecl {
                    msg: "カンマの後に識別子が必要です".to_string(),
                    span: self.current_span(),
                });
            }
        }

        // 値の割り当て
        let mut variants = Vec::new();
        let mut current_val: i64 = 0;
        for (name, value_opt) in variants_with_opt {
            let value = if let Some(v) = value_opt {
                current_val = v;
                v
            } else {
                current_val
            };
            variants.push((name, value));
            current_val += 1;
        }
        Ok(variants)
    }

    // enumerator ::= ident ("=" const_expr)?
    fn enumerator(&mut self) -> Result<Option<(String, Option<i64>)>, CompileError> {
        if let Some((name, _)) = self.consume_ident() {
            let value =
                if self.consume_punct("=").is_some() { Some(self.const_expr()?) } else { None };
            return Ok(Some((name, value)));
        }
        Ok(None)
    }

    // spec_qual_list ::= type_spec_qual+
    fn spec_qual_list(&mut self) -> Result<Vec<TypeSpecQual>, CompileError> {
        let mut type_spec_quals = Vec::new();
        while let Some(type_spec_qual) = self.type_spec_qual()? {
            type_spec_quals.push(type_spec_qual);
        }
        Ok(type_spec_quals)
    }

    // type_spec_qual ::= type_spec | type_qual
    fn type_spec_qual(&mut self) -> Result<Option<TypeSpecQual>, CompileError> {
        if let Some(spec) = self.type_spec()? {
            return Ok(Some(TypeSpecQual::TypeSpec(spec)));
        }
        if let Some(type_qual) = self.type_qual() {
            return Ok(Some(TypeSpecQual::TypeQual(type_qual)));
        }
        Ok(None)
    }

    // type_qual ::= "const" | "volatile" | "restrict"
    fn type_qual(&mut self) -> Option<TypeQualKind> {
        TypeQualKind::all()
            .into_iter()
            .find(|type_qual| self.consume_keyword(&type_qual.to_string()).is_some())
    }

    // func_spec ::= "inline"
    fn func_spec(&mut self) -> Option<FuncKind> {
        FuncKind::all().into_iter().find(|spec| self.consume_keyword(&spec.to_string()).is_some())
    }

    // type_qual_list ::= type_qual*
    fn type_qual_list(&mut self) -> Vec<TypeQualKind> {
        let mut type_quals = Vec::new();
        while let Some(type_qual) = self.type_qual() {
            type_quals.push(type_qual);
        }
        type_quals
    }

    // ptr ::= "*" type_qual_list? ptr?
    fn ptr(&mut self, base_ty: &TypeRef) -> TypeRef {
        let mut current_ty = *base_ty;
        while self.consume_punct("*").is_some() {
            let type_quals = self.type_qual_list();
            current_ty = TypeRef::register(
                TypeKind::Ptr { to: TypeRef::register(current_ty.kind(), current_ty.attr(), None) },
                TypeAttr::from_type_quals(&type_quals),
                current_ty.storage_class(),
            );
        }
        current_ty
    }

    // declarator ::= ptr? direct_declarator
    pub(super) fn declarator(&mut self, base_ty: &TypeRef) -> Result<Decl, CompileError> {
        let ty = self.ptr(base_ty);
        self.direct_declarator(&ty)
    }

    // direct_declarator ::= "(" declarator ")"
    //                     | ident
    //                     | direct_declarator "[" type_qual_list? assign_expr? "]"
    //                     | direct_declarator "(" param_type_list ")"
    fn direct_declarator(&mut self, base_ty: &TypeRef) -> Result<Decl, CompileError> {
        if let Some(paren_span) = self.consume_punct("(") {
            // 括弧で囲まれた宣言子の場合、プレースホルダーを使って内側の型を一時的に表現
            let placeholder = TypeRef::default();
            let inner_decl = self.declarator(&placeholder)?;
            self.expect_punct(")")?;
            let suffix_ty = self.direct_declarator_suffix(base_ty)?; // 外側のsuffixを先にパース
            // プレースホルダーをsuffix_tyで置き換え
            let final_ty = inner_decl.ty.replace_type(placeholder, suffix_ty);
            Ok(Decl::new(inner_decl.name, final_ty, paren_span))
        } else if let Some((ident_name, ident_span)) = self.consume_ident() {
            // 識別子の場合は普通にsuffixを適用
            let final_ty = self.direct_declarator_suffix(base_ty)?;
            Ok(Decl::new(ident_name, final_ty, ident_span))
        } else {
            Err(CompileError::InvalidDecl {
                msg: "識別子または括弧で囲まれた宣言子が必要です".to_string(),
                span: self.current_span(),
            })
        }
    }

    // 右結合で解析
    fn direct_declarator_suffix(&mut self, base_ty: &TypeRef) -> Result<TypeRef, CompileError> {
        // "[" type_qual_list? assign_expr? "]"
        if self.consume_punct("[").is_some() {
            let type_quals = self.type_qual_list();
            let array_size = if self.peek_punct("]") {
                0
            } else {
                let assign_expr = self.assign_expr()?.ok_or_else(|| CompileError::InvalidDecl {
                    msg: "配列のサイズが必要です".to_string(),
                    span: self.current_span(),
                })?;
                Self::eval_const_expr(&assign_expr)? as usize
            };
            self.expect_punct("]")?;
            let inner_ty = self.direct_declarator_suffix(base_ty)?;
            let elem_ty = TypeRef::register(inner_ty.kind(), inner_ty.attr(), None); // 要素型のストレージクラスはなし
            Ok(TypeRef::register(
                TypeKind::Array { base: elem_ty, size: array_size },
                TypeAttr::from_type_quals(&type_quals),
                inner_ty.storage_class(),
            ))
        }
        // "(" param_type_list ")"
        else if self.consume_punct("(").is_some() {
            let (params, is_variadic) = if self.peek_punct(")") {
                // パラメータが0個の場合
                self.expect_punct(")")?;
                (Vec::new(), false)
            } else {
                // パラメータが1個以上の場合
                let result = self.param_type_list()?;
                self.expect_punct(")")?;
                result
            };
            let inner_ty = self.direct_declarator_suffix(base_ty)?;
            let return_ty = TypeRef::register(inner_ty.kind(), inner_ty.attr(), None); // 戻り値型にストレージクラスは付けられない
            Ok(TypeRef::register(
                TypeKind::Func { return_ty, params, is_variadic },
                TypeAttr::default(),
                inner_ty.storage_class(),
            ))
        } else {
            Ok(*base_ty)
        }
    }

    // param_type_list ::= param_list
    //                   | param_list "," "..." // param_listで処理
    fn param_type_list(&mut self) -> Result<(Vec<Decl>, bool), CompileError> {
        self.param_list()
    }

    // param_list ::= param_decl ("," param_decl)*
    fn param_list(&mut self) -> Result<(Vec<Decl>, bool), CompileError> {
        let mut params = Vec::new();
        let param = self.param_decl()?;
        params.push(param);
        while self.consume_punct(",").is_some() {
            if self.consume_punct("...").is_some() {
                return Ok((params, true));
            }
            let param = self.param_decl()?;
            params.push(param);
        }
        Ok((params, false))
    }

    // param_decl ::= decl_specs declarator
    //              | decl_specs abstract_declarator?
    fn param_decl(&mut self) -> Result<Decl, CompileError> {
        let decl_specs = self.decl_specs()?;
        if decl_specs.is_empty() {
            return Err(CompileError::InvalidDecl {
                msg: "パラメータ宣言には型指定子が必要です".to_string(),
                span: self.current_span(),
            });
        }

        let base_ty = TypeRef::from_ds(&decl_specs).ok_or_else(|| CompileError::InvalidDecl {
            msg: format!("無効な型指定子です: {:?}", decl_specs),
            span: self.current_span(),
        })?;

        // declarator
        if let Some(decl) = self.attempt(|s| s.declarator(&base_ty)) {
            return Ok(decl);
        }

        // abstract_declarator
        if let Some(abst_ty) = self.attempt(|s| s.abst_declarator(&base_ty)) {
            return Ok(Decl::new_abst(abst_ty, self.current_span()));
        }

        // 両方失敗した場合は、型のみ（int など単純な型）として扱う
        // 次のトークンが "," か ")" なら型のみの宣言として許可
        if self.peek_punct(",") || self.peek_punct(")") {
            return Ok(Decl::new_abst(base_ty, self.current_span()));
        }

        Err(CompileError::InvalidDecl {
            msg: "無効なパラメータ宣言です".to_string(),
            span: self.current_span(),
        })
    }

    // type_name ::= spec_qual_list abst_declarator?
    pub(super) fn type_name(&mut self) -> Result<TypeRef, CompileError> {
        let type_spec_quals = self.spec_qual_list()?;
        if type_spec_quals.is_empty() {
            return Err(CompileError::InvalidDecl {
                msg: "無効な型名です".to_string(),
                span: self.current_span(),
            });
        }
        let base_ty =
            TypeRef::from_tsq(&type_spec_quals).ok_or_else(|| CompileError::InvalidDecl {
                msg: format!("無効な型指定子です: {:?}", type_spec_quals),
                span: self.current_span(),
            })?;
        if let Ok(abst_ty) = self.abst_declarator(&base_ty) {
            return Ok(abst_ty);
        }
        Ok(base_ty)
    }

    // abst_declarator ::= ptr
    //                   | ptr? direct_abst_declarator
    fn abst_declarator(&mut self, base_ty: &TypeRef) -> Result<TypeRef, CompileError> {
        let ty = self.ptr(base_ty);

        // ptrが適用された場合（*がある場合）、direct_abst_declaratorは省略可能
        // 例: int * の場合、*だけでOK
        if ty != *base_ty {
            // ptrが適用された（*があった）場合
            // direct_abst_declaratorを試すが、失敗してもptrの結果を返す
            let result = self.attempt(|s| s.direct_abst_declarator(&ty)).unwrap_or(ty);
            return Ok(result);
        }

        // ptrが適用されなかった場合は、direct_abst_declaratorが必須
        self.direct_abst_declarator(&ty)
    }

    // direct_abst_declarator ::= "(" abst_declarator ")"
    //                          | direct_abst_declarator "[" type_qual_list? assign_expr? "]"
    //                          | direct_abst_declarator "(" param_type_list ")"
    fn direct_abst_declarator(&mut self, base_ty: &TypeRef) -> Result<TypeRef, CompileError> {
        let current_ty = if self.consume_punct("(").is_some() {
            let inner_ty = self.abst_declarator(base_ty)?;
            self.expect_punct(")")?;
            inner_ty
        } else {
            *base_ty
        };
        self.direct_abst_declarator_suffix(&current_ty)
    }

    // 右結合で解析
    fn direct_abst_declarator_suffix(
        &mut self,
        base_ty: &TypeRef,
    ) -> Result<TypeRef, CompileError> {
        // "[" type_qual_list? assign_expr? "]"
        if self.consume_punct("[").is_some() {
            let type_quals = self.type_qual_list();
            let array_size = if self.peek_punct("]") {
                0
            } else {
                let assign_expr = self.assign_expr()?.ok_or_else(|| CompileError::InvalidDecl {
                    msg: "配列のサイズが必要です".to_string(),
                    span: self.current_span(),
                })?;
                Self::eval_const_expr(&assign_expr)? as usize
            };
            self.expect_punct("]")?;
            let inner_ty = self.direct_abst_declarator_suffix(base_ty)?;
            Ok(TypeRef::register(
                TypeKind::Array { base: inner_ty, size: array_size },
                TypeAttr::from_type_quals(&type_quals),
                None,
            ))
        }
        // "(" param_type_list ")"
        else if self.consume_punct("(").is_some() {
            let (params, is_variadic) = if self.peek_punct(")") {
                // パラメータが0個の場合
                self.expect_punct(")")?;
                (Vec::new(), false)
            } else {
                // パラメータが1個以上の場合
                let result = self.param_type_list()?;
                self.expect_punct(")")?;
                result
            };
            let inner_ty = self.direct_abst_declarator_suffix(base_ty)?;
            Ok(TypeRef::register(
                TypeKind::Func { return_ty: inner_ty, params, is_variadic },
                TypeAttr::default(),
                None,
            ))
        } else {
            Ok(*base_ty)
        }
    }

    // initializer ::= assign_expr
    //               | "{" initializer_list "}"
    //               | "{" initializer_list "," "}" // initializer_listで処理
    pub(super) fn initializer(&mut self) -> Result<Vec<Node>, CompileError> {
        if self.consume_punct("{").is_some() {
            let init_list = self.initializer_list()?;
            self.expect_punct("}")?;
            return Ok(init_list);
        }
        Ok(vec![*self.assign_expr()?.ok_or_else(|| CompileError::InvalidDecl {
            msg: "初期化式が必要です。'= 定数式' の形式で初期値を指定してください".to_string(),
            span: self.current_span(),
        })?])
    }

    // initializer_list ::= initializer ("," initializer)*
    fn initializer_list(&mut self) -> Result<Vec<Node>, CompileError> {
        let mut init_list = Vec::new();
        init_list.extend(self.initializer()?);
        while self.consume_punct(",").is_some() {
            if self.peek_punct("}") {
                // 末尾カンマに対応
                break;
            }
            init_list.extend(self.initializer()?);
        }
        Ok(init_list)
    }
}
