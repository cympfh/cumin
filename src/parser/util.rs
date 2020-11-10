use combine::error::ParseError;
use combine::parser::char::{char, spaces, string};
use combine::parser::combinator::attempt;
use combine::stream::Stream;
use combine::{choice, eof, many, many1, none_of, Parser};

fn comment<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    spaces()
        .with(string("//"))
        .with(many::<String, _, _>(none_of("\n".chars())))
        .with(choice!(char('\n').map(|_| ()), eof().map(|_| ())))
        .with(spaces())
}

pub fn commentable_spaces<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice!(attempt(many1(comment())), spaces())
}

#[cfg(test)]
mod test_comment {
    use crate::parser::util::*;

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
}
