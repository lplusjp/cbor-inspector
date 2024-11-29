use bstr::ByteSlice as _;

use crate::cbor_object::{CborObject, ToTree};
use crate::tree::Node;
use crate::type_common::{AdditionalInfoValue, ParsedBytesWithValue};

pub struct TextString {
    parsed_bytes: ParsedBytesWithValue,
    value: Vec<u8>,
}

impl TextString {
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

impl ToTree for TextString {
    fn into_tree(self) -> Node {
        let comment = format!("tstr({:#x} = {})", self.value.len(), self.value.len());
        let payload_comment = format!("{:?}", self.value.as_bstr().to_str_lossy());
        let payload_node = Node::new(self.value.to_owned()).with_comment(payload_comment);
        self.parsed_bytes
            .into_node()
            .with_comment(comment)
            .with_child(payload_node)
    }
}

pub struct IndefiniteTextString {
    parsed_bytes: ParsedBytesWithValue,
    value: Vec<CborObject>,
}

impl IndefiniteTextString {
    pub fn new(bytes: Vec<u8>, value: Vec<CborObject>) -> Self {
        Self {
            parsed_bytes: ParsedBytesWithValue::new(bytes, vec![], AdditionalInfoValue::Indefinite),
            value,
        }
    }
}

impl ToTree for IndefiniteTextString {
    fn into_tree(self) -> Node {
        let IndefiniteTextString {
            parsed_bytes,
            value,
        } = self;
        let comment = "tstr(*)";
        let children = value.into_iter().map(|child| child.into_tree()).collect();
        parsed_bytes
            .into_node()
            .with_comment(comment)
            .with_children(children)
    }
}
