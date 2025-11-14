use crate::token::Token;
use crate::token::{
    RESERVED_SYMBOLS, RESERVED_WORDS, STORAGE_CLASSES, STRUCT_OR_UNION, TYPE_QUALIFIERS, TYPES,
};
use crate::types::TypeKind;

pub struct Tokenizer {}

impl Tokenizer {
    pub fn new() -> Self {
        Tokenizer {}
    }

    pub fn tokenize(&self, input: &str) -> Result<Vec<Token>, String> {
        // 演算子トークンを長い順にソート
        let mut sorted_reserved_symbols = RESERVED_SYMBOLS.to_vec();
        sorted_reserved_symbols.sort_by(|a, b| b.len().cmp(&a.len()));

        let mut tokens = Vec::new();
        let chars = input.chars().collect::<Vec<char>>();
        let mut pos = 0;

        while pos < chars.len() {
            let c = chars[pos];

            // 空白文字をスキップ
            if matches!(c, ' ' | '\t' | '\n' | '\r') {
                pos += 1;
                continue;
            }

            // 行コメントをスキップ
            if c == '/' && pos + 1 < chars.len() && chars[pos + 1] == '/' {
                pos += 2;
                while pos < chars.len() && chars[pos] != '\n' {
                    pos += 1;
                }
                pos += 1;
                continue;
            }

            // ブロックコメントをスキップ
            if c == '/' && pos + 1 < chars.len() && chars[pos + 1] == '*' {
                pos += 2;
                while pos + 1 < chars.len() {
                    if chars[pos] == '*' && chars[pos + 1] == '/' {
                        pos += 2;
                        break;
                    }
                    pos += 1;
                }
                if pos == chars.len() - 1 {
                    return Err("ブロックコメントが閉じられていません".to_string());
                }
                continue;
            }

            // 演算子トークン
            let mut matched = false;
            for symbol in &sorted_reserved_symbols {
                let symbol_len = symbol.len();
                if pos + symbol_len <= chars.len() {
                    let candidate: String = chars[pos..pos + symbol_len].iter().collect();
                    if candidate == *symbol {
                        tokens.push(Token::Symbol(symbol.to_string()));
                        pos += symbol_len;
                        matched = true;
                        break;
                    }
                }
            }
            if matched {
                continue;
            }

            // 数字トークン
            if matches!(c, '0'..='9') {
                let mut num_str = String::new();
                num_str.push(c);
                pos += 1;
                while pos < chars.len() {
                    let next_c = chars[pos];
                    if matches!(next_c, '0'..='9') {
                        num_str.push(next_c);
                        pos += 1;
                    } else {
                        break;
                    }
                }
                let val = num_str.parse::<i64>().unwrap();
                tokens.push(Token::Num(val));
                continue;
            }

            // 識別子トークン
            if matches!(c, 'a'..='z' | 'A'..='Z') {
                let mut ident = c.to_string();
                pos += 1;
                while pos < chars.len() {
                    let next_c = chars[pos];
                    if matches!(next_c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_') {
                        ident.push(next_c);
                        pos += 1;
                    } else {
                        break;
                    }
                }
                if TYPES.contains(&ident.as_str()) {
                    // 型はTypeトークンとして扱う
                    let type_kind = match ident.as_str() {
                        "short" => TypeKind::Short,
                        "int" => TypeKind::Int,
                        "long" => TypeKind::Long,
                        _ => return Err(format!("未対応の型です: {}", ident)),
                    };
                    tokens.push(Token::Type(type_kind));
                    continue;
                } else if STRUCT_OR_UNION.contains(&ident.as_str()) {
                    // structまたはunionはStructOrUnionトークンとして扱う
                    tokens.push(Token::StructOrUnion(ident));
                    continue;
                } else if ident == "enum" {
                    // enumはEnumトークンとして扱う
                    tokens.push(Token::Enum);
                    continue;
                } else if TYPE_QUALIFIERS.contains(&ident.as_str()) {
                    // 型修飾子はTypeQualifierトークンとして扱う
                    tokens.push(Token::TypeQualifier(ident));
                    continue;
                } else if STORAGE_CLASSES.contains(&ident.as_str()) {
                    // 記憶クラスはStorageClassトークンとして扱う
                    tokens.push(Token::StorageClass(ident));
                    continue;
                } else if RESERVED_WORDS.contains(&ident.as_str()) {
                    // 予約語はReservedトークンとして扱う
                    tokens.push(Token::Reserved(ident));
                    continue;
                } else {
                    // それ以外は識別子トークン
                    tokens.push(Token::Ident(ident));
                    continue;
                }
            }
            return Err(format!("不明な文字が含まれています: {}", c));
        }
        tokens.push(Token::EOF);
        Ok(tokens)
    }
}
