use crate::{
    type_parser::TypeParser,
    types::{Content, ProcessedType, Sum},
};
use swc_common::Span;
use swc_ecma_ast::{
    ClassDecl, ClassMember::ClassProp, TsInterfaceDecl, TsKeywordType, TsKeywordTypeKind, TsType,
    TsTypeAliasDecl, TsUnionOrIntersectionType,
};

use crate::types::{Field, Statement, StructureType};

pub trait Processor {
    fn process(&self) -> Statement;
}

impl Processor for ClassDecl {
    fn process(&self) -> Statement {
        let name = self.ident.sym.to_string();
        let fields = self
            .class
            .body
            .iter()
            .filter_map(|member| {
                if let ClassProp(prop) = member {
                    let (name, is_optional) = prop
                        .key
                        .clone()
                        .ident()
                        .map(|ident| (ident.sym.to_string(), ident.optional))
                        .unwrap_or_default();
                    let data_type = prop
                        .type_ann
                        .clone()
                        .map_or(
                            Box::new(TsType::TsKeywordType(TsKeywordType {
                                span: Span::default(),
                                kind: TsKeywordTypeKind::TsAnyKeyword,
                            })),
                            |ann| ann.type_ann,
                        )
                        .parser();
                    Some(Field {
                        name,
                        data_type,
                        is_optional,
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        Statement {
            structure_type: StructureType::Class,
            name,
            content: if fields.is_empty() {
                Content::NoContents
            } else {
                Content::Fields(fields)
            },
        }
    }
}
impl Processor for Box<TsInterfaceDecl> {
    fn process(&self) -> Statement {
        let name = self.id.sym.to_string();
        let fields = self
            .body
            .body
            .iter()
            .filter_map(|property| {
                property
                    .clone()
                    .ts_property_signature()
                    .map(|signature| {
                        let Some(name) = signature.key.ident().map(|ident| ident.sym.to_string())
                        else {
                            return None;
                        };
                        let is_optional = signature.optional;
                        let Some(data_type) = signature.type_ann.map(|ann| ann.type_ann.parser())
                        else {
                            return None;
                        };

                        Some(Field {
                            name,
                            data_type,
                            is_optional,
                        })
                    })
                    .unwrap_or_default()
            })
            .collect::<Vec<Field>>();
        Statement {
            structure_type: StructureType::Interface,
            name,
            content: if fields.is_empty() {
                Content::NoContents
            } else {
                Content::Fields(fields)
            },
        }
    }
}
impl Processor for Box<TsTypeAliasDecl> {
    fn process(&self) -> Statement {
        let name = self.id.sym.to_string();
        let content = match &*self.clone().type_ann {
            TsType::TsTypeLit(type_literal) => {
                let fields = type_literal
                    .members
                    .iter()
                    .filter_map(|property| {
                        property
                            .clone()
                            .ts_property_signature()
                            .map(|signature| {
                                let Some(name) =
                                    signature.key.ident().map(|ident| ident.sym.to_string())
                                else {
                                    return None;
                                };
                                let is_optional = signature.optional;
                                let Some(data_type) =
                                    signature.type_ann.map(|ann| ann.type_ann.parser())
                                else {
                                    return None;
                                };

                                Some(Field {
                                    name,
                                    data_type,
                                    is_optional,
                                })
                            })
                            .unwrap_or_default()
                    })
                    .collect::<Vec<Field>>();
                Content::Fields(fields)
            }
            TsType::TsUnionOrIntersectionType(content) => {
                let types = content
                    .get_fields()
                    .iter()
                    .map(TypeParser::parser)
                    .collect::<Vec<ProcessedType>>();
                match content {
                    TsUnionOrIntersectionType::TsUnionType(_) => Content::UnionTypes(types),
                    TsUnionOrIntersectionType::TsIntersectionType(_) => {
                        Content::IntersectionTypes(types)
                    }
                }
            }
            _ => Content::NoContents,
        };
        Statement {
            structure_type: StructureType::TypeAlias,
            name,
            content,
        }
    }
}
