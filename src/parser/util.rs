use combine::error::ParseError;
use combine::parser::char::{alpha_num, char, letter, spaces, string};
use combine::parser::combinator::attempt;
use combine::stream::Stream;
use combine::{choice, eof, many, many1, none_of, one_of, Parser};

fn comment<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        spaces(),
        string("//"),
        many::<String, _, _>(none_of("\n".chars())),
        choice!(char('\n').map(|_| ()), eof().map(|_| ())),
        spaces(),
    )
        .map(|_| ())
}

pub fn commentable_spaces<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice!(attempt(many1(comment())), spaces())
}

pub fn identifier<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let allowed_other_chars = "_@/#".chars();
    let head = letter().or(one_of(allowed_other_chars.clone()));
    let tail = alpha_num().or(one_of(allowed_other_chars));
    (many1::<String, _, _>(head), many::<String, _, _>(tail)).map(|(head, tail)| {
        let mut s = head.to_string();
        s.push_str(tail.as_str());
        s
    })
}

#[cfg(test)]
mod test_comment {
    use crate::parser::util::*;
    use combine::error::StringStreamError;

    #[test]
    fn test_comment() {
        assert_eq!(commentable_spaces().parse(" "), Ok(((), "")));
        assert_eq!(commentable_spaces().parse("// hoge"), Ok(((), "")));
        assert_eq!(
            commentable_spaces().parse(
                "// hoge
                // fuga // piyo"
            ),
            Ok(((), ""))
        );
    }

    #[test]
    fn test_identifier() {
        assert_eq!(
            identifier().parse("3"),
            Err(StringStreamError::UnexpectedParse)
        );
        assert_eq!(
            identifier().parse("3x"),
            Err(StringStreamError::UnexpectedParse)
        );
        assert_eq!(identifier().parse("x"), Ok(("x".to_string(), "")));
        assert_eq!(identifier().parse("Hoge0"), Ok(("Hoge0".to_string(), "")));
        assert_eq!(identifier().parse("_oge0"), Ok(("_oge0".to_string(), "")));
    }
}
