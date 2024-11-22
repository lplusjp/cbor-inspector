use nom::branch::alt;
use nom::bytes::complete::{tag, take_till, take_while_m_n};
use nom::character::complete::{line_ending, multispace1};
use nom::combinator::{map_res, opt};
use nom::sequence::delimited;
use nom::IResult;

fn hex_byte(input: &str) -> IResult<&str, u8> {
    let parser = take_while_m_n(2, 2, |c: char| c.is_ascii_hexdigit());
    map_res(parser, |s: &str| u8::from_str_radix(s, 16))(input)
}

fn comment_hyphen(input: &str) -> IResult<&str, ()> {
    let (input, _) = delimited(tag("--"), take_till(|c| c == '\n'), opt(line_ending))(input)?;
    Ok((input, ()))
}

fn comment_hash(input: &str) -> IResult<&str, ()> {
    let (input, _) = delimited(tag("#"), take_till(|c| c == '\n'), opt(line_ending))(input)?;
    Ok((input, ()))
}

fn comment(input: &str) -> IResult<&str, ()> {
    alt((comment_hyphen, comment_hash))(input)
}

pub fn parse_hex(input: &str) -> IResult<&str, Vec<u8>> {
    let mut bytes = Vec::new();
    let mut input = input;
    while !input.is_empty() {
        // spaces
        if let Ok((input_new, _)) = multispace1::<_, ()>(input) {
            input = input_new;
            continue;
        }
        // comment
        if let Ok((input_new, _)) = comment(input) {
            input = input_new;
            continue;
        }

        // hex byte
        let (input_new, byte) = hex_byte(input)?;
        input = input_new;
        bytes.push(byte);
    }
    Ok((input, bytes))
}

#[cfg(test)]
mod tests {
    use super::parse_hex;

    #[test]
    fn test_parse_hex() {
        assert_eq!(parse_hex("00"), Ok(("", vec![0])));
        assert_eq!(parse_hex("00ff"), Ok(("", vec![0, 255])));
        assert_eq!(parse_hex("00ff aa"), Ok(("", vec![0, 255, 170])));
        assert_eq!(
            parse_hex(" 00ff  aa 11 \n"),
            Ok(("", vec![0, 255, 170, 17]))
        );
        assert_eq!(parse_hex(" 00ff  aa 11\n"), Ok(("", vec![0, 255, 170, 17])));
        assert!(parse_hex("00ff  aa 11zz").is_err());
        assert!(parse_hex("00ff  aa 11 zz").is_err());
    }

    #[test]
    fn test_parse_hex_with_comment_hyphen() {
        assert_eq!(parse_hex("00 -- 01"), Ok(("", vec![0])));
        assert_eq!(parse_hex("00 -- 01\n"), Ok(("", vec![0])));
        assert_eq!(
            parse_hex("00 -- 00\n01 02\n--00\n 03"),
            Ok(("", vec![0, 1, 2, 3]))
        );
    }

    #[test]
    fn test_parse_hex_with_comment_hash() {
        assert_eq!(parse_hex("00 # 01"), Ok(("", vec![0])));
        assert_eq!(parse_hex("00 # 01\n"), Ok(("", vec![0])));
        assert_eq!(
            parse_hex("00 # 00\n01 02\n#00\n 03"),
            Ok(("", vec![0, 1, 2, 3]))
        );
    }

    #[test]
    fn test_parse_hex_incomplete() {
        assert!(parse_hex("00f").is_err());
        assert!(parse_hex("00f -").is_err());
        assert!(parse_hex("00f -- \nf").is_err());
    }
}
