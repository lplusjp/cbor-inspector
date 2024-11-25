use half::f16;

use crate::cbor_object::ToTree;
use crate::tree::Node;
use crate::type_common::ParsedBytesWithoutValue;

const SIMPLE_VALUE_FALSE: u8 = 20;
const SIMPLE_VALUE_TRUE: u8 = 21;
const SIMPLE_VALUE_NULL: u8 = 22;
const SIMPLE_VALUE_UNDEFINED: u8 = 23;

pub struct SimpleValue {
    parsed_bytes: ParsedBytesWithoutValue,
    value: u8,
}

impl SimpleValue {
    pub fn new(bytes: Vec<u8>, more_bytes: Vec<u8>, value: u8) -> Self {
        Self {
            parsed_bytes: ParsedBytesWithoutValue::new(bytes, more_bytes),
            value,
        }
    }
}

impl ToTree for SimpleValue {
    fn into_tree(self) -> Node {
        let SimpleValue {
            parsed_bytes,
            value,
        } = self;
        let comment = format!(
            "simple({:#x} = {}) = {}",
            value,
            value,
            match value {
                SIMPLE_VALUE_FALSE => "false",
                SIMPLE_VALUE_TRUE => "true",
                SIMPLE_VALUE_NULL => "null",
                SIMPLE_VALUE_UNDEFINED => "undefined",
                _ => "?",
            }
        );
        parsed_bytes.into_node().with_comment(comment)
    }
}

pub struct HalfPrecisionFloat {
    parsed_bytes: ParsedBytesWithoutValue,
    value: f16,
}

impl HalfPrecisionFloat {
    pub fn new(bytes: Vec<u8>, more_bytes: Vec<u8>, value: f16) -> Self {
        Self {
            parsed_bytes: ParsedBytesWithoutValue::new(bytes, more_bytes),
            value,
        }
    }
}

impl ToTree for HalfPrecisionFloat {
    fn into_tree(self) -> Node {
        let HalfPrecisionFloat {
            parsed_bytes,
            value,
        } = self;
        let comment = format!("float16({:.1e})", value);
        parsed_bytes.into_node().with_comment(comment)
    }
}

pub struct SinglePrecisionFloat {
    parsed_bytes: ParsedBytesWithoutValue,
    value: f32,
}

impl SinglePrecisionFloat {
    pub fn new(bytes: Vec<u8>, more_bytes: Vec<u8>, value: f32) -> Self {
        Self {
            parsed_bytes: ParsedBytesWithoutValue::new(bytes, more_bytes),
            value,
        }
    }
}

impl ToTree for SinglePrecisionFloat {
    fn into_tree(self) -> Node {
        let SinglePrecisionFloat {
            parsed_bytes,
            value,
        } = self;
        let comment = format!("float32({:.1e})", value);
        parsed_bytes.into_node().with_comment(comment)
    }
}

pub struct DoublePrecisionFloat {
    parsed_bytes: ParsedBytesWithoutValue,
    value: f64,
}

impl DoublePrecisionFloat {
    pub fn new(bytes: Vec<u8>, more_bytes: Vec<u8>, value: f64) -> Self {
        Self {
            parsed_bytes: ParsedBytesWithoutValue::new(bytes, more_bytes),
            value,
        }
    }
}

impl ToTree for DoublePrecisionFloat {
    fn into_tree(self) -> Node {
        let DoublePrecisionFloat {
            parsed_bytes,
            value,
        } = self;
        let comment = format!("float64({:.1e})", value);
        parsed_bytes.into_node().with_comment(comment)
    }
}

pub struct ReservedSimpleOrFloat {
    parsed_bytes: ParsedBytesWithoutValue,
    additional_info_argument: u8,
}

impl ReservedSimpleOrFloat {
    pub fn new(bytes: Vec<u8>, additional_info_argument: u8) -> Self {
        Self {
            parsed_bytes: ParsedBytesWithoutValue::new(bytes, vec![]),
            additional_info_argument,
        }
    }
}

impl ToTree for ReservedSimpleOrFloat {
    fn into_tree(self) -> Node {
        let ReservedSimpleOrFloat {
            parsed_bytes,
            additional_info_argument,
        } = self;
        let comment = format!(
            "reserved simple/float({:#x} = {})",
            additional_info_argument, additional_info_argument,
        );
        parsed_bytes.into_node().with_comment(comment)
    }
}

pub struct Break {
    parsed_bytes: ParsedBytesWithoutValue,
}

impl Break {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            parsed_bytes: ParsedBytesWithoutValue::new(bytes, vec![]),
        }
    }
}

impl ToTree for Break {
    fn into_tree(self) -> Node {
        self.parsed_bytes.into_node().with_comment("break")
    }
}
