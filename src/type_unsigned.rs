use crate::cbor_object::ToTree;
use crate::tree::Node;
use crate::type_common::{AdditionalInfoValue, ParsedBytesWithValue};

pub struct UnsignedInteger {
    parsed_bytes: ParsedBytesWithValue,
}

impl UnsignedInteger {
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

impl ToTree for UnsignedInteger {
    fn into_tree(self) -> Node {
        let comment = match self.parsed_bytes.additional_info_value {
            AdditionalInfoValue::Value(x) => format!("unsigned({:#x}) = {}", x, x),
            _ => "unsigned(?)".to_string(),
        };
        self.parsed_bytes.into_node().with_comment(comment)
    }
}
