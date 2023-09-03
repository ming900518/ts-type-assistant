use std::fmt::Display;

use crate::processor::Processor;

use swc_ecma_ast::{
    Decl::{Class, TsInterface, TsTypeAlias},
    Module,
    ModuleItem::Stmt,
    Stmt::Decl,
    TsIntersectionType, TsType, TsUnionOrIntersectionType, TsUnionType,
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
                    "欄位: \n{}",
                    fields
                        .iter()
                        .map(|field| format!("{field}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
            Content::UnionTypes(ref types) => {
                format!(
                    "聯集型別（Union Type）代稱: \n{}",
                    types
                        .iter()
                        .map(|field| format!("{field}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
            Content::IntersectionTypes(ref types) => {
                format!(
                    "交集型別（Intersection Type）代稱： \n{}",
                    types
                        .iter()
                        .map(|field| format!("{field}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
            Content::NoContents => String::from("無欄位或型別代稱存在"),
        };
        write!(f, "{} - {}\n\n{}", self.name, self.structure_type, content)
    }
}

pub enum Content {
    Fields(Vec<Field>),
    UnionTypes(Vec<ProcessedType>),
    IntersectionTypes(Vec<ProcessedType>),
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
            "{}{}: {}",
            self.name,
            if self.is_optional { " - 非必填" } else { "" },
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
            Self::Class => write!(f, "類別（Class）"),
            Self::Interface => write!(f, "介面（Interface）"),
            Self::TypeAlias => write!(f, "型別代稱（Type Alias）"),
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
    LiteralTypes(LiteralType),
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
            Self::Array(inner_value) => format!("{}的陣列（Array）", inner_value.value()),
            Self::Tuple(inner_values) => format!(
                "{}的元組（Tuple）",
                inner_values
                    .iter()
                    .map(Self::value)
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Optional(inner_value) => format!("{}（非必填）", inner_value.value()),
            Self::Union(inner_values) => format!(
                "{}的聯集（Union）",
                inner_values
                    .iter()
                    .map(Self::value)
                    .collect::<Vec<String>>()
                    .join("、")
            ),
            Self::Intersection(inner_values) => format!(
                "{}的交集（Intersection）",
                inner_values
                    .iter()
                    .map(Self::value)
                    .collect::<Vec<String>>()
                    .join("、")
            ),
            Self::TypeReference(type_name) => format!("{}類別", type_name.clone()),
            Self::Import(type_import) => format!("由`{type_import}`引入"),
            Self::TypeLiteral(inner_fields) => format!(
                "實字物件（Object Literal，包含{}）",
                inner_fields
                    .iter()
                    .map(|inner_value| format!("{inner_value}"))
                    .collect::<Vec<String>>()
                    .join("、")
            ),
            Self::LiteralTypes(types) => format!("實字（Literal）型別 \"{types}\"",),
            Self::Any => String::from("任意（any）"),
            Self::Unknown => String::from("未知（unknown）"),
            Self::Number => String::from("數字（number）"),
            Self::String => String::from("字串（string）"),
            Self::Object => String::from("物件（Object）"),
            Self::BigInt => String::from("大數字（bigint）"),
            Self::Symbol => String::from("符號（Symbol）"),
            Self::Void => String::from("沒有回傳值（void）"),
            Self::Undefined => String::from("未初始化（undefined）"),
            Self::Null => String::from("空值（null）"),
            Self::Never => String::from("不預期的空值（never）"),
            Self::Intrinsic => String::from("編譯用型別（Intrinsic）"),
            Self::Boolean => String::from("布林（boolean）"),
            Self::Other(raw_type) => format!("其他型別：{raw_type:?}"),
        }
    }
}

impl Display for ProcessedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[allow(clippy::vec_box)]
pub trait Sum {
    fn get_fields(&self) -> Vec<Box<TsType>>;
}
impl Sum for TsUnionOrIntersectionType {
    fn get_fields(&self) -> Vec<Box<TsType>> {
        match self {
            Self::TsUnionType(union) => union.get_fields(),
            Self::TsIntersectionType(intersection) => intersection.get_fields(),
        }
    }
}
impl Sum for TsUnionType {
    fn get_fields(&self) -> Vec<Box<TsType>> {
        self.clone().types
    }
}
impl Sum for TsIntersectionType {
    fn get_fields(&self) -> Vec<Box<TsType>> {
        self.clone().types
    }
}

pub enum LiteralType {
    String(String),
    Boolean(bool),
    Number(f64),
    BigInt(String),
    Template(Vec<ProcessedType>),
}

impl Display for LiteralType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(ref str) => write!(f, "字串 \"{str}\""),
            Self::Boolean(ref bool) => write!(f, "布林 {bool}"),
            Self::Number(ref num) => write!(f, "數字 {num}"),
            Self::BigInt(ref bigint) => write!(f, "大數字 {bigint}"),
            Self::Template(ref template) => write!(
                f,
                "{}",
                template
                    .iter()
                    .map(|inner_value| format!("{inner_value}"))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}
