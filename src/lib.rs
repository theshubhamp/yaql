#[macro_use] extern crate lalrpop_util;

lalrpop_mod!(pub yaql); // synthesized by LALRPOP

#[test]
fn parse() {
    assert!(yaql::TermParser::new().parse("22").is_ok());
    assert!(yaql::TermParser::new().parse("(22)").is_ok());
    assert!(yaql::TermParser::new().parse("((((22))))").is_ok());
    assert!(yaql::TermParser::new().parse("((22)").is_err());
}
