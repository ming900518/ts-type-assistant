use std::{fs, path::PathBuf, sync::OnceLock};

use dashmap::DashMap;
use oxc::{
    allocator::Allocator,
    ast::ast::{Declaration, ModuleDeclaration, Statement},
    parser::Parser,
    span::SourceType,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::types::{Fields, ParsedStructure, StructureInfo};

pub static PARSED: OnceLock<DashMap<StructureInfo, Fields>> = OnceLock::new();

pub fn parse_all(files: &[PathBuf]) {
    let parsed = PARSED.get_or_init(DashMap::new);

    files
        .par_iter()
        .filter_map(|path| fs::read_to_string(path).ok().map(|content| (path, content)))
        .for_each(|(path, content)| {
            let allocator = Allocator::default();
            let source_type = SourceType::from_path(path);
            Parser::new(&allocator, &content, source_type.unwrap_or_default())
                .parse()
                .program
                .body
                .into_iter()
                .filter_map(parse_statement)
                .for_each(|declaration| {
                    ParsedStructure::try_new(declaration, path).map_or_else(
                        |error| {
                            eprintln!("Error when parsing {}: {error:?}", path.to_string_lossy());
                        },
                        |ParsedStructure(key, value)| {
                            parsed.insert(key, value);
                        },
                    );
                });
        });

    fn parse_statement(statement: Statement) -> Option<Declaration> {
        match statement {
            Statement::ModuleDeclaration(module_declaration) => {
                if let ModuleDeclaration::ExportNamedDeclaration(export_declaration) =
                    module_declaration.unbox()
                {
                    export_declaration.unbox().declaration
                } else {
                    None
                }
            }
            Statement::Declaration(declaration) => Some(declaration),
            _ => None,
        }
    }
}
