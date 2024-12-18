use half::f16;
use nom::bytes::complete::take;
use nom::combinator::{eof, fail};
use nom::IResult;

use crate::cbor_object::CborObject;
use crate::type_array::Array;
use crate::type_byte_string::{ByteString, ByteStringWithEmbedded, IndefiniteByteString};
use crate::type_common::AdditionalInfoValue;
use crate::type_map::Map;
use crate::type_negative::NegativeInteger;
use crate::type_simple_or_float::{
    Break, DoublePrecisionFloat, HalfPrecisionFloat, ReservedSimpleOrFloat, SimpleValue,
    SinglePrecisionFloat,
};
use crate::type_tag::Tag;
use crate::type_text_string::{IndefiniteTextString, TextString};
use crate::type_unsigned::UnsignedInteger;

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
                AdditionalInfoValue::Value(argument.into())
            }
            ADDITIONAL_INFO_VALUE_FOLLOWED_BY_1_BYTE => {
                AdditionalInfoValue::Value(more_bytes[0].into())
            }
            ADDITIONAL_INFO_VALUE_FOLLOWED_BY_2_BYTES => {
                // unwrap safety: more_bytes is 2 bytes
                AdditionalInfoValue::Value(
                    u16::from_be_bytes(more_bytes.try_into().unwrap()).into(),
                )
            }
            ADDITIONAL_INFO_VALUE_FOLLOWED_BY_4_BYTES => {
                // unwrap safety: more_bytes is 4 bytes
                AdditionalInfoValue::Value(
                    u32::from_be_bytes(more_bytes.try_into().unwrap()).into(),
                )
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

fn unsigned_integer(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], CborObject> {
    move |input: &[u8]| {
        let (major_type, additional_info_argument) = split_major_type_and_additional_info(b);
        if major_type != MAJOR_UNSIGNED_INTEGER {
            return fail(input);
        }

        let (input, (value, more_bytes)) =
            parse_additional_info_value(additional_info_argument)(input)?;
        let object = UnsignedInteger::new(vec![b], more_bytes.to_owned(), value).into();
        Ok((input, object))
    }
}

fn negative_integer(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], CborObject> {
    move |input: &[u8]| {
        let (major_type, additional_info_argument) = split_major_type_and_additional_info(b);
        if major_type != MAJOR_NEGATIVE_INTEGER {
            return fail(input);
        }

        let (input, (value, more_bytes)) =
            parse_additional_info_value(additional_info_argument)(input)?;
        let object = NegativeInteger::new(vec![b], more_bytes.to_owned(), value).into();
        Ok((input, object))
    }
}

fn embedded_cbor_object(input: &[u8], length: usize) -> IResult<&[u8], (Vec<u8>, CborObject)> {
    let (input, cbor_input) = take(length)(input)?;
    let (rest, cbor_object) = cbor_object(cbor_input)?;
    eof(rest)?;
    Ok((input, (cbor_input.to_owned(), cbor_object)))
}

fn byte_string_or_text_string(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], CborObject> {
    move |input: &[u8]| {
        let (major_type, additional_info_argument) = split_major_type_and_additional_info(b);
        if major_type != MAJOR_BYTE_STRING && major_type != MAJOR_TEXT_STRING {
            return fail(input);
        }

        let (input, (additional_info_value, more_bytes)) =
            parse_additional_info_value(additional_info_argument)(input)?;

        // Embedded CBOR object
        if major_type == MAJOR_BYTE_STRING {
            // TODO: AdditionalInfoValue::Indefinite
            if let AdditionalInfoValue::Value(length) = additional_info_value {
                if let Ok((input, (embedded_raw, embedded_object))) =
                    embedded_cbor_object(input, usize::try_from(length).unwrap_or(usize::MAX))
                {
                    return Ok((
                        input,
                        CborObject::ByteStringWithEmbedded(ByteStringWithEmbedded::new(
                            vec![b],
                            more_bytes.to_owned(),
                            additional_info_value,
                            embedded_raw,
                            embedded_object,
                        )),
                    ));
                }
            }
        }

        let mut input = input;
        let object = match additional_info_value {
            AdditionalInfoValue::Value(_) | AdditionalInfoValue::Reserved => {
                let (input_new, payload) = match additional_info_value {
                    AdditionalInfoValue::Value(length) => {
                        take(usize::try_from(length).unwrap_or(usize::MAX))(input)?
                    }
                    AdditionalInfoValue::Reserved => take(0usize)(input)?,
                    // unreachable safety: In this branch, length is always Value or Reserved
                    _ => unreachable!(),
                };
                input = input_new;
                match major_type {
                    MAJOR_BYTE_STRING => CborObject::ByteString(ByteString::new(
                        vec![b],
                        more_bytes.to_owned(),
                        additional_info_value,
                        payload.to_owned(),
                    )),
                    MAJOR_TEXT_STRING => CborObject::TextString(TextString::new(
                        vec![b],
                        more_bytes.to_owned(),
                        additional_info_value,
                        payload.to_owned(),
                    )),
                    // unreachable safety: when major_type is not byte string or text string, already failed
                    _ => unreachable!(),
                }
            }
            AdditionalInfoValue::Indefinite => {
                let mut children = Vec::new();
                loop {
                    let (input_new, child) = cbor_object(input)?;
                    input = input_new;
                    let is_break = child.is_break();
                    children.push(child);
                    if is_break {
                        break;
                    }
                }
                match major_type {
                    MAJOR_BYTE_STRING => CborObject::IndefiniteByteString(
                        IndefiniteByteString::new(vec![b], children),
                    ),
                    MAJOR_TEXT_STRING => CborObject::IndefiniteTextString(
                        IndefiniteTextString::new(vec![b], children),
                    ),
                    // unreachable safety: when major_type is not byte string or text string, already failed
                    _ => unreachable!(),
                }
            }
        };

        Ok((input, object))
    }
}

fn byte_string(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], CborObject> {
    byte_string_or_text_string(b)
}

fn text_string(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], CborObject> {
    byte_string_or_text_string(b)
}

fn array_or_map(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], CborObject> {
    move |input: &[u8]| {
        let (major_type, additional_info_argument) = split_major_type_and_additional_info(b);
        if major_type != MAJOR_ARRAY && major_type != MAJOR_MAP {
            return fail(input);
        }

        let (input, (additional_info_value, more_bytes)) =
            parse_additional_info_value(additional_info_argument)(input)?;
        let mut input = input;
        let children = match additional_info_value {
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
                children
            }
            AdditionalInfoValue::Reserved => vec![],
            AdditionalInfoValue::Indefinite => {
                let mut children = Vec::new();
                loop {
                    let (input_new, child) = cbor_object(input)?;
                    input = input_new;
                    let is_break = child.is_break();
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
            match major_type {
                MAJOR_ARRAY => CborObject::Array(Array::new(
                    vec![b],
                    more_bytes.to_owned(),
                    additional_info_value,
                    children,
                )),
                MAJOR_MAP => CborObject::Map(Map::new(
                    vec![b],
                    more_bytes.to_owned(),
                    additional_info_value,
                    children,
                )),
                // unreachable safety: when major_type is not array or map, already failed
                _ => unreachable!(),
            },
        ))
    }
}

fn array(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], CborObject> {
    array_or_map(b)
}

fn map(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], CborObject> {
    array_or_map(b)
}

fn tag(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], CborObject> {
    move |input: &[u8]| {
        let (major_type, additional_info_argument) = split_major_type_and_additional_info(b);
        if major_type != MAJOR_TAG {
            return fail(input);
        }

        let (input, (additional_info_value, more_bytes)) =
            parse_additional_info_value(additional_info_argument)(input)?;

        let (input, child) = cbor_object(input)?;
        Ok((
            input,
            CborObject::Tag(Tag::new(
                vec![b],
                more_bytes.to_owned(),
                additional_info_value,
                child,
            )),
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

fn simple_or_float(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], CborObject> {
    move |input: &[u8]| {
        let (major_type, additional_info_argument) = split_major_type_and_additional_info(b);
        if major_type != MAJOR_SIMPLE_OR_FLOAT {
            return fail(input);
        }

        let (input, (simple_or_float_value, more_bytes)) =
            parse_simple_or_float_value(additional_info_argument)(input)?;
        let object = match simple_or_float_value {
            SimpleOrFloat::Simple(simple_value) => {
                SimpleValue::new(vec![b], more_bytes.to_owned(), simple_value).into()
            }
            SimpleOrFloat::FloatHalf(float_half) => {
                HalfPrecisionFloat::new(vec![b], more_bytes.to_owned(), float_half).into()
            }
            SimpleOrFloat::FloatSingle(float_single) => {
                SinglePrecisionFloat::new(vec![b], more_bytes.to_owned(), float_single).into()
            }
            SimpleOrFloat::FloatDouble(float_double) => {
                DoublePrecisionFloat::new(vec![b], more_bytes.to_owned(), float_double).into()
            }
            SimpleOrFloat::Reserved => {
                ReservedSimpleOrFloat::new(vec![b], additional_info_argument).into()
            }
            SimpleOrFloat::Break => Break::new(vec![b]).into(),
        };
        Ok((input, object))
    }
}

fn cbor_object(orig_input: &[u8]) -> IResult<&[u8], CborObject> {
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

pub fn parse_cbor(input: &[u8]) -> IResult<&[u8], CborObject> {
    cbor_object(input)
}
