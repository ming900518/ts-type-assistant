use std::{
    error::Error,
    io,
    ops::Deref,
    path::{Path, PathBuf},
};

use oxc::ast::ast::{
    ClassElement, Declaration, PropertyDefinition, PropertyKey, TSLiteral, TSLiteralType,
    TSPropertySignature, TSSignature, TSTupleElement, TSType,
};

#[derive(Debug)]
pub struct ParsedStructure(pub StructureInfo, pub Fields);

impl ParsedStructure {
    pub fn try_new(declaration: Declaration, path: &Path) -> Result<Self, Box<dyn Error>> {
        let structure_info = StructureInfo {
            struct_name: StructureName::try_from(&declaration)?,
            struct_type: StructureType::try_from(&declaration)?,
            file_path: path.to_path_buf(),
        };

        let fields = Fields::try_from(declaration)?;

        Ok(Self(structure_info, fields))
    }
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub struct StructureInfo {
    struct_name: StructureName,
    struct_type: StructureType,
    file_path: PathBuf,
}

#[derive(Debug, Default)]
pub enum DataType {
    Array(Box<DataType>),
    Tuple(Vec<DataType>),
    Optional(Box<DataType>),
    Union(Vec<DataType>),
    Intersection(Vec<DataType>),
    TypeLiteral(Fields),
    LiteralTypes(LiteralType),
    #[default]
    Any,
    Unknown,
    Number,
    String,
    Object,
    BigInt,
    Symbol,
    Void,
    Undefined,
    Null,
    Never,
    Boolean,
    Other,
}

impl From<TSType<'_>> for DataType {
    fn from(value: TSType) -> Self {
        match value {
            TSType::TSAnyKeyword(_) => Self::Any,
            TSType::TSBigIntKeyword(_) => Self::BigInt,
            TSType::TSBooleanKeyword(_) => Self::Boolean,
            TSType::TSNeverKeyword(_) => Self::Never,
            TSType::TSNullKeyword(_) => Self::Null,
            TSType::TSNumberKeyword(_) => Self::Number,
            TSType::TSObjectKeyword(_) => Self::Object,
            TSType::TSStringKeyword(_) => Self::String,
            TSType::TSSymbolKeyword(_) => Self::Symbol,
            TSType::TSUndefinedKeyword(_) => Self::Undefined,
            TSType::TSUnknownKeyword(_) => Self::Unknown,
            TSType::TSVoidKeyword(_) => Self::Void,
            TSType::TSArrayType(array) => {
                Self::Array(Box::new(Self::from(array.unbox().element_type)))
            }
            TSType::TSIntersectionType(intersection) => Self::Intersection(
                intersection
                    .unbox()
                    .types
                    .into_iter()
                    .map(Self::from)
                    .collect(),
            ),
            TSType::TSTupleType(tuple) => Self::Tuple(
                tuple
                    .unbox()
                    .element_types
                    .into_iter()
                    .map(|tuple_element| match tuple_element {
                        TSTupleElement::TSType(ts_types) => Self::from(ts_types),
                        TSTupleElement::TSOptionalType(optional_types) => Self::Optional(Box::new(
                            Self::from(optional_types.unbox().type_annotation),
                        )),
                        _ => Self::Other,
                    })
                    .collect(),
            ),
            TSType::TSTypeLiteral(type_literal) => Self::TypeLiteral(Fields(
                type_literal
                    .unbox()
                    .members
                    .into_iter()
                    .filter_map(|signature| {
                        if let TSSignature::TSPropertySignature(property) = signature {
                            Some(Field::from(property))
                        } else {
                            None
                        }
                    })
                    .collect(),
            )),
            TSType::TSLiteralType(literal) => Self::LiteralTypes(LiteralType::from(literal)),
            TSType::TSUnionType(union) => {
                Self::Union(union.unbox().types.into_iter().map(Self::from).collect())
            }
            _ => Self::Other,
        }
    }
}

#[derive(Debug)]
pub enum LiteralType {
    String(String),
    Boolean(bool),
    Number(f64),
    BigInt(String),
    RegExp(String),
    Template,
    UnaryExpression,
    Null,
}

impl From<oxc::allocator::Box<'_, TSLiteralType<'_>>> for LiteralType {
    fn from(value: oxc::allocator::Box<'_, TSLiteralType<'_>>) -> Self {
        let value = value.unbox();

        match value.literal {
            TSLiteral::BooleanLiteral(boolean) => Self::Boolean(boolean.value),
            TSLiteral::NullLiteral(_) => Self::Null,
            TSLiteral::NumberLiteral(number) => Self::Number(number.value),
            TSLiteral::BigintLiteral(bigint) => Self::BigInt(bigint.value.to_string()),
            TSLiteral::RegExpLiteral(regexp) => Self::RegExp(regexp.regex.to_string()),
            TSLiteral::StringLiteral(string) => Self::String(string.value.to_string()),
            TSLiteral::TemplateLiteral(_) => Self::Template,
            TSLiteral::UnaryExpression(_) => Self::UnaryExpression,
        }
    }
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub struct StructureName(String);

impl Deref for StructureName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&Declaration<'_>> for StructureName {
    type Error = io::Error;

    fn try_from(value: &Declaration<'_>) -> Result<Self, Self::Error> {
        match value {
            Declaration::ClassDeclaration(class) => class.id.as_ref().map_or_else(
                || {
                    Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "Class name not found.",
                    ))
                },
                |id| Ok(Self(id.name.to_string())),
            ),
            Declaration::TSInterfaceDeclaration(interface) => {
                Ok(Self(interface.id.name.to_string()))
            }
            Declaration::TSTypeAliasDeclaration(type_alias) => {
                Ok(Self(type_alias.id.name.to_string()))
            }
            _ => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Type unsupported",
            )),
        }
    }
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub data_type: DataType,
    pub optional: bool,
}

impl From<oxc::allocator::Box<'_, PropertyDefinition<'_>>> for Field {
    fn from(value: oxc::allocator::Box<'_, PropertyDefinition<'_>>) -> Self {
        let value = value.unbox();
        Self {
            name: match value.key {
                PropertyKey::Identifier(id) => id.name.to_string(),
                PropertyKey::PrivateIdentifier(id) => id.name.to_string(),
                PropertyKey::Expression(_expr) => String::from("expr"),
            },
            data_type: value
                .type_annotation
                .map(|type_annotation| DataType::from(type_annotation.unbox().type_annotation))
                .unwrap_or_default(),
            optional: value.optional,
        }
    }
}

impl From<oxc::allocator::Box<'_, TSPropertySignature<'_>>> for Field {
    fn from(value: oxc::allocator::Box<'_, TSPropertySignature<'_>>) -> Self {
        let value = value.unbox();
        Self {
            name: match value.key {
                PropertyKey::Identifier(id) => id.name.to_string(),
                PropertyKey::PrivateIdentifier(id) => id.name.to_string(),
                PropertyKey::Expression(_expr) => String::from("expr"),
            },
            data_type: value
                .type_annotation
                .map(|type_annotation| DataType::from(type_annotation.unbox().type_annotation))
                .unwrap_or_default(),
            optional: value.optional,
        }
    }
}

#[derive(Debug)]
pub struct Fields(Vec<Field>);

impl Deref for Fields {
    type Target = Vec<Field>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<Declaration<'_>> for Fields {
    type Error = io::Error;

    fn try_from(value: Declaration<'_>) -> Result<Self, Self::Error> {
        match value {
            Declaration::ClassDeclaration(class) => Ok(Self(
                class
                    .unbox()
                    .body
                    .unbox()
                    .body
                    .into_iter()
                    .filter_map(|element| {
                        if let ClassElement::PropertyDefinition(property) = element {
                            Some(Field::from(property))
                        } else {
                            None
                        }
                    })
                    .collect(),
            )),
            Declaration::TSInterfaceDeclaration(interface) => Ok(Self(
                interface
                    .unbox()
                    .body
                    .unbox()
                    .body
                    .into_iter()
                    .filter_map(|signature| {
                        if let TSSignature::TSPropertySignature(property) = signature {
                            Some(Field::from(property))
                        } else {
                            None
                        }
                    })
                    .collect(),
            )),
            Declaration::TSTypeAliasDeclaration(type_alias) => {
                if let DataType::TypeLiteral(type_literal) =
                    DataType::from(type_alias.unbox().type_annotation)
                {
                    Ok(type_literal)
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unable to parse the content of the type literal.",
                    ))
                }
            }
            _ => Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported.")),
        }
    }
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum StructureType {
    Class,
    Interface,
    TypeAlias,
}

impl TryFrom<&Declaration<'_>> for StructureType {
    type Error = io::Error;

    fn try_from(value: &Declaration) -> Result<Self, Self::Error> {
        match &value {
            Declaration::ClassDeclaration(_) => Ok(Self::Class),
            Declaration::TSInterfaceDeclaration(_) => Ok(Self::Interface),
            Declaration::TSTypeAliasDeclaration(_) => Ok(Self::TypeAlias),
            _ => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Type Unsupported.",
            )),
        }
    }
}
