use std::fmt::Display;

use crate::processor::Processor;

use swc_ecma_ast::{
    Decl::{Class, TsInterface, TsTypeAlias},
    Module,
    ModuleItem::Stmt,
    Stmt::Decl,
    TsType,
};

pub struct Statement {
    pub structure_type: StructureType,
    pub name: String,
    pub content: Content,
}

impl Statement {
    pub fn parse_module(module: &Module) -> Vec<Self> {
        module
            .body
            .iter()
            .filter_map(|body| {
                if let Stmt(Decl(statement)) = body {
                    match statement {
                        Class(class) => Some(class.process()),
                        TsInterface(interface) => Some(interface.process()),
                        TsTypeAlias(type_alias) => Some(type_alias.process()),
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = match self.content {
            Content::Fields(ref fields) => {
                format!(
                    "Fields: \n{}",
                    fields
                        .iter()
                        .map(|field| format!("{field}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
            Content::AliasOfTypes => String::from("Alias"),
            Content::NoContents => String::from("No fields or alias detail available."),
        };
        write!(
            f,
            "Input Name：{}\nType of Structure：{}\n{}",
            self.name, self.structure_type, content
        )
    }
}

pub enum Content {
    Fields(Vec<Field>),
    AliasOfTypes,
    NoContents,
}

pub struct Field {
    pub name: String,
    pub data_type: ProcessedType,
    pub is_optional: bool,
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{} - {}",
            self.name,
            if self.is_optional { " - Optional" } else { "" },
            self.data_type
        )
    }
}

pub enum StructureType {
    Class,
    Interface,
    TypeAlias,
}

impl Display for StructureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Class => write!(f, "Class"),
            Self::Interface => write!(f, "Interface"),
            Self::TypeAlias => write!(f, "Type Alias"),
        }
    }
}

pub enum ProcessedType {
    Array(Box<ProcessedType>),
    Tuple(Vec<ProcessedType>),
    Optional(Box<ProcessedType>),
    Union(Vec<ProcessedType>),
    Intersection(Vec<ProcessedType>),
    TypeReference(String),
    Import(String),
    TypeLiteral(Vec<Field>),
    //Literal(LiteralType),
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
    Intrinsic,
    Boolean,
    Other(Box<TsType>),
}

impl ProcessedType {
    fn value(&self) -> String {
        match self {
            Self::Array(inner_value) => format!("Array of {}", inner_value.value()),
            Self::Tuple(inner_values) => format!(
                "Tuple of {}",
                inner_values
                    .iter()
                    .map(Self::value)
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Optional(inner_value) => format!("Optional {}", inner_value.value()),
            Self::Union(inner_values) => format!(
                "Union of {}",
                inner_values
                    .iter()
                    .map(Self::value)
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Intersection(inner_values) => format!(
                "Intersection of {}",
                inner_values
                    .iter()
                    .map(Self::value)
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::TypeReference(type_name) => type_name.clone(),
            Self::Import(type_import) => format!("Import with `{type_import}`"),
            Self::TypeLiteral(inner_fields) => format!(
                "Object Literal {{{}}}",
                inner_fields
                    .iter()
                    .map(|inner_value| format!("{inner_value}"))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Any => String::from("Any"),
            Self::Unknown => String::from("Unknown"),
            Self::Number => String::from("Number"),
            Self::String => String::from("String"),
            Self::Object => String::from("Object"),
            Self::BigInt => String::from("BigInt"),
            Self::Symbol => String::from("Symbol"),
            Self::Void => String::from("Void"),
            Self::Undefined => String::from("Undefined"),
            Self::Null => String::from("Null"),
            Self::Never => String::from("Never"),
            Self::Intrinsic => String::from("Intrinsic"),
            Self::Boolean => String::from("Boolean"),
            Self::Other(raw_type) => format!("Other type: {raw_type:?}"),
        }
    }
}

impl Display for ProcessedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

enum LiteralType {
    String(String),
    Boolean(bool),
    Number(f64),
    //TemplateLiteral(),
}
