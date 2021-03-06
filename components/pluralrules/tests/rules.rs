mod fixtures;
mod helpers;

use icu_pluralrules::rules::{parse, parse_condition, test_condition, Lexer};
use icu_pluralrules::PluralOperands;

#[test]
fn test_parsing_operands() {
    let path = "./tests/fixtures/rules.json";
    let test_set: fixtures::RuleTestSet =
        helpers::read_fixture(path).expect("Failed to read a fixture");

    for test in test_set.0 {
        match test.output {
            fixtures::RuleTestOutput::Value(val) => {
                let lex = Lexer::new(test.rule.as_bytes());
                lex.count();
                let ast = parse_condition(test.rule.as_bytes()).expect("Failed to parse.");
                let operands: PluralOperands = test.input.into();

                if val {
                    assert!(test_condition(&ast, &operands));
                } else {
                    assert!(!test_condition(&ast, &operands));
                }
            }
            fixtures::RuleTestOutput::Error(val) => {
                let err = parse(test.rule.as_bytes()).unwrap_err();
                assert_eq!(format!("{:?}", err), val);
            }
        }
    }
}
