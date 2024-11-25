use crate::cbor_object::ToTree;
use crate::tree::Node;
use crate::type_common::{AdditionalInfoValue, ParsedBytesWithValue};

pub struct NegativeInteger {
    parsed_bytes: ParsedBytesWithValue,
}

impl NegativeInteger {
    pub fn new(
        bytes: Vec<u8>,
        more_bytes: Vec<u8>,
        additional_info_value: AdditionalInfoValue,
    ) -> Self {
        Self {
            parsed_bytes: ParsedBytesWithValue::new(bytes, more_bytes, additional_info_value),
        }
    }
}

impl ToTree for NegativeInteger {
    fn into_tree(self) -> Node {
        let comment = match self.parsed_bytes.additional_info_value {
            AdditionalInfoValue::Value(x) => format!("negative({:#x}) = {}", x, -1 - (x as i128)),
            _ => "negative(?)".to_string(),
        };
        self.parsed_bytes.into_node().with_comment(comment)
    }
}
