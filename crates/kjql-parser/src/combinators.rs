use winnow::{
    PResult,
    Parser,
    ascii::{
        digit1,
        multispace0,
    },
    combinator::{
        alt,
        delimited,
        dispatch,
        fail,
        opt,
        peek,
        preceded,
        repeat,
        separated,
        separated_pair,
    },
    error::ParserError,
    token::{
        any,
        literal,
        take_until,
    },
};

use crate::tokens::{
    Index,
    LensValue,
    Range,
    Token,
};

/// Colon.
static COLON: char = ':';
/// Comma.
static COMMA: char = ',';
/// Curly brace open.
static CURLY_BRACKET_OPEN: char = '{';
/// Curly brace close.
static CURLY_BRACKET_CLOSE: char = '}';
/// Double quote.
static DOUBLE_QUOTE: char = '"';
/// Equal.
static EQUAL: char = '=';
/// Square brace open.
static SQUARE_BRACKET_OPEN: char = '[';
/// Square brace close.
static SQUARE_BRACKET_CLOSE: char = ']';
/// False.
static FALSE: &str = "false";
/// True
static TRUE: &str = "true";
/// Lenses start.
static LENSES_START: &str = "|={";
/// Keys operator.
static KEYS: &str = "@";
/// Flatten operator.
static FLATTEN: &str = "..";
/// Group separator.
static GROUP_SEP: &str = ",";
/// Pipe in operator.
static PIPE_IN: &str = "|>";
/// Pipe out operator
static PIPE_OUT: &str = "<|";
/// Truncate operator
static TRUNCATE: &str = "!";

/// A combinator which takes an `inner` parser and produces a parser which also
/// consumes both leading and trailing whitespaces, returning the output of `inner`.
pub(crate) fn trim<'a, F, O, E>(inner: F) -> impl Parser<&'a str, O, E>
where
    E: ParserError<&'a str>,
    F: Parser<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

/// A combinator which parses a stringified number as an `Index`.
pub(crate) fn parse_number(input: &mut &str) -> PResult<Index> {
    digit1.parse_to().parse_next(input)
}

/// A combinator which parses a key surrounded by double quotes.
pub(crate) fn parse_key<'a>(input: &mut &'a str) -> PResult<&'a str> {
    trim(delimited(
        DOUBLE_QUOTE,
        take_until(0.., r#"""#),
        DOUBLE_QUOTE,
    ))
    .parse_next(input)
}

/// A combinator which parses a list of `Index`
pub(crate) fn parse_indexes(input: &mut &str) -> PResult<Vec<Index>> {
    separated(1.., parse_number, trim(COMMA)).parse_next(input)
}

/// A combinator which parses a list of keys.
fn parse_keys<'a>(input: &mut &'a str) -> PResult<Vec<&'a str>> {
    trim(separated(1.., parse_key, trim(COMMA))).parse_next(input)
}

/// A combinator which parses a list of keys surrounded by curly braces.
pub(crate) fn parse_multi_key<'a>(input: &mut &'a str) -> PResult<Vec<&'a str>> {
    delimited(CURLY_BRACKET_OPEN, parse_keys, CURLY_BRACKET_CLOSE).parse_next(input)
}

/// A combinator which parses an array of `Index`
pub(crate) fn parse_array_index(input: &mut &str) -> PResult<Vec<Index>> {
    delimited(
        trim(SQUARE_BRACKET_OPEN),
        parse_indexes,
        trim(SQUARE_BRACKET_CLOSE),
    )
    .parse_next(input)
}

/// A combinator which parses an array range.
pub(crate) fn parse_array_range(input: &mut &str) -> PResult<(Option<Index>, Option<Index>)> {
    trim(delimited(
        trim(SQUARE_BRACKET_OPEN),
        separated_pair(opt(parse_number), trim(COLON), opt(parse_number)),
        trim(SQUARE_BRACKET_CLOSE),
    ))
    .parse_next(input)
}

/// A combinator which parses a list of index surrounded by curly braces.
pub(crate) fn parse_object_index(input: &mut &str) -> PResult<Vec<Index>> {
    delimited(
        trim(CURLY_BRACKET_OPEN),
        parse_indexes,
        trim(CURLY_BRACKET_CLOSE),
    )
    .parse_next(input)
}

/// A combinator which parses an object range.
pub(crate) fn parse_object_range(input: &mut &str) -> PResult<(Option<Index>, Option<Index>)> {
    delimited(
        trim(CURLY_BRACKET_OPEN),
        separated_pair(opt(parse_number), trim(COLON), opt(parse_number)),
        trim(CURLY_BRACKET_CLOSE),
    )
    .parse_next(input)
}

/// A combinator which parses a lens key.
fn parse_lens_key<'a>(input: &mut &'a str) -> PResult<Token<'a>> {
    trim(dispatch! {peek(any);
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
        _ => fail
    })
        .parse_next(input)
}

/// A combinator which parses multiple lens keys.
fn parse_lens_keys<'a>(input: &mut &'a str) -> PResult<Vec<Token<'a>>> {
    repeat(1.., parse_lens_key).parse_next(input)
}
/// A combinator which parses any lens value.
pub(crate) fn parse_lens_value<'a>(input: &mut &'a str) -> PResult<LensValue<'a>> {
    dispatch! {peek(any);
        'f' => FALSE.value(LensValue::Bool(false)),
        't' => TRUE.value(LensValue::Bool(true)),
        'n' => "null".value(LensValue::Null),
        '0'..='9' => digit1.try_map(|s: &str| s.parse::<usize>().map(LensValue::Number)),
        _ => parse_key.map(LensValue::String),
    }
    .parse_next(input)
}

/// A combinator which parses a lens.
pub(crate) fn parse_lens<'a>(
    input: &mut &'a str,
) -> PResult<(Vec<Token<'a>>, Option<LensValue<'a>>)> {
    trim((
        parse_lens_keys,
        opt(preceded(trim(EQUAL), parse_lens_value)),
    ))
    .parse_next(input)
}

/// A combinator which parses a list of lenses.
pub(crate) fn parse_lenses<'a>(
    input: &mut &'a str,
) -> PResult<Vec<(Vec<Token<'a>>, Option<LensValue<'a>>)>> {
    delimited(
        trim(LENSES_START),
        separated(1.., parse_lens, trim(COMMA)),
        trim(CURLY_BRACKET_CLOSE),
    )
    .parse_next(input)
}

/// A combinator which parses a keys operator.
pub(crate) fn parse_keys_operator<'a>(input: &mut &'a str) -> PResult<&'a str> {
    literal(KEYS).parse_next(input)
}

/// A combinator which parses a flatten operator.
pub(crate) fn parse_flatten_operator<'a>(input: &mut &'a str) -> PResult<&'a str> {
    literal(FLATTEN).parse_next(input)
}

/// A combinator which parses a pipe in operator.
pub(crate) fn parse_pipe_in_operator<'a>(input: &mut &'a str) -> PResult<&'a str> {
    literal(PIPE_IN).parse_next(input)
}

/// A combinator which parses a pipe out operator.
pub(crate) fn parse_pipe_out_operator<'a>(input: &mut &'a str) -> PResult<&'a str> {
    literal(PIPE_OUT).parse_next(input)
}

/// A combinator which parses a truncate operator.
pub(crate) fn parse_truncate_operator<'a>(input: &mut &'a str) -> PResult<&'a str> {
    trim(TRUNCATE).parse_next(input)
}

/// A combinator which parses a group separator.
pub(crate) fn parse_group_separator<'a>(input: &mut &'a str) -> PResult<&'a str> {
    literal(GROUP_SEP).parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::{
        FLATTEN,
        GROUP_SEP,
        KEYS,
        PIPE_IN,
        PIPE_OUT,
        TRUNCATE,
        parse_array_index,
        parse_array_range,
        parse_flatten_operator,
        parse_group_separator,
        parse_indexes,
        parse_key,
        parse_keys_operator,
        parse_lens,
        parse_lenses,
        parse_multi_key,
        parse_number,
        parse_object_index,
        parse_object_range,
        parse_pipe_in_operator,
        parse_pipe_out_operator,
        parse_truncate_operator,
    };
    use crate::tokens::{
        Index,
        LensValue,
        Token,
    };

    #[test]
    fn check_parse_number() {
        assert_eq!(Ok(Index(123)), parse_number(&mut "123"));
        assert!(parse_number(&mut "abc").is_err());
        assert!(parse_number(&mut "abc123").is_err());
    }

    #[test]
    fn check_parse_key() {
        assert_eq!(Ok("abc"), parse_key(&mut r#""abc""#));
        assert!(parse_key(&mut "abc").is_err());
    }

    #[test]
    fn check_parse_indexes() {
        assert_eq!(Ok(vec![Index(123)]), parse_indexes(&mut "123"));
        assert_eq!(
            Ok(vec![Index(123), Index(456), Index(789)]),
            parse_indexes(&mut "123,456,789"),
        );
        assert!(parse_indexes(&mut "abc").is_err());
    }

    #[test]
    fn check_parse_multi_key() {
        assert_eq!(Ok(vec!["abc"]), parse_multi_key(&mut r#"{"abc"}"#));
        assert_eq!(
            Ok(vec!["abc", "def"]),
            parse_multi_key(&mut r#"{"abc", "def"}"#),
        );
        assert!(parse_multi_key(&mut "{}").is_err());
        assert!(parse_multi_key(&mut "{123}").is_err());
    }

    #[test]
    fn check_parse_array_index() {
        assert_eq!(Ok(vec![Index(1)]), parse_array_index(&mut "[1]"));
        assert_eq!(
            Ok(vec![Index(1), Index(2), Index(3)]),
            parse_array_index(&mut "[1,2,3]"),
        );
        assert!(parse_array_index(&mut "[]").is_err());
        assert!(parse_array_index(&mut r#"["1"]"#).is_err());
    }

    #[test]
    fn check_parse_array_range() {
        assert_eq!(Ok((None, None)), parse_array_range(&mut "[:]"),);
        assert_eq!(Ok((Some(Index(1)), None)), parse_array_range(&mut "[1:]"),);
        assert_eq!(Ok((None, Some(Index(1)))), parse_array_range(&mut "[:1]"),);
        assert_eq!(
            Ok((Some(Index(1)), Some(Index(3)))),
            parse_array_range(&mut "[1:3]"),
        );
        assert!(parse_array_range(&mut "[]").is_err());
    }

    #[test]
    fn check_parse_object_index() {
        assert_eq!(Ok(vec![Index(1)]), parse_object_index(&mut "{1}"),);
        assert_eq!(
            Ok(vec![Index(1), Index(2), Index(3)]),
            parse_object_index(&mut "{1,2,3}"),
        );
        assert!(parse_object_index(&mut "{}").is_err());
        assert!(parse_object_index(&mut "{1,2,3").is_err());
    }

    #[test]
    fn check_parse_object_range() {
        assert_eq!(Ok((None, None)), parse_object_range(&mut "{:}"),);
        assert_eq!(
            Ok((Some(Index(1)), Some(Index(3)))),
            parse_object_range(&mut "{1:3}"),
        );
        assert_eq!(Ok((Some(Index(1)), None)), parse_object_range(&mut "{1:}"),);
        assert!(parse_object_range(&mut "{}").is_err());
        assert!(parse_object_range(&mut "{1:3").is_err());
    }

    #[test]
    fn check_parse_keys_operator() {
        assert_eq!(Ok(KEYS), parse_keys_operator(&mut "@"));
        assert!(parse_keys_operator(&mut "").is_err());
    }

    #[test]
    fn check_parse_flatten_operator() {
        assert_eq!(Ok(FLATTEN), parse_flatten_operator(&mut ".."),);
        assert!(parse_flatten_operator(&mut "").is_err());
    }

    #[test]
    fn check_parse_pipe_in_operator() {
        assert_eq!(Ok(PIPE_IN), parse_pipe_in_operator(&mut "|>"),);
        assert!(parse_pipe_in_operator(&mut "").is_err());
    }

    #[test]
    fn check_parse_pipe_out_operator() {
        assert_eq!(Ok(PIPE_OUT), parse_pipe_out_operator(&mut "<|"),);
        assert!(parse_pipe_out_operator(&mut "").is_err());
    }

    #[test]
    fn check_parse_truncate_operator() {
        assert_eq!(Ok(TRUNCATE), parse_truncate_operator(&mut "!"),);
        assert!(parse_truncate_operator(&mut "").is_err());
    }

    #[test]
    fn check_parse_group_separator() {
        assert_eq!(Ok(GROUP_SEP), parse_group_separator(&mut ","),);
        assert!(parse_group_separator(&mut "").is_err());
    }

    #[test]
    fn check_parse_lens() {
        assert_eq!(
            Ok((vec![Token::KeySelector("abc")], None)),
            parse_lens(&mut r#""abc""#)
        );
        assert_eq!(
            Ok((vec![Token::KeySelector("abc")], Some(LensValue::Null))),
            parse_lens(&mut r#""abc"=null"#),
        );
        assert_eq!(
            Ok((
                vec![Token::KeySelector("abc")],
                Some(LensValue::Number(123))
            )),
            parse_lens(&mut r#""abc"=123"#),
        );
        assert_eq!(
            Ok((
                vec![Token::KeySelector("abc")],
                Some(LensValue::String("def"))
            )),
            parse_lens(&mut r#""abc"="def""#),
        );
        assert!(parse_lenses(&mut "").is_err());
    }

    #[test]
    fn check_parse_lenses() {
        assert_eq!(
            Ok(vec![
                (vec![Token::KeySelector("abc")], None),
                (
                    vec![Token::KeySelector("bcd")],
                    Some(LensValue::Number(123))
                ),
                (vec![Token::KeySelector("efg")], Some(LensValue::Null)),
                (
                    vec![Token::KeySelector("hij")],
                    Some(LensValue::String("test"))
                )
            ]),
            parse_lenses(&mut r#"|={"abc", "bcd"=123,"efg"=null,"hij"="test"}"#),
        );
    }
}
