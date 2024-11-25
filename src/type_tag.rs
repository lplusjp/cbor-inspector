use crate::cbor_object::{CborObject, ToTree};
use crate::tree::Node;
use crate::type_common::{AdditionalInfoValue, ParsedBytesWithValue};

pub struct Tag {
    parsed_bytes: ParsedBytesWithValue,
    payload: Box<CborObject>,
}

impl Tag {
    pub fn new(
        bytes: Vec<u8>,
        more_bytes: Vec<u8>,
        additional_info_value: AdditionalInfoValue,
        payload: CborObject,
    ) -> Self {
        Self {
            parsed_bytes: ParsedBytesWithValue::new(bytes, more_bytes, additional_info_value),
            payload: Box::new(payload),
        }
    }
}

impl ToTree for Tag {
    fn into_tree(self) -> Node {
        let Tag {
            parsed_bytes,
            payload,
        } = self;
        let comment = match parsed_bytes.additional_info_value {
            AdditionalInfoValue::Value(tag) => format!("tag({:#x} = {})", tag, tag),
            AdditionalInfoValue::Reserved | AdditionalInfoValue::Indefinite => "tag(*)".to_string(),
        };
        parsed_bytes
            .into_node()
            .with_comment(comment)
            .with_child(payload.into_tree())
    }
}
