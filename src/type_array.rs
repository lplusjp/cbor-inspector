use crate::cbor_object::{CborObject, ToTree};
use crate::tree::Node;
use crate::type_common::{AdditionalInfoValue, ParsedBytesWithValue};

pub struct Array {
    parsed_bytes: ParsedBytesWithValue,
    value: Vec<CborObject>,
}

impl Array {
    pub fn new(
        bytes: Vec<u8>,
        more_bytes: Vec<u8>,
        additional_info_value: AdditionalInfoValue,
        value: Vec<CborObject>,
    ) -> Self {
        Self {
            parsed_bytes: ParsedBytesWithValue::new(bytes, more_bytes, additional_info_value),
            value,
        }
    }
}

impl ToTree for Array {
    fn into_tree(self) -> Node {
        let Array {
            parsed_bytes,
            value,
        } = self;
        let comment = match parsed_bytes.additional_info_value {
            AdditionalInfoValue::Value(length) => format!("array({:#x} = {})", length, length),
            AdditionalInfoValue::Reserved => "array(?)".to_string(),
            AdditionalInfoValue::Indefinite => "array(*)".to_string(),
        };
        let children = value.into_iter().map(|child| child.into_tree()).collect();
        parsed_bytes
            .into_node()
            .with_comment(comment)
            .with_children(children)
    }
}
