#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::perf)]

mod processor;
mod type_parser;
mod types;

use swc_common::BytePos;
use swc_ecma_ast::{EsVersion, Module};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};

use crate::types::Statement;

fn main() {
    let input = r"type Janitor =
  | { kind: 'test1' }
  | { kind: 'test2'; partnerId: string }| `test${AAA}`";
    for statement in Statement::parse_module(&parser(input)) {
        println!("{statement}");
    }
}

fn parser(input: &str) -> Module {
    let lexer = Lexer::new(
        Syntax::Typescript(TsConfig::default()),
        EsVersion::latest(),
        StringInput::new(
            input,
            BytePos(0),
            BytePos(u32::try_from(input.len()).expect("Unable to convert usize to u32")),
        ),
        None,
    );
    let mut parser = Parser::new_from(lexer);
    parser
        .parse_typescript_module()
        .expect("failed to parse module")
}

#[allow(dead_code)]
const TEST_INPUTS: [&str; 4] = [
    "type TestWithTypeAliasToClass = Map<string, string>",
    r"type TestWithTypeAliasToObjectLiteral = {
    stringField: string,
    numberField: number,
    boolField: boolean,
    anyField1: any,
    anyField2,
    arrayField: any[],
    tupleField: [string, boolean],
    unionField: string | null,
    intersectionField: {data1: string} & {data2: string},
    jsObjectField: Map<string, string>,
    queryField: typeof SomeType,
    arrayWithOptionalField: [boolean, string?]
}",
    r"interface TestWithInterface {
    stringField: string;
    numberField: number;
    boolField: boolean;
    anyField1: any;
    anyField2;
    arrayField: any[];
    tupleField: [string, boolean];
    unionField: string | null;
    intersectionField: {data1: string} & {data2: string};
    jsObjectField: Map<string, string>;
    queryField: typeof SomeType;
    arrayWithOptionalField: [boolean, string?];
}",
    r"class TestWithClass {
    stringField: string;
    numberField: number;
    boolField: boolean;
    anyField1: any;
    anyField2;
    arrayField: any[];
    tupleField: [string, boolean];
    unionField: string | null;
    intersectionField: {data1: string} & {data2: string};
    jsObjectField: Map<string, string>;
    queryField: typeof SomeType;
    arrayWithOptionalField: [boolean, string?];

    constructor() {}
}",
];
