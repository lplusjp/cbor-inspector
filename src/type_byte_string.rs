use bstr::ByteSlice as _;

use crate::cbor_object::{CborObject, ToTree};
use crate::tree::Node;
use crate::type_common::{AdditionalInfoValue, ParsedBytesWithValue};

pub struct ByteString {
    parsed_bytes: ParsedBytesWithValue,
    value: Vec<u8>,
}

impl ByteString {
    pub fn new(
        bytes: Vec<u8>,
        more_bytes: Vec<u8>,
        additional_info_value: AdditionalInfoValue,
        value: Vec<u8>,
    ) -> Self {
        Self {
            parsed_bytes: ParsedBytesWithValue::new(bytes, more_bytes, additional_info_value),
            value,
        }
    }
}

impl ToTree for ByteString {
    fn into_tree(self) -> Node {
        let comment = format!("bstr({:#x} = {})", self.value.len(), self.value.len());
        let payload_comment = format!("\"{}\"", self.value.escape_bytes());
        let payload_node = Node::new(self.value.to_owned()).with_comment(payload_comment);
        self.parsed_bytes
            .into_node()
            .with_comment(comment)
            .with_child(payload_node)
    }
}

pub struct IndefiniteByteString {
    parsed_bytes: ParsedBytesWithValue,
    value: Vec<CborObject>,
}

impl IndefiniteByteString {
    pub fn new(bytes: Vec<u8>, value: Vec<CborObject>) -> Self {
        Self {
            parsed_bytes: ParsedBytesWithValue::new(bytes, vec![], AdditionalInfoValue::Indefinite),
            value,
        }
    }
}

impl ToTree for IndefiniteByteString {
    fn into_tree(self) -> Node {
        let IndefiniteByteString {
            parsed_bytes,
            value,
        } = self;
        let comment = "bstr(*)";
        let children = value.into_iter().map(|child| child.into_tree()).collect();
        parsed_bytes
            .into_node()
            .with_comment(comment)
            .with_children(children)
    }
}

pub struct ByteStringWithEmbedded {
    parsed_bytes: ParsedBytesWithValue,
    raw_value: Vec<u8>,
    value: Box<CborObject>,
}

impl ByteStringWithEmbedded {
    pub fn new(
        bytes: Vec<u8>,
        more_bytes: Vec<u8>,
        additional_info_value: AdditionalInfoValue,
        raw_value: Vec<u8>,
        value: CborObject,
    ) -> Self {
        Self {
            parsed_bytes: ParsedBytesWithValue::new(bytes, more_bytes, additional_info_value),
            raw_value,
            value: Box::new(value),
        }
    }
}

impl ToTree for ByteStringWithEmbedded {
    fn into_tree(self) -> Node {
        let ByteStringWithEmbedded {
            parsed_bytes,
            raw_value,
            value,
        } = self;
        let comment = format!("bstr({:#x} = {})", raw_value.len(), raw_value.len());
        parsed_bytes
            .into_node()
            .with_comment(comment)
            .with_child(value.into_tree().mark_embedded())
    }
}
