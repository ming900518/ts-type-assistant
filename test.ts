interface TestWithInterface {
    stringField: string;
    numberField: number;
    boolField: boolean;
    anyField1: any;
    anyField2;
    arrayField: any[];
    tupleField: [string, boolean];
    unionField: string | null;
    intersectionField: { data1: string } & { data2: string };
    jsObjectField: Map<string, string>;
    queryField: typeof SomeType;
    arrayWithOptionalField: [boolean, string?];
}
