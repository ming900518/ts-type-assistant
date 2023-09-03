use swc_ecma_ast::{
    TsEntityName, TsKeywordTypeKind, TsLit, TsType, TsTypeElement, TsTypeQueryExpr,
    TsUnionOrIntersectionType,
};

use crate::types::{Field, LiteralType, ProcessedType};

pub trait TypeParser {
    fn parser(&self) -> ProcessedType;
}

impl TypeParser for Box<TsType> {
    fn parser(&self) -> ProcessedType {
        match *self.clone() {
            TsType::TsArrayType(array) => ProcessedType::Array(Box::new(array.elem_type.parser())),
            TsType::TsTupleType(tuple) => ProcessedType::Tuple(
                tuple
                    .elem_types
                    .iter()
                    .map(|element| element.ty.parser())
                    .collect(),
            ),
            TsType::TsOptionalType(optional) => {
                ProcessedType::Optional(Box::new(optional.type_ann.parser()))
            }
            TsType::TsUnionOrIntersectionType(TsUnionOrIntersectionType::TsUnionType(union)) => {
                ProcessedType::Union(union.types.iter().map(Self::parser).collect())
            }
            TsType::TsUnionOrIntersectionType(TsUnionOrIntersectionType::TsIntersectionType(
                intersection,
            )) => {
                ProcessedType::Intersection(intersection.types.iter().map(Self::parser).collect())
            }
            TsType::TsTypeRef(type_ref) => ProcessedType::TypeReference(match type_ref.type_name {
                TsEntityName::Ident(ident) => ident.sym.to_string(),
                TsEntityName::TsQualifiedName(qualified) => qualified.right.sym.to_string(),
            }),
            TsType::TsTypeQuery(type_query) => match type_query.expr_name {
                TsTypeQueryExpr::TsEntityName(entity) => {
                    ProcessedType::TypeReference(match entity {
                        TsEntityName::Ident(ident) => ident.sym.to_string(),
                        TsEntityName::TsQualifiedName(qualified) => qualified.right.sym.to_string(),
                    })
                }
                TsTypeQueryExpr::Import(import) => {
                    ProcessedType::Import(import.arg.value.to_string())
                }
            },
            TsType::TsTypeLit(type_literal) => ProcessedType::TypeLiteral(
                type_literal
                    .members
                    .iter()
                    .filter_map(|member| {
                        if let TsTypeElement::TsPropertySignature(property) = member {
                            let is_optional = property.optional;
                            let name = property
                                .clone()
                                .key
                                .ident()
                                .map_or_else(String::default, |ident| ident.sym.to_string());
                            let data_type = property.type_ann.clone().map_or_else(
                                || ProcessedType::Undefined,
                                |ann| ann.type_ann.parser(),
                            );
                            Some(Field {
                                name,
                                data_type,
                                is_optional,
                            })
                        } else {
                            None
                        }
                    })
                    .collect(),
            ),
            TsType::TsLitType(lit) => ProcessedType::LiteralTypes(match lit.lit {
                TsLit::Number(num) => LiteralType::Number(num.value),
                TsLit::Str(str) => LiteralType::String(str.value.to_string()),
                TsLit::Bool(bool) => LiteralType::Boolean(bool.value),
                TsLit::BigInt(bigint) => LiteralType::BigInt(bigint.value.to_string()),
                TsLit::Tpl(tpl) => {
                    LiteralType::Template(tpl.types.iter().map(TypeParser::parser).collect())
                }
            }),
            TsType::TsImportType(import) => ProcessedType::Import(import.arg.value.to_string()),
            TsType::TsKeywordType(keyword) => match keyword.kind {
                TsKeywordTypeKind::TsAnyKeyword => ProcessedType::Any,
                TsKeywordTypeKind::TsUnknownKeyword => ProcessedType::Unknown,
                TsKeywordTypeKind::TsNumberKeyword => ProcessedType::Number,
                TsKeywordTypeKind::TsStringKeyword => ProcessedType::String,
                TsKeywordTypeKind::TsObjectKeyword => ProcessedType::Object,
                TsKeywordTypeKind::TsBigIntKeyword => ProcessedType::BigInt,
                TsKeywordTypeKind::TsSymbolKeyword => ProcessedType::Symbol,
                TsKeywordTypeKind::TsVoidKeyword => ProcessedType::Void,
                TsKeywordTypeKind::TsUndefinedKeyword => ProcessedType::Undefined,
                TsKeywordTypeKind::TsNullKeyword => ProcessedType::Null,
                TsKeywordTypeKind::TsNeverKeyword => ProcessedType::Never,
                TsKeywordTypeKind::TsIntrinsicKeyword => ProcessedType::Intrinsic,
                TsKeywordTypeKind::TsBooleanKeyword => ProcessedType::Boolean,
            },
            _ => ProcessedType::Other(self.clone()),
        }
    }
}
