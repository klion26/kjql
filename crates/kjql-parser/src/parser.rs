use winnow::{
    PResult,
    Parser,
    combinator::{
        alt,
        dispatch,
        fail,
        iterator,
        peek,
    },
    token::any,
};

use crate::{
    combinators::{
        parse_array_index,
        parse_array_range,
        parse_flatten_operator,
        parse_group_separator,
        parse_key,
        parse_lenses,
        parse_multi_key,
        parse_object_index,
        parse_object_range,
        parse_pipe_in_operator,
        parse_pipe_out_operator,
        parse_truncate_operator,
        trim,
    },
    errors::KjqlParserError,
    tokens::{
        Lens,
        Range,
        Token,
        View,
    },
};

/// Parses the provided input and map it to the first matching token.
fn parse_fragment<'a>(input: &mut &'a str) -> PResult<Token<'a>> {
    trim(
        dispatch! {peek(any);
            '[' => {
                    alt((
                        parse_array_index.map(Token::ArrayIndexSelector),
                        parse_array_range.map(|(start, end)| Token::ArrayRangeSelector(Range(start, end))),
                    ))
                },
            '"' => parse_key.map(Token::KeySelector),
            '{' => {
                alt((
                    parse_multi_key.map(Token::MultiKeySelector),
                    parse_object_index.map(Token::ObjectIndexSelector),
                    parse_object_range.map(|(start, end)| Token::ObjectRangeSelector(Range(start, end))),
                ))
            },
            '|' => {
                alt((
                    parse_lenses.map(|lenses| {
                        Token::LensSelector(
                            lenses
                                .into_iter()
                                .map(|(tokens, value)| Lens(tokens, value))
                                .collect(),
                        )
                    }),
                    parse_pipe_in_operator.value(Token::PipeInOperator),
                ))
            },
            '.' => parse_flatten_operator.value(Token::FlattenOperator),
            '<' => parse_pipe_out_operator.value(Token::PipeOutOperator),
            ',' => parse_group_separator.value(Token::GroupSeparator),
            '!' => parse_truncate_operator.value(Token::TruncateOperator),
            _ => fail
        }
    )
        .parse_next(input)
}

/// Parses the provided input and returns a vector of `Tokens`.
///
/// # Errors
///
/// Returns a `KjqlParserError` if the input cannot be parsed.
pub fn parse(input: &str) -> Result<Vec<Token>, KjqlParserError> {
    let mut parser_iterator = iterator(input, parse_fragment);
    let tokens = parser_iterator.collect::<Vec<Token>>();
    let result: PResult<_, _> = parser_iterator.finish();

    match result {
        Ok((unparsed, ())) => {
            if !unparsed.is_empty() {
                return Err(KjqlParserError::ParsingError {
                    tokens: tokens.stringify(),
                    unparsed: unparsed.to_string(),
                });
            }

            let trucate_count = tokens
                .iter()
                .filter(|&token| *token == Token::TruncateOperator)
                .count();
            if trucate_count > 1
                || (trucate_count == 1 && tokens.last() != Some(&Token::TruncateOperator))
            {
                return Err(KjqlParserError::TruncateError(tokens.stringify()));
            }

            Ok(tokens)
        }
        Err(_) => Err(KjqlParserError::UnknownError),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse,
        parse_fragment,
    };
    use crate::{
        errors::KjqlParserError,
        tokens::{
            Index,
            Lens,
            LensValue,
            Range,
            Token,
            View,
        },
    };

    #[test]
    fn check_array_index_selector() {
        assert_eq!(
            Ok(Token::ArrayIndexSelector(vec![
                Index(0),
                Index(1),
                Index(2)
            ])),
            parse_fragment(&mut "[0, 1, 2]"),
        );

        assert_eq!(
            Ok(Token::ArrayIndexSelector(vec![
                Index(0),
                Index(1),
                Index(2)
            ])),
            parse_fragment(&mut "[ 0, 1, 2 ]"),
        );
    }

    #[test]
    fn check_array_range_selector() {
        assert_eq!(
            Ok(Token::ArrayRangeSelector(Range(
                Some(Index(0)),
                Some(Index(2))
            ))),
            parse_fragment(&mut "[0:2]"),
        );

        assert_eq!(
            Ok(Token::ArrayRangeSelector(Range(None, Some(Index(2))))),
            parse_fragment(&mut "[:2]"),
        );

        assert_eq!(
            Ok(Token::ArrayRangeSelector(Range(Some(Index(0)), None))),
            parse_fragment(&mut "[0:]"),
        );
        assert_eq!(
            Ok(Token::ArrayRangeSelector(Range(None, None))),
            parse_fragment(&mut "[:]"),
        );
    }

    #[test]
    fn check_key_selector() {
        assert_eq!(
            Ok(Token::KeySelector("one")),
            parse_fragment(&mut r#""one""#),
        );
        assert_eq!(
            Ok(Token::KeySelector("one")),
            parse_fragment(&mut r#" "one" "#),
        );
    }

    #[test]
    fn check_multi_key_selector() {
        assert_eq!(
            Ok(Token::MultiKeySelector(vec!["one", "two", "three"])),
            parse_fragment(&mut r#"{"one", "two", "three"}"#),
        );
        assert_eq!(
            Ok(Token::MultiKeySelector(vec!["one", "two", "three"])),
            parse_fragment(&mut r#" { "one", "two" , "three" } "#),
        );
    }

    #[test]
    fn check_object_index_selector() {
        assert_eq!(
            Ok(Token::ObjectIndexSelector(vec![
                Index(0),
                Index(1),
                Index(2)
            ])),
            parse_fragment(&mut "{0,1,2}"),
        );
        assert_eq!(
            Ok(Token::ObjectIndexSelector(vec![
                Index(0),
                Index(1),
                Index(2)
            ])),
            parse_fragment(&mut " { 0 , 1 , 2 } "),
        );
    }

    #[test]
    fn check_object_range_selector() {
        assert_eq!(
            Ok(Token::ObjectRangeSelector(Range(
                Some(Index(0)),
                Some(Index(2))
            ))),
            parse_fragment(&mut "{0:2}"),
        );
        assert_eq!(
            Ok(Token::ObjectRangeSelector(Range(None, Some(Index(2))))),
            parse_fragment(&mut " {:2}"),
        );
        assert_eq!(
            Ok(Token::ObjectRangeSelector(Range(Some(Index(0)), None))),
            parse_fragment(&mut "{0:}"),
        );
        assert_eq!(
            Ok(Token::ObjectRangeSelector(Range(None, None))),
            parse_fragment(&mut "{:}"),
        );
    }

    #[test]
    fn check_lens_selector() {
        assert_eq!(
            Ok(Token::LensSelector(vec![
                Lens(
                    vec![Token::KeySelector("abc"), Token::KeySelector("c")],
                    None
                ),
                Lens(
                    vec![Token::KeySelector("bcd"), Token::KeySelector("d")],
                    Some(LensValue::Number(123))
                ),
                Lens(vec![Token::KeySelector("efg")], Some(LensValue::Null)),
                Lens(
                    vec![Token::KeySelector("hij")],
                    Some(LensValue::String("test"))
                ),
            ])),
            parse_fragment(&mut r#"|={"abc""c","bcd""d"=123,"efg"=null,"hij"="test"}"#),
        );
    }

    #[test]
    fn check_flatten_operator() {
        assert_eq!(parse_fragment(&mut ".."), Ok(Token::FlattenOperator));
        assert_eq!(parse_fragment(&mut " .. "), Ok(Token::FlattenOperator));
    }

    #[test]
    fn check_pipe_in_operator() {
        assert_eq!(parse_fragment(&mut "|>"), Ok(Token::PipeInOperator));
        assert_eq!(parse_fragment(&mut " |> "), Ok(Token::PipeInOperator));
    }

    #[test]
    fn check_pipe_out_operator() {
        assert_eq!(parse_fragment(&mut "<|"), Ok(Token::PipeOutOperator));
        assert_eq!(parse_fragment(&mut " <| "), Ok(Token::PipeOutOperator));
    }

    #[test]
    fn check_truncate_operator() {
        assert_eq!(parse_fragment(&mut "!"), Ok(Token::TruncateOperator));
        assert_eq!(parse_fragment(&mut " ! "), Ok(Token::TruncateOperator));
    }

    #[test]
    fn check_group_separator() {
        assert_eq!(parse_fragment(&mut ","), Ok(Token::GroupSeparator));
        assert_eq!(parse_fragment(&mut " , "), Ok(Token::GroupSeparator));
    }

    #[test]
    fn check_full_parser() {
        assert_eq!(
            parse(r#""this"[9,0]"#),
            Ok(vec![
                Token::KeySelector("this"),
                Token::ArrayIndexSelector(vec![Index(9), Index(0)])
            ]),
        );
        assert_eq!(
            parse("[9,0]nope"),
            Err(KjqlParserError::ParsingError {
                tokens: [Token::ArrayIndexSelector(vec![Index(9), Index(0)])].stringify(),
                unparsed: "nope".to_string(),
            })
        );
        assert_eq!(
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
            parse(r#""this"[9,0]|>"some"<|"ok"..!"#),
        );
        assert_eq!(
            parse(r#""a"!"b""#),
            Err(KjqlParserError::TruncateError(
                [
                    Token::KeySelector("a"),
                    Token::TruncateOperator,
                    Token::KeySelector("b")
                ]
                .stringify()
            ))
        );
    }
}
