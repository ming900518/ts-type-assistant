mod field_parser;
mod processor;
mod types;

use swc_common::BytePos;
use swc_ecma_ast::Module;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};

use crate::types::Statement;

fn main() {
    TEST_INPUTS
        .iter()
        .for_each(|input| println!("{}\n\n", Statement::parse_module(parser(input))[0]))
}

fn parser(input: &str) -> Module {
    let lexer = Lexer::new(
        Syntax::Typescript(TsConfig::default()),
        Default::default(),
        StringInput::new(input, BytePos(0), BytePos(input.len() as u32)),
        None,
    );
    let mut parser = Parser::new_from(lexer);
    parser
        .parse_typescript_module()
        .expect("failed to parse module")
}

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
