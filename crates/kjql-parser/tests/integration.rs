use kjql_parser::{
    parser::parse,
    tokens::{
        Index,
        Token,
    },
};

#[test]
fn check_parse_integration() {
    assert_eq!(
        parse(r#""this"[9,0]|>"some"<|"ok"..!"#),
        Ok(vec![
            Token::KeySelector("this"),
            Token::ArrayIndexSelector(vec![Index(9), Index(0)]),
            Token::PipeInOperator,
            Token::KeySelector("some"),
            Token::PipeOutOperator,
            Token::KeySelector("ok"),
            Token::FlattenOperator,
            Token::TruncateOperator
        ]),
    );
}
