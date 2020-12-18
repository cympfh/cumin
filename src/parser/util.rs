use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_while, take_while1},
    combinator::{eof, opt},
    multi::many0,
    sequence::tuple,
    IResult,
};

pub fn spaces(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_whitespace())(input)
}

fn comment(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("//")(input)?;
    let (input, _) = opt(is_not("\n\r"))(input)?;
    alt((eof, spaces))(input)
}

pub fn commentable_spaces(input: &str) -> IResult<&str, ()> {
    let (input, _) = spaces(input)?;
    let (input, _) = many0(tuple((comment, spaces)))(input)?;
    Ok((input, ()))
}

pub fn identifier(input: &str) -> IResult<&str, String> {
    fn head(c: char) -> bool {
        c.is_alphabetic() || c == '_' || c == '#' || c == '@'
    }
    fn tail(c: char) -> bool {
        c.is_alphanumeric() || head(c)
    }
    let (input, s) = take_while1(head)(input)?;
    let (input, t) = take_while(tail)(input)?;
    let mut name = String::new();
    name.push_str(s);
    name.push_str(t);
    Ok((input, name))
}

#[cfg(test)]
mod test_comment {
    use crate::parser::util::*;

    #[test]
    fn test_comment() {
        assert_eq!(commentable_spaces(""), Ok(("", ())));
        assert_eq!(commentable_spaces(" \t\n"), Ok(("", ())));
        assert_eq!(commentable_spaces("//"), Ok(("", ())));
        assert_eq!(commentable_spaces("// "), Ok(("", ())));
        assert_eq!(commentable_spaces("// hoge"), Ok(("", ())));
        assert_eq!(
            commentable_spaces(
                "//
                // hoge
                //"
            ),
            Ok(("", ()))
        );
        assert_eq!(
            commentable_spaces(
                "// hoge
                // fuga"
            ),
            Ok(("", ()))
        );
        assert_eq!(
            commentable_spaces(
                "// hoge

                let x = 1; // fuga"
            ),
            Ok(("let x = 1; // fuga", ()))
        );
    }

    #[test]
    fn test_identifier() {
        assert!(identifier("3").is_err());
        assert!(identifier("3x").is_err());
        assert_eq!(identifier("x").unwrap(), ("", "x".to_string()));
        assert_eq!(identifier("x0").unwrap(), ("", "x0".to_string()));
        assert_eq!(identifier("_x").unwrap(), ("", "_x".to_string()));
    }
}
