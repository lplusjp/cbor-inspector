use crate::tree::Node;

pub enum AdditionalInfoValue {
    Value(u64),
    Reserved,
    Indefinite,
}

pub struct ParsedBytesWithValue {
    pub bytes: Vec<u8>,
    pub more_bytes: Vec<u8>,
    pub additional_info_value: AdditionalInfoValue,
}

impl ParsedBytesWithValue {
    pub fn new(
        bytes: Vec<u8>,
        more_bytes: Vec<u8>,
        additional_info_value: AdditionalInfoValue,
    ) -> Self {
        Self {
            bytes,
            more_bytes,
            additional_info_value,
        }
    }

    pub fn into_node(self) -> Node {
        Node::new(self.bytes).with_more_bytes(self.more_bytes)
    }
}

pub struct ParsedBytesWithoutValue {
    pub bytes: Vec<u8>,
    pub more_bytes: Vec<u8>,
}

impl ParsedBytesWithoutValue {
    pub fn new(bytes: Vec<u8>, more_bytes: Vec<u8>) -> Self {
        Self { bytes, more_bytes }
    }

    pub fn into_node(self) -> Node {
        Node::new(self.bytes).with_more_bytes(self.more_bytes)
    }
}
