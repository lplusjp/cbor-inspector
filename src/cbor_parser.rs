use bstr::ByteSlice as _;
use half::f16;
use nom::bytes::complete::take;
use nom::combinator::{eof, fail};
use nom::IResult;

use crate::tree::Node;

const MAJOR_UNSIGNED_INTEGER: u8 = 0;
const MAJOR_NEGATIVE_INTEGER: u8 = 1;
const MAJOR_BYTE_STRING: u8 = 2;
const MAJOR_TEXT_STRING: u8 = 3;
const MAJOR_ARRAY: u8 = 4;
const MAJOR_MAP: u8 = 5;
const MAJOR_TAG: u8 = 6;
const MAJOR_SIMPLE_OR_FLOAT: u8 = 7;

const ADDITIONAL_INFO_VALUE_FOLLOWED_BY_1_BYTE: u8 = 24;
const ADDITIONAL_INFO_VALUE_FOLLOWED_BY_2_BYTES: u8 = 25;
const ADDITIONAL_INFO_VALUE_FOLLOWED_BY_4_BYTES: u8 = 26;
const ADDITIONAL_INFO_VALUE_FOLLOWED_BY_8_BYTES: u8 = 27;
const ADDITIONAL_INFO_VALUE_FOLLOWED_BY_INDEFINITE_BYTES: u8 = 31;

const SIMPLE_OR_FLOAT_SIMPLE_BEGIN: u8 = 0;
const SIMPLE_OR_FLOAT_SIMPLE_END: u8 = 23;
const SIMPLE_OR_FLOAT_SIMPLE_FOLLOWS: u8 = 24;
const SIMPLE_OR_FLOAT_FLOAT_HALF: u8 = 25;
const SIMPLE_OR_FLOAT_FLOAT_SINGLE: u8 = 26;
const SIMPLE_OR_FLOAT_FLOAT_DOUBLE: u8 = 27;
const SIMPLE_OR_FLOAT_BREAK: u8 = 31;

const SIMPLE_VALUE_FALSE: u8 = 20;
const SIMPLE_VALUE_TRUE: u8 = 21;
const SIMPLE_VALUE_NULL: u8 = 22;
const SIMPLE_VALUE_UNDEFINED: u8 = 23;

enum AdditionalInfoValue {
    Value(u64),
    Reserved,
    Indefinite,
}

fn parse_additional_info_value(
    argument: u8,
) -> impl Fn(&[u8]) -> IResult<&[u8], (AdditionalInfoValue, &[u8])> {
    move |input: &[u8]| {
        let more_bytes_length = match argument {
            0..ADDITIONAL_INFO_VALUE_FOLLOWED_BY_1_BYTE => 0usize,
            ADDITIONAL_INFO_VALUE_FOLLOWED_BY_1_BYTE => 1usize,
            ADDITIONAL_INFO_VALUE_FOLLOWED_BY_2_BYTES => 2usize,
            ADDITIONAL_INFO_VALUE_FOLLOWED_BY_4_BYTES => 4usize,
            ADDITIONAL_INFO_VALUE_FOLLOWED_BY_8_BYTES => 8usize,
            ADDITIONAL_INFO_VALUE_FOLLOWED_BY_INDEFINITE_BYTES => 0usize,
            _ => 0usize, // Reserved
        };
        let (input, more_bytes) = take(more_bytes_length)(input)?;
        let additional_info_value = match argument {
            0..ADDITIONAL_INFO_VALUE_FOLLOWED_BY_1_BYTE => {
                AdditionalInfoValue::Value(argument as u64)
            }
            ADDITIONAL_INFO_VALUE_FOLLOWED_BY_1_BYTE => {
                AdditionalInfoValue::Value(more_bytes[0] as u64)
            }
            ADDITIONAL_INFO_VALUE_FOLLOWED_BY_2_BYTES => {
                // unwrap safety: more_bytes is 2 bytes
                AdditionalInfoValue::Value(u16::from_be_bytes(more_bytes.try_into().unwrap()) as u64)
            }
            ADDITIONAL_INFO_VALUE_FOLLOWED_BY_4_BYTES => {
                // unwrap safety: more_bytes is 4 bytes
                AdditionalInfoValue::Value(u32::from_be_bytes(more_bytes.try_into().unwrap()) as u64)
            }
            ADDITIONAL_INFO_VALUE_FOLLOWED_BY_8_BYTES => {
                // unwrap safety: more_bytes is 8 bytes
                AdditionalInfoValue::Value(u64::from_be_bytes(more_bytes.try_into().unwrap()))
            }
            ADDITIONAL_INFO_VALUE_FOLLOWED_BY_INDEFINITE_BYTES => AdditionalInfoValue::Indefinite,
            _ => AdditionalInfoValue::Reserved,
        };
        Ok((input, (additional_info_value, more_bytes)))
    }
}

fn split_major_type_and_additional_info(b: u8) -> (u8, u8) {
    (b >> 5, b & 0b00011111)
}

fn unsigned_or_negative_integer(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], Node> {
    move |input: &[u8]| {
        let (major_type, additional_info_argument) = split_major_type_and_additional_info(b);
        if major_type != MAJOR_UNSIGNED_INTEGER && major_type != MAJOR_NEGATIVE_INTEGER {
            return fail(input);
        }

        let (input, (value, more_bytes)) =
            parse_additional_info_value(additional_info_argument)(input)?;
        let value = match value {
            AdditionalInfoValue::Value(length) => Some(length),
            _ => None,
        };
        let (input, actual_value) = match major_type {
            MAJOR_UNSIGNED_INTEGER => (input, value.map(|v| v as i64)),
            MAJOR_NEGATIVE_INTEGER => (input, value.map(|v| -(v as i64) - 1)),
            // unreachable safety: when major_type is not unsigned or negative, already failed
            _ => unreachable!(),
        };
        let comment = format!(
            "{}({}) = {}",
            match major_type {
                MAJOR_UNSIGNED_INTEGER => "unsigned",
                MAJOR_NEGATIVE_INTEGER => "negative",
                // unreachable safety: when major_type is not unsigned or negative, already failed
                _ => unreachable!(),
            },
            value
                .map(|v| format!("{:#x}", v))
                .unwrap_or("?".to_string()),
            actual_value
                .map(|v| v.to_string())
                .unwrap_or("?".to_string())
        );
        Ok((
            input,
            Node::new(vec![b])
                .with_more_bytes(more_bytes.to_owned())
                .with_comment(comment),
        ))
    }
}

fn unsigned_integer(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], Node> {
    unsigned_or_negative_integer(b)
}

fn negative_integer(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], Node> {
    unsigned_or_negative_integer(b)
}

fn embedded_cbor_object(input: &[u8], length: usize) -> IResult<&[u8], Node> {
    let (input, cbor_input) = take(length)(input)?;
    let (rest, cbor_tree) = cbor_object(cbor_input)?;
    eof(rest)?;
    Ok((input, cbor_tree.mark_embedded()))
}

fn byte_string_or_text_string(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], Node> {
    move |input: &[u8]| {
        let (major_type, additional_info_argument) = split_major_type_and_additional_info(b);
        if major_type != MAJOR_BYTE_STRING && major_type != MAJOR_TEXT_STRING {
            return fail(input);
        }

        let (input, (length, more_bytes)) =
            parse_additional_info_value(additional_info_argument)(input)?;
        let comment = format!(
            "{}({})",
            match major_type {
                MAJOR_BYTE_STRING => "bstr",
                MAJOR_TEXT_STRING => "tstr",
                // unreachable safety: when major_type is not byte string or text string, already failed
                _ => unreachable!(),
            },
            match length {
                AdditionalInfoValue::Value(length) => length.to_string(),
                AdditionalInfoValue::Reserved => "?".to_string(),
                AdditionalInfoValue::Indefinite => "*".to_string(),
            }
        );

        // Embedded CBOR object
        if major_type == MAJOR_BYTE_STRING {
            // TODO: AdditionalInfoValue::Indefinite
            if let AdditionalInfoValue::Value(length) = length {
                if let Ok((input, embedded_object)) = embedded_cbor_object(input, length as usize) {
                    return Ok((
                        input,
                        Node::new(vec![b])
                            .with_more_bytes(more_bytes.to_owned())
                            .with_comment(comment)
                            .with_child(embedded_object),
                    ));
                }
            }
        }

        let mut input = input;
        let payload_objects = match length {
            AdditionalInfoValue::Value(_) | AdditionalInfoValue::Reserved => {
                let (input_new, payload) = match length {
                    AdditionalInfoValue::Value(length) => take(length as usize)(input)?,
                    AdditionalInfoValue::Reserved => take(0usize)(input)?,
                    // unreachable safety: In this branch, length is always Value or Reserved
                    _ => unreachable!(),
                };
                input = input_new;
                let payload_comment = match major_type {
                    MAJOR_BYTE_STRING => format!("\"{}\"", payload.escape_bytes()),
                    MAJOR_TEXT_STRING => format!("{:?}", payload.as_bstr().to_str_lossy()),
                    // unreachable safety: when major_type is not byte string or text string, already failed
                    _ => unreachable!(),
                };
                vec![Node::new(payload.to_owned()).with_comment(payload_comment)]
            }
            AdditionalInfoValue::Indefinite => {
                let mut children = Vec::new();
                loop {
                    let (input_new, child) = cbor_object(input)?;
                    input = input_new;
                    let is_break = child.bytes() == [0xff];
                    children.push(child);
                    if is_break {
                        break;
                    }
                }
                children
            }
        };

        Ok((
            input,
            Node::new(vec![b])
                .with_more_bytes(more_bytes.to_owned())
                .with_comment(comment)
                .with_children(payload_objects),
        ))
    }
}

fn byte_string(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], Node> {
    byte_string_or_text_string(b)
}

fn text_string(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], Node> {
    byte_string_or_text_string(b)
}

fn array_or_map(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], Node> {
    move |input: &[u8]| {
        let (major_type, additional_info_argument) = split_major_type_and_additional_info(b);
        if major_type != MAJOR_ARRAY && major_type != MAJOR_MAP {
            return fail(input);
        }

        let (input, (length, more_bytes)) =
            parse_additional_info_value(additional_info_argument)(input)?;
        let mut input = input;
        let comment = format!(
            "{}({})",
            match major_type {
                MAJOR_ARRAY => "array",
                MAJOR_MAP => "map",
                // unreachable safety: when major_type is not array or map, already failed
                _ => unreachable!(),
            },
            match length {
                AdditionalInfoValue::Value(length) => format!("{:#x} = {}", length, length),
                AdditionalInfoValue::Reserved => "?".to_string(),
                AdditionalInfoValue::Indefinite => "*".to_string(),
            }
        );
        let node = match length {
            AdditionalInfoValue::Value(length) => {
                let mut children = Vec::new();
                for _ in 0..length {
                    let (input_new, child) = cbor_object(input)?;
                    input = input_new;
                    children.push(child);

                    if major_type == MAJOR_MAP {
                        let (input_new, child) = cbor_object(input)?;
                        input = input_new;
                        children.push(child);
                    }
                }
                Node::new(vec![b])
                    .with_more_bytes(more_bytes.to_owned())
                    .with_comment(comment)
                    .with_children(children)
            }
            AdditionalInfoValue::Reserved => Node::new(vec![b])
                .with_more_bytes(more_bytes.to_owned())
                .with_comment(comment),
            AdditionalInfoValue::Indefinite => {
                let mut children = Vec::new();
                loop {
                    let (input_new, child) = cbor_object(input)?;
                    input = input_new;
                    let is_break = child.bytes() == [0xff];
                    children.push(child);
                    if is_break {
                        break;
                    }
                }
                Node::new(vec![b])
                    .with_comment(comment)
                    .with_children(children)
            }
        };
        Ok((input, node))
    }
}

fn array(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], Node> {
    array_or_map(b)
}

fn map(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], Node> {
    array_or_map(b)
}

fn tag(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], Node> {
    move |input: &[u8]| {
        let (major_type, additional_info_argument) = split_major_type_and_additional_info(b);
        if major_type != MAJOR_TAG {
            return fail(input);
        }

        let (input, (tag, more_bytes)) =
            parse_additional_info_value(additional_info_argument)(input)?;

        let (input, child) = cbor_object(input)?;
        let comment = match tag {
            AdditionalInfoValue::Value(tag) => format!("tag({:#x} = {})", tag, tag),
            AdditionalInfoValue::Reserved | AdditionalInfoValue::Indefinite => "tag(?)".to_string(),
        };
        Ok((
            input,
            Node::new(vec![b])
                .with_more_bytes(more_bytes.to_owned())
                .with_comment(comment)
                .with_child(child),
        ))
    }
}

enum SimpleOrFloat {
    Simple(u8),
    FloatHalf(f16),
    FloatSingle(f32),
    FloatDouble(f64),
    Reserved,
    Break,
}

fn parse_simple_or_float_value(
    additional_info_argument: u8,
) -> impl Fn(&[u8]) -> IResult<&[u8], (SimpleOrFloat, &[u8])> {
    move |input: &[u8]| {
        let mut input = input;
        let (simple_or_float, more_bytes): (_, &[u8]) = match additional_info_argument {
            SIMPLE_OR_FLOAT_SIMPLE_BEGIN..=SIMPLE_OR_FLOAT_SIMPLE_END => {
                (SimpleOrFloat::Simple(additional_info_argument), &[])
            }
            SIMPLE_OR_FLOAT_SIMPLE_FOLLOWS => {
                let (input_new, simple_value) = take(1usize)(input)?;
                input = input_new;
                (SimpleOrFloat::Simple(simple_value[0]), simple_value)
            }
            SIMPLE_OR_FLOAT_FLOAT_HALF => {
                let (input_new, float_half) = take(2usize)(input)?;
                input = input_new;
                // unwrap safety: float_half is 2 bytes
                (
                    SimpleOrFloat::FloatHalf(f16::from_be_bytes(float_half.try_into().unwrap())),
                    float_half,
                )
            }
            SIMPLE_OR_FLOAT_FLOAT_SINGLE => {
                let (input_new, float_single) = take(4usize)(input)?;
                input = input_new;
                // unwrap safety: float_single is 4 bytes
                (
                    SimpleOrFloat::FloatSingle(f32::from_be_bytes(
                        float_single.try_into().unwrap(),
                    )),
                    float_single,
                )
            }
            SIMPLE_OR_FLOAT_FLOAT_DOUBLE => {
                let (input_new, float_double) = take(8usize)(input)?;
                input = input_new;
                // unwrap safety: float_double is 8 bytes
                (
                    SimpleOrFloat::FloatDouble(f64::from_be_bytes(
                        float_double.try_into().unwrap(),
                    )),
                    float_double,
                )
            }
            SIMPLE_OR_FLOAT_BREAK => (SimpleOrFloat::Break, &[]),
            _ => (SimpleOrFloat::Reserved, &[]),
        };
        Ok((input, (simple_or_float, more_bytes)))
    }
}

fn simple_or_float(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], Node> {
    move |input: &[u8]| {
        let (major_type, additional_info_argument) = split_major_type_and_additional_info(b);
        if major_type != MAJOR_SIMPLE_OR_FLOAT {
            return fail(input);
        }

        let (input, (simple_or_float_value, more_bytes)) =
            parse_simple_or_float_value(additional_info_argument)(input)?;
        let comment = match simple_or_float_value {
            SimpleOrFloat::Simple(simple_value) => format!(
                "simple({:#x} = {}) = {}",
                simple_value,
                simple_value,
                match simple_value {
                    SIMPLE_VALUE_FALSE => "false",
                    SIMPLE_VALUE_TRUE => "true",
                    SIMPLE_VALUE_NULL => "null",
                    SIMPLE_VALUE_UNDEFINED => "undefined",
                    _ => "?",
                }
            ),
            SimpleOrFloat::FloatHalf(float_half) => format!("float16({:.1e})", float_half),
            SimpleOrFloat::FloatSingle(float_single) => format!("float32({:.1e})", float_single),
            SimpleOrFloat::FloatDouble(float_double) => format!("float64({:.1e})", float_double),
            SimpleOrFloat::Reserved => "reserved simple/float".to_string(),
            SimpleOrFloat::Break => "break".to_string(),
        };
        Ok((
            input,
            Node::new(vec![b])
                .with_more_bytes(more_bytes.to_owned())
                .with_comment(comment),
        ))
    }
}

fn cbor_object(orig_input: &[u8]) -> IResult<&[u8], Node> {
    let (input, b) = take(1usize)(orig_input)?;
    let (major_type, _additional_info_argument) = split_major_type_and_additional_info(b[0]);
    match major_type {
        MAJOR_UNSIGNED_INTEGER => unsigned_integer(b[0])(input),
        MAJOR_NEGATIVE_INTEGER => negative_integer(b[0])(input),
        MAJOR_BYTE_STRING => byte_string(b[0])(input),
        MAJOR_TEXT_STRING => text_string(b[0])(input),
        MAJOR_ARRAY => array(b[0])(input),
        MAJOR_MAP => map(b[0])(input),
        MAJOR_TAG => tag(b[0])(input),
        MAJOR_SIMPLE_OR_FLOAT => simple_or_float(b[0])(input),
        // unreachable safety: major_type is always in 0..=7
        _ => unreachable!(),
    }
}

pub fn parse_cbor(input: &[u8]) -> IResult<&[u8], Node> {
    cbor_object(input)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use pretty_assertions::assert_eq;

    use crate::tree::Node;

    #[test]
    fn parse_unsigned_integer_short() -> Result<()> {
        let input = b"\x01\x00";
        let expected = Node::new(vec![0x01]).with_comment("unsigned(0x1) = 1".to_string());
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_unsigned_integer_1_byte() -> Result<()> {
        let input = b"\x18\x03\x00";
        let expected = Node::new(vec![0x18])
            .with_more_bytes(vec![0x03])
            .with_comment("unsigned(0x3) = 3".to_string());
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_unsigned_integer_2_bytes() -> Result<()> {
        let input = b"\x19\x00\x03\x00";
        let expected = Node::new(vec![0x19])
            .with_more_bytes(vec![0x00, 0x03])
            .with_comment("unsigned(0x3) = 3".to_string());
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_unsigned_integer_4_bytes() -> Result<()> {
        let input = b"\x1a\x00\x00\x00\x03\x00";
        let expected = Node::new(vec![0x1a])
            .with_more_bytes(vec![0x00, 0x00, 0x00, 0x03])
            .with_comment("unsigned(0x3) = 3".to_string());
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_unsigned_integer_8_bytes() -> Result<()> {
        let input = b"\x1b\x01\x02\x03\x04\x05\x06\x07\x08\x00";
        let expected_inner_value = 0x0102030405060708u64;
        let expected_value = 0x0102030405060708u64;
        let expected = Node::new(vec![0x1b])
            .with_more_bytes(vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08])
            .with_comment(format!(
                "{}({:#x}) = {}",
                "unsigned", expected_inner_value, expected_value,
            ));
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_negative_integer_short() -> Result<()> {
        let input = b"\x20\x00";
        let expected = Node::new(vec![0x20]).with_comment("negative(0x0) = -1".to_string());
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_negative_integer_1_byte() -> Result<()> {
        let input = b"\x38\xff\x00";
        let expected_inner_value = 255u64;
        let expected_value = -256i64;
        let expected = Node::new(vec![0x38])
            .with_more_bytes(vec![0xff])
            .with_comment(format!(
                "{}({:#x}) = {}",
                "negative", expected_inner_value, expected_value
            ));
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_negative_integer_2_bytes() -> Result<()> {
        let input = b"\x39\xff\xff\x00";
        let expected_inner_value = 0xffffu64;
        let expected_value = -0x10000i64;
        let expected = Node::new(vec![0x39])
            .with_more_bytes(vec![0xff, 0xff])
            .with_comment(format!(
                "{}({:#x}) = {}",
                "negative", expected_inner_value, expected_value
            ));
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_negative_integer_4_bytes() -> Result<()> {
        let input = b"\x3a\xff\xff\xff\xff\x00";
        let expected_inner_value = 0xffffffffu64;
        let expected_value = -0x100000000i64;
        let expected = Node::new(vec![0x3a])
            .with_more_bytes(vec![0xff, 0xff, 0xff, 0xff])
            .with_comment(format!(
                "{}({:#x}) = {}",
                "negative", expected_inner_value, expected_value
            ));
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_negative_integer_8_bytes() -> Result<()> {
        let input = b"\x3b\x7f\xff\xff\xff\xff\xff\xff\xff\x00";
        let expected_inner_value = 0x7fffffffffffffffu64;
        let expected_value = -0x8000000000000000i64;
        let expected = Node::new(vec![0x3b])
            .with_more_bytes(vec![0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff])
            .with_comment(format!(
                "{}({:#x}) = {}",
                "negative", expected_inner_value, expected_value
            ));
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_byte_string_short() -> Result<()> {
        let input = b"\x43\x01\x02\x03\x00";
        let expected = Node::new(vec![0x43])
            .with_comment("bstr(3)".to_string())
            .with_child(
                Node::new(vec![0x01, 0x02, 0x03]).with_comment("\"\\x01\\x02\\x03\"".to_string()),
            );
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_byte_string_more_bytes_1() -> Result<()> {
        let input = b"\x58\x03\x01\x02\x03\x00";
        let expected = Node::new(vec![0x58])
            .with_more_bytes(vec![0x03])
            .with_comment("bstr(3)".to_string())
            .with_child(
                Node::new(vec![0x01, 0x02, 0x03]).with_comment("\"\\x01\\x02\\x03\"".to_string()),
            );
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_byte_string_more_bytes_2() -> Result<()> {
        let input = b"\x59\x00\x03\x01\x02\x03\x00";
        let expected = Node::new(vec![0x59])
            .with_more_bytes(vec![0x00, 0x03])
            .with_comment("bstr(3)".to_string())
            .with_child(
                Node::new(vec![0x01, 0x02, 0x03]).with_comment("\"\\x01\\x02\\x03\"".to_string()),
            );
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_byte_string_more_bytes_4() -> Result<()> {
        let input = b"\x5a\x00\x00\x00\x03\x01\x02\x03\x00";
        let expected = Node::new(vec![0x5a])
            .with_more_bytes(vec![0x00, 0x00, 0x00, 0x03])
            .with_comment("bstr(3)".to_string())
            .with_child(
                Node::new(vec![0x01, 0x02, 0x03]).with_comment("\"\\x01\\x02\\x03\"".to_string()),
            );
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_byte_string_more_bytes_8() -> Result<()> {
        let input = b"\x5b\x00\x00\x00\x00\x00\x00\x00\x03\x01\x02\x03\x00";
        let expected = Node::new(vec![0x5b])
            .with_more_bytes(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03])
            .with_comment("bstr(3)".to_string())
            .with_child(
                Node::new(vec![0x01, 0x02, 0x03]).with_comment("\"\\x01\\x02\\x03\"".to_string()),
            );
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_byte_string_indefinite() -> Result<()> {
        let input = b"\x5f\x43\x01\x02\x03\xff\x00";
        let expected = Node::new(vec![0x5f])
            .with_comment("bstr(*)".to_string())
            .with_children(vec![
                Node::new(vec![0x43])
                    .with_comment("bstr(3)".to_string())
                    .with_child(
                        Node::new(vec![0x01, 0x02, 0x03])
                            .with_comment("\"\\x01\\x02\\x03\"".to_string()),
                    ),
                Node::new(vec![0xff]).with_comment("break".to_string()),
            ]);
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_byte_string_embedded() -> Result<()> {
        let input = b"\x46\xb8\x02\x01\x02\x03\x04\x00";
        let expected = Node::new(vec![0x46])
            .with_comment("bstr(6)".to_string())
            .with_child(
                Node::new(vec![0xb8])
                    .with_more_bytes(vec![0x02])
                    .with_comment("map(0x2 = 2)".to_string())
                    .with_children(vec![
                        Node::new(vec![0x01]).with_comment("unsigned(0x1) = 1".to_string()),
                        Node::new(vec![0x02]).with_comment("unsigned(0x2) = 2".to_string()),
                        Node::new(vec![0x03]).with_comment("unsigned(0x3) = 3".to_string()),
                        Node::new(vec![0x04]).with_comment("unsigned(0x4) = 4".to_string()),
                    ])
                    .mark_embedded(),
            );
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_text_string_short() -> Result<()> {
        let input = b"\x63\x61\x62\x63\x00";
        let expected = Node::new(vec![0x63])
            .with_comment("tstr(3)".to_string())
            .with_child(Node::new(vec![0x61, 0x62, 0x63]).with_comment("\"abc\"".to_string()));
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_text_string_more_bytes_1() -> Result<()> {
        let input = b"\x78\x03\x61\x62\x63\x00";
        let expected = Node::new(vec![0x78])
            .with_more_bytes(vec![0x03])
            .with_comment("tstr(3)".to_string())
            .with_child(Node::new(vec![0x61, 0x62, 0x63]).with_comment("\"abc\"".to_string()));
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_text_string_more_bytes_2() -> Result<()> {
        let input = b"\x79\x00\x03\x61\x62\x63\x00";
        let expected = Node::new(vec![0x79])
            .with_more_bytes(vec![0x00, 0x03])
            .with_comment("tstr(3)".to_string())
            .with_child(Node::new(vec![0x61, 0x62, 0x63]).with_comment("\"abc\"".to_string()));
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_text_string_more_bytes_4() -> Result<()> {
        let input = b"\x7a\x00\x00\x00\x03\x61\x62\x63\x00";
        let expected = Node::new(vec![0x7a])
            .with_more_bytes(vec![0x00, 0x00, 0x00, 0x03])
            .with_comment("tstr(3)".to_string())
            .with_child(Node::new(vec![0x61, 0x62, 0x63]).with_comment("\"abc\"".to_string()));
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_text_string_more_bytes_8() -> Result<()> {
        let input = b"\x7b\x00\x00\x00\x00\x00\x00\x00\x03\x61\x62\x63\x00";
        let expected = Node::new(vec![0x7b])
            .with_more_bytes(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03])
            .with_comment("tstr(3)".to_string())
            .with_child(Node::new(vec![0x61, 0x62, 0x63]).with_comment("\"abc\"".to_string()));
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_text_string_indefinite() -> Result<()> {
        let input = b"\x7f\x63\x61\x62\x63\xff\x00";
        let expected = Node::new(vec![0x7f])
            .with_comment("tstr(*)".to_string())
            .with_children(vec![
                Node::new(vec![0x63])
                    .with_comment("tstr(3)".to_string())
                    .with_child(
                        Node::new(vec![0x61, 0x62, 0x63]).with_comment("\"abc\"".to_string()),
                    ),
                Node::new(vec![0xff]).with_comment("break".to_string()),
            ]);
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_array_short() -> Result<()> {
        let input = b"\x83\x01\x02\x03\x00";
        let expected = Node::new(vec![0x83])
            .with_comment("array(0x3 = 3)".to_string())
            .with_children(vec![
                Node::new(vec![0x01]).with_comment("unsigned(0x1) = 1".to_string()),
                Node::new(vec![0x02]).with_comment("unsigned(0x2) = 2".to_string()),
                Node::new(vec![0x03]).with_comment("unsigned(0x3) = 3".to_string()),
            ]);
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_array_more_bytes_1() -> Result<()> {
        let input = b"\x98\x03\x01\x02\x03\x00";
        let expected = Node::new(vec![0x98])
            .with_more_bytes(vec![0x03])
            .with_comment("array(0x3 = 3)".to_string())
            .with_children(vec![
                Node::new(vec![0x01]).with_comment("unsigned(0x1) = 1".to_string()),
                Node::new(vec![0x02]).with_comment("unsigned(0x2) = 2".to_string()),
                Node::new(vec![0x03]).with_comment("unsigned(0x3) = 3".to_string()),
            ]);
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_array_more_bytes_2() -> Result<()> {
        let input = b"\x99\x00\x03\x01\x02\x03\x00";
        let expected = Node::new(vec![0x99])
            .with_more_bytes(vec![0x00, 0x03])
            .with_comment("array(0x3 = 3)".to_string())
            .with_children(vec![
                Node::new(vec![0x01]).with_comment("unsigned(0x1) = 1".to_string()),
                Node::new(vec![0x02]).with_comment("unsigned(0x2) = 2".to_string()),
                Node::new(vec![0x03]).with_comment("unsigned(0x3) = 3".to_string()),
            ]);
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_array_more_bytes_4() -> Result<()> {
        let input = b"\x9a\x00\x00\x00\x03\x01\x02\x03\x00";
        let expected = Node::new(vec![0x9a])
            .with_more_bytes(vec![0x00, 0x00, 0x00, 0x03])
            .with_comment("array(0x3 = 3)".to_string())
            .with_children(vec![
                Node::new(vec![0x01]).with_comment("unsigned(0x1) = 1".to_string()),
                Node::new(vec![0x02]).with_comment("unsigned(0x2) = 2".to_string()),
                Node::new(vec![0x03]).with_comment("unsigned(0x3) = 3".to_string()),
            ]);
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_array_more_bytes_8() -> Result<()> {
        let input = b"\x9b\x00\x00\x00\x00\x00\x00\x00\x03\x01\x02\x03\x00";
        let expected = Node::new(vec![0x9b])
            .with_more_bytes(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03])
            .with_comment("array(0x3 = 3)".to_string())
            .with_children(vec![
                Node::new(vec![0x01]).with_comment("unsigned(0x1) = 1".to_string()),
                Node::new(vec![0x02]).with_comment("unsigned(0x2) = 2".to_string()),
                Node::new(vec![0x03]).with_comment("unsigned(0x3) = 3".to_string()),
            ]);
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_array_indefinite() -> Result<()> {
        let input = b"\x9f\x01\x02\x03\xff\x00";
        let expected = Node::new(vec![0x9f])
            .with_comment("array(*)".to_string())
            .with_children(vec![
                Node::new(vec![0x01]).with_comment("unsigned(0x1) = 1".to_string()),
                Node::new(vec![0x02]).with_comment("unsigned(0x2) = 2".to_string()),
                Node::new(vec![0x03]).with_comment("unsigned(0x3) = 3".to_string()),
                Node::new(vec![0xff]).with_comment("break".to_string()),
            ]);
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_map_short() -> Result<()> {
        let input = b"\xa2\x01\x02\x03\x04\x00";
        let expected = Node::new(vec![0xa2])
            .with_comment("map(0x2 = 2)".to_string())
            .with_children(vec![
                Node::new(vec![0x01]).with_comment("unsigned(0x1) = 1".to_string()),
                Node::new(vec![0x02]).with_comment("unsigned(0x2) = 2".to_string()),
                Node::new(vec![0x03]).with_comment("unsigned(0x3) = 3".to_string()),
                Node::new(vec![0x04]).with_comment("unsigned(0x4) = 4".to_string()),
            ]);
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_map_more_bytes_1() -> Result<()> {
        let input = b"\xb8\x02\x01\x02\x03\x04\x00";
        let expected = Node::new(vec![0xb8])
            .with_more_bytes(vec![0x02])
            .with_comment("map(0x2 = 2)".to_string())
            .with_children(vec![
                Node::new(vec![0x01]).with_comment("unsigned(0x1) = 1".to_string()),
                Node::new(vec![0x02]).with_comment("unsigned(0x2) = 2".to_string()),
                Node::new(vec![0x03]).with_comment("unsigned(0x3) = 3".to_string()),
                Node::new(vec![0x04]).with_comment("unsigned(0x4) = 4".to_string()),
            ]);
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_map_more_bytes_2() -> Result<()> {
        let input = b"\xb9\x00\x02\x01\x02\x03\x04\x00";
        let expected = Node::new(vec![0xb9])
            .with_more_bytes(vec![0x00, 0x02])
            .with_comment("map(0x2 = 2)".to_string())
            .with_children(vec![
                Node::new(vec![0x01]).with_comment("unsigned(0x1) = 1".to_string()),
                Node::new(vec![0x02]).with_comment("unsigned(0x2) = 2".to_string()),
                Node::new(vec![0x03]).with_comment("unsigned(0x3) = 3".to_string()),
                Node::new(vec![0x04]).with_comment("unsigned(0x4) = 4".to_string()),
            ]);
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_map_more_bytes_4() -> Result<()> {
        let input = b"\xba\x00\x00\x00\x02\x01\x02\x03\x04\x00";
        let expected = Node::new(vec![0xba])
            .with_more_bytes(vec![0x00, 0x00, 0x00, 0x02])
            .with_comment("map(0x2 = 2)".to_string())
            .with_children(vec![
                Node::new(vec![0x01]).with_comment("unsigned(0x1) = 1".to_string()),
                Node::new(vec![0x02]).with_comment("unsigned(0x2) = 2".to_string()),
                Node::new(vec![0x03]).with_comment("unsigned(0x3) = 3".to_string()),
                Node::new(vec![0x04]).with_comment("unsigned(0x4) = 4".to_string()),
            ]);
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_map_more_bytes_8() -> Result<()> {
        let input = b"\xbb\x00\x00\x00\x00\x00\x00\x00\x02\x01\x02\x03\x04\x00";
        let expected = Node::new(vec![0xbb])
            .with_more_bytes(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02])
            .with_comment("map(0x2 = 2)".to_string())
            .with_children(vec![
                Node::new(vec![0x01]).with_comment("unsigned(0x1) = 1".to_string()),
                Node::new(vec![0x02]).with_comment("unsigned(0x2) = 2".to_string()),
                Node::new(vec![0x03]).with_comment("unsigned(0x3) = 3".to_string()),
                Node::new(vec![0x04]).with_comment("unsigned(0x4) = 4".to_string()),
            ]);
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_map_indefinite() -> Result<()> {
        let input = b"\xbf\x01\x02\x03\x04\xff\x00";
        let expected = Node::new(vec![0xbf])
            .with_comment("map(*)".to_string())
            .with_children(vec![
                Node::new(vec![0x01]).with_comment("unsigned(0x1) = 1".to_string()),
                Node::new(vec![0x02]).with_comment("unsigned(0x2) = 2".to_string()),
                Node::new(vec![0x03]).with_comment("unsigned(0x3) = 3".to_string()),
                Node::new(vec![0x04]).with_comment("unsigned(0x4) = 4".to_string()),
                Node::new(vec![0xff]).with_comment("break".to_string()),
            ]);
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_simple_short() -> Result<()> {
        let input = b"\xe0\x00";
        let expected = Node::new(vec![0xe0]).with_comment("simple(0x0 = 0) = ?".to_string());
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_simple_long() -> Result<()> {
        let input = b"\xf8\xff\x00";
        let expected = Node::new(vec![0xf8])
            .with_more_bytes(vec![0xff])
            .with_comment("simple(0xff = 255) = ?".to_string());
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_false() -> Result<()> {
        let input = b"\xf4\x00";
        let expected = Node::new(vec![0xf4]).with_comment("simple(0x14 = 20) = false".to_string());
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_true() -> Result<()> {
        let input = b"\xf5\x00";
        let expected = Node::new(vec![0xf5]).with_comment("simple(0x15 = 21) = true".to_string());
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_null() -> Result<()> {
        let input = b"\xf6\x00";
        let expected = Node::new(vec![0xf6]).with_comment("simple(0x16 = 22) = null".to_string());
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_undefined() -> Result<()> {
        let input = b"\xf7\x00";
        let expected =
            Node::new(vec![0xf7]).with_comment("simple(0x17 = 23) = undefined".to_string());
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_float_half() -> Result<()> {
        let input = b"\xf9\x3c\x00\x00";
        let expected = Node::new(vec![0xf9])
            .with_more_bytes(vec![0x3c, 0x00])
            .with_comment("float16(1e0)".to_string());
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_float_single() -> Result<()> {
        let input = b"\xfa\x47\xc3\x50\x00\x00";
        let expected = Node::new(vec![0xfa])
            .with_more_bytes(vec![0x47, 0xc3, 0x50, 0x00])
            .with_comment("float32(1.0e5)".to_string());
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_float_double() -> Result<()> {
        let input = b"\xfb\x7e\x37\xe4\x3c\x88\x00\x75\x9c\x00";
        let expected = Node::new(vec![0xfb])
            .with_more_bytes(vec![0x7e, 0x37, 0xe4, 0x3c, 0x88, 0x00, 0x75, 0x9c])
            .with_comment("float64(1.0e300)".to_string());
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_tag_short() -> Result<()> {
        let input = b"\xc1\xb8\x02\x01\x02\x03\x04\x00";
        let expected = Node::new(vec![0xc1])
            .with_comment("tag(0x1 = 1)".to_string())
            .with_child(
                Node::new(vec![0xb8])
                    .with_more_bytes(vec![0x02])
                    .with_comment("map(0x2 = 2)".to_string())
                    .with_children(vec![
                        Node::new(vec![0x01]).with_comment("unsigned(0x1) = 1".to_string()),
                        Node::new(vec![0x02]).with_comment("unsigned(0x2) = 2".to_string()),
                        Node::new(vec![0x03]).with_comment("unsigned(0x3) = 3".to_string()),
                        Node::new(vec![0x04]).with_comment("unsigned(0x4) = 4".to_string()),
                    ]),
            );
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_tag_more_bytes_1() -> Result<()> {
        let input = b"\xd8\xff\xb8\x02\x01\x02\x03\x04\x00";
        let expected = Node::new(vec![0xd8])
            .with_more_bytes(vec![0xff])
            .with_comment("tag(0xff = 255)".to_string())
            .with_child(
                Node::new(vec![0xb8])
                    .with_more_bytes(vec![0x02])
                    .with_comment("map(0x2 = 2)".to_string())
                    .with_children(vec![
                        Node::new(vec![0x01]).with_comment("unsigned(0x1) = 1".to_string()),
                        Node::new(vec![0x02]).with_comment("unsigned(0x2) = 2".to_string()),
                        Node::new(vec![0x03]).with_comment("unsigned(0x3) = 3".to_string()),
                        Node::new(vec![0x04]).with_comment("unsigned(0x4) = 4".to_string()),
                    ]),
            );
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_tag_more_bytes_2() -> Result<()> {
        let input = b"\xd9\xee\xff\xb8\x02\x01\x02\x03\x04\x00";
        let expected = Node::new(vec![0xd9])
            .with_more_bytes(vec![0xee, 0xff])
            .with_comment("tag(0xeeff = 61183)".to_string())
            .with_child(
                Node::new(vec![0xb8])
                    .with_more_bytes(vec![0x02])
                    .with_comment("map(0x2 = 2)".to_string())
                    .with_children(vec![
                        Node::new(vec![0x01]).with_comment("unsigned(0x1) = 1".to_string()),
                        Node::new(vec![0x02]).with_comment("unsigned(0x2) = 2".to_string()),
                        Node::new(vec![0x03]).with_comment("unsigned(0x3) = 3".to_string()),
                        Node::new(vec![0x04]).with_comment("unsigned(0x4) = 4".to_string()),
                    ]),
            );
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_tag_more_bytes_4() -> Result<()> {
        let input = b"\xda\xcc\xdd\xee\xff\xb8\x02\x01\x02\x03\x04\x00";
        let expected = Node::new(vec![0xda])
            .with_more_bytes(vec![0xcc, 0xdd, 0xee, 0xff])
            .with_comment("tag(0xccddeeff = 3437096703)".to_string())
            .with_child(
                Node::new(vec![0xb8])
                    .with_more_bytes(vec![0x02])
                    .with_comment("map(0x2 = 2)".to_string())
                    .with_children(vec![
                        Node::new(vec![0x01]).with_comment("unsigned(0x1) = 1".to_string()),
                        Node::new(vec![0x02]).with_comment("unsigned(0x2) = 2".to_string()),
                        Node::new(vec![0x03]).with_comment("unsigned(0x3) = 3".to_string()),
                        Node::new(vec![0x04]).with_comment("unsigned(0x4) = 4".to_string()),
                    ]),
            );
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn parse_tag_more_bytes_8() -> Result<()> {
        let input = b"\xdb\x88\x99\xaa\xbb\xcc\xdd\xee\xff\xb8\x02\x01\x02\x03\x04\x00";
        let expected = Node::new(vec![0xdb])
            .with_more_bytes(vec![0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff])
            .with_comment("tag(0x8899aabbccddeeff = 9843086184167632639)".to_string())
            .with_child(
                Node::new(vec![0xb8])
                    .with_more_bytes(vec![0x02])
                    .with_comment("map(0x2 = 2)".to_string())
                    .with_children(vec![
                        Node::new(vec![0x01]).with_comment("unsigned(0x1) = 1".to_string()),
                        Node::new(vec![0x02]).with_comment("unsigned(0x2) = 2".to_string()),
                        Node::new(vec![0x03]).with_comment("unsigned(0x3) = 3".to_string()),
                        Node::new(vec![0x04]).with_comment("unsigned(0x4) = 4".to_string()),
                    ]),
            );
        let (input, actual) = super::cbor_object(input)?;
        assert_eq!(input, b"\x00");
        assert_eq!(actual, expected);
        Ok(())
    }
}
