use anyhow::Result;
use pretty_assertions::assert_eq;

use crate::cbor_object::*;
use crate::cbor_parser::parse_cbor;
use crate::tree::Node;

#[test]
fn parse_unsigned_integer_short() -> Result<()> {
    let input = b"\x01\x00";
    let expected = Node::new(vec![0x01]).with_comment("unsigned(0x1) = 1".to_string());
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_unsigned_integer_1_byte() -> Result<()> {
    let input = b"\x18\x03\x00";
    let expected = Node::new(vec![0x18])
        .with_more_bytes(vec![0x03])
        .with_comment("unsigned(0x3) = 3".to_string());
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_unsigned_integer_2_bytes() -> Result<()> {
    let input = b"\x19\x00\x03\x00";
    let expected = Node::new(vec![0x19])
        .with_more_bytes(vec![0x00, 0x03])
        .with_comment("unsigned(0x3) = 3".to_string());
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_unsigned_integer_4_bytes() -> Result<()> {
    let input = b"\x1a\x00\x00\x00\x03\x00";
    let expected = Node::new(vec![0x1a])
        .with_more_bytes(vec![0x00, 0x00, 0x00, 0x03])
        .with_comment("unsigned(0x3) = 3".to_string());
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_negative_integer_short() -> Result<()> {
    let input = b"\x20\x00";
    let expected = Node::new(vec![0x20]).with_comment("negative(0x0) = -1".to_string());
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_negative_integer_8_bytes_min() -> Result<()> {
    let input = b"\x3b\xff\xff\xff\xff\xff\xff\xff\xff\x00";
    let expected_inner_value = 0xffffffffffffffffu64;
    let expected_value = -0x10000000000000000i128;
    let expected = Node::new(vec![0x3b])
        .with_more_bytes(vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff])
        .with_comment(format!(
            "{}({:#x}) = {}",
            "negative", expected_inner_value, expected_value
        ));
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_byte_string_short() -> Result<()> {
    let input = b"\x43\x01\x02\x03\x00";
    let expected = Node::new(vec![0x43])
        .with_comment("bstr(0x3 = 3)".to_string())
        .with_child(
            Node::new(vec![0x01, 0x02, 0x03]).with_comment("\"\\x01\\x02\\x03\"".to_string()),
        );
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_byte_string_more_bytes_1() -> Result<()> {
    let input = b"\x58\x03\x01\x02\x03\x00";
    let expected = Node::new(vec![0x58])
        .with_more_bytes(vec![0x03])
        .with_comment("bstr(0x3 = 3)".to_string())
        .with_child(
            Node::new(vec![0x01, 0x02, 0x03]).with_comment("\"\\x01\\x02\\x03\"".to_string()),
        );
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_byte_string_more_bytes_2() -> Result<()> {
    let input = b"\x59\x00\x03\x01\x02\x03\x00";
    let expected = Node::new(vec![0x59])
        .with_more_bytes(vec![0x00, 0x03])
        .with_comment("bstr(0x3 = 3)".to_string())
        .with_child(
            Node::new(vec![0x01, 0x02, 0x03]).with_comment("\"\\x01\\x02\\x03\"".to_string()),
        );
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_byte_string_more_bytes_4() -> Result<()> {
    let input = b"\x5a\x00\x00\x00\x03\x01\x02\x03\x00";
    let expected = Node::new(vec![0x5a])
        .with_more_bytes(vec![0x00, 0x00, 0x00, 0x03])
        .with_comment("bstr(0x3 = 3)".to_string())
        .with_child(
            Node::new(vec![0x01, 0x02, 0x03]).with_comment("\"\\x01\\x02\\x03\"".to_string()),
        );
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_byte_string_more_bytes_8() -> Result<()> {
    let input = b"\x5b\x00\x00\x00\x00\x00\x00\x00\x03\x01\x02\x03\x00";
    let expected = Node::new(vec![0x5b])
        .with_more_bytes(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03])
        .with_comment("bstr(0x3 = 3)".to_string())
        .with_child(
            Node::new(vec![0x01, 0x02, 0x03]).with_comment("\"\\x01\\x02\\x03\"".to_string()),
        );
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_byte_string_indefinite() -> Result<()> {
    let input = b"\x5f\x43\x01\x02\x03\x42\x04\x05\xff\x00";
    let expected = Node::new(vec![0x5f])
        .with_comment("bstr(*)".to_string())
        .with_children(vec![
            Node::new(vec![0x43])
                .with_comment("bstr(0x3 = 3)".to_string())
                .with_child(
                    Node::new(vec![0x01, 0x02, 0x03])
                        .with_comment("\"\\x01\\x02\\x03\"".to_string()),
                ),
            Node::new(vec![0x42])
                .with_comment("bstr(0x2 = 2)".to_string())
                .with_child(Node::new(vec![0x04, 0x05]).with_comment("\"\\x04\\x05\"".to_string())),
            Node::new(vec![0xff]).with_comment("break".to_string()),
        ]);
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_byte_string_embedded() -> Result<()> {
    let input = b"\x46\xb8\x02\x01\x02\x03\x04\x00";
    let expected = Node::new(vec![0x46])
        .with_comment("bstr(0x6 = 6)".to_string())
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_text_string_short() -> Result<()> {
    let input = b"\x63\x61\x62\x63\x00";
    let expected = Node::new(vec![0x63])
        .with_comment("tstr(0x3 = 3)".to_string())
        .with_child(Node::new(vec![0x61, 0x62, 0x63]).with_comment("\"abc\"".to_string()));
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_text_string_more_bytes_1() -> Result<()> {
    let input = b"\x78\x03\x61\x62\x63\x00";
    let expected = Node::new(vec![0x78])
        .with_more_bytes(vec![0x03])
        .with_comment("tstr(0x3 = 3)".to_string())
        .with_child(Node::new(vec![0x61, 0x62, 0x63]).with_comment("\"abc\"".to_string()));
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_text_string_more_bytes_2() -> Result<()> {
    let input = b"\x79\x00\x03\x61\x62\x63\x00";
    let expected = Node::new(vec![0x79])
        .with_more_bytes(vec![0x00, 0x03])
        .with_comment("tstr(0x3 = 3)".to_string())
        .with_child(Node::new(vec![0x61, 0x62, 0x63]).with_comment("\"abc\"".to_string()));
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_text_string_more_bytes_4() -> Result<()> {
    let input = b"\x7a\x00\x00\x00\x03\x61\x62\x63\x00";
    let expected = Node::new(vec![0x7a])
        .with_more_bytes(vec![0x00, 0x00, 0x00, 0x03])
        .with_comment("tstr(0x3 = 3)".to_string())
        .with_child(Node::new(vec![0x61, 0x62, 0x63]).with_comment("\"abc\"".to_string()));
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_text_string_more_bytes_8() -> Result<()> {
    let input = b"\x7b\x00\x00\x00\x00\x00\x00\x00\x03\x61\x62\x63\x00";
    let expected = Node::new(vec![0x7b])
        .with_more_bytes(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03])
        .with_comment("tstr(0x3 = 3)".to_string())
        .with_child(Node::new(vec![0x61, 0x62, 0x63]).with_comment("\"abc\"".to_string()));
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_text_string_indefinite() -> Result<()> {
    let input = b"\x7f\x63\x61\x62\x63\x62\x64\x65\xff\x00";
    let expected = Node::new(vec![0x7f])
        .with_comment("tstr(*)".to_string())
        .with_children(vec![
            Node::new(vec![0x63])
                .with_comment("tstr(0x3 = 3)".to_string())
                .with_child(Node::new(vec![0x61, 0x62, 0x63]).with_comment("\"abc\"".to_string())),
            Node::new(vec![0x62])
                .with_comment("tstr(0x2 = 2)".to_string())
                .with_child(Node::new(vec![0x64, 0x65]).with_comment("\"de\"".to_string())),
            Node::new(vec![0xff]).with_comment("break".to_string()),
        ]);
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
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
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_simple_short() -> Result<()> {
    let input = b"\xe0\x00";
    let expected = Node::new(vec![0xe0]).with_comment("simple(0x0 = 0) = ?".to_string());
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_simple_long() -> Result<()> {
    let input = b"\xf8\xff\x00";
    let expected = Node::new(vec![0xf8])
        .with_more_bytes(vec![0xff])
        .with_comment("simple(0xff = 255) = ?".to_string());
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_false() -> Result<()> {
    let input = b"\xf4\x00";
    let expected = Node::new(vec![0xf4]).with_comment("simple(0x14 = 20) = false".to_string());
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_true() -> Result<()> {
    let input = b"\xf5\x00";
    let expected = Node::new(vec![0xf5]).with_comment("simple(0x15 = 21) = true".to_string());
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_null() -> Result<()> {
    let input = b"\xf6\x00";
    let expected = Node::new(vec![0xf6]).with_comment("simple(0x16 = 22) = null".to_string());
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_undefined() -> Result<()> {
    let input = b"\xf7\x00";
    let expected = Node::new(vec![0xf7]).with_comment("simple(0x17 = 23) = undefined".to_string());
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_float_half() -> Result<()> {
    let input = b"\xf9\x3c\x00\x00";
    let expected = Node::new(vec![0xf9])
        .with_more_bytes(vec![0x3c, 0x00])
        .with_comment("float16(1e0)".to_string());
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_float_single() -> Result<()> {
    let input = b"\xfa\x47\xc3\x50\x00\x00";
    let expected = Node::new(vec![0xfa])
        .with_more_bytes(vec![0x47, 0xc3, 0x50, 0x00])
        .with_comment("float32(1.0e5)".to_string());
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_float_double() -> Result<()> {
    let input = b"\xfb\x7e\x37\xe4\x3c\x88\x00\x75\x9c\x00";
    let expected = Node::new(vec![0xfb])
        .with_more_bytes(vec![0x7e, 0x37, 0xe4, 0x3c, 0x88, 0x00, 0x75, 0x9c])
        .with_comment("float64(1.0e300)".to_string());
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_reserved_simple_or_float() -> Result<()> {
    let input = b"\xfc\x00";
    let expected =
        Node::new(vec![0xfc]).with_comment("reserved simple/float(0x1c = 28)".to_string());
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn parse_break() -> Result<()> {
    let input = b"\xff\x00";
    let expected = Node::new(vec![0xff]).with_comment("break".to_string());
    let (input, object) = parse_cbor(input)?;
    assert_eq!(input, b"\x00");
    let actual = object.into_tree();
    assert_eq!(actual, expected);
    Ok(())
}
