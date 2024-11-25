use crate::tree::Node;
use crate::type_array::Array;
use crate::type_byte_string::{ByteString, ByteStringWithEmbedded, IndefiniteByteString};
use crate::type_map::Map;
use crate::type_negative::NegativeInteger;
use crate::type_simple_or_float::{
    Break, DoublePrecisionFloat, HalfPrecisionFloat, ReservedSimpleOrFloat, SimpleValue,
    SinglePrecisionFloat,
};
use crate::type_tag::Tag;
use crate::type_text_string::{IndefiniteTextString, TextString};
use crate::type_unsigned::UnsignedInteger;

pub trait ToTree {
    fn into_tree(self) -> Node;
}

pub enum CborObject {
    UnsignedInteger(UnsignedInteger),
    NegativeInteger(NegativeInteger),
    ByteString(ByteString),
    IndefiniteByteString(IndefiniteByteString),
    ByteStringWithEmbedded(ByteStringWithEmbedded),
    TextString(TextString),
    IndefiniteTextString(IndefiniteTextString),
    Array(Array),
    Map(Map),
    Tag(Tag),
    SimpleValue(SimpleValue),
    HalfPrecisionFloat(HalfPrecisionFloat),
    SinglePrecisionFloat(SinglePrecisionFloat),
    DoublePrecisionFloat(DoublePrecisionFloat),
    ReservedSimpleOrFloat(ReservedSimpleOrFloat),
    Break(Break),
}

impl CborObject {
    pub fn is_break(&self) -> bool {
        matches!(self, CborObject::Break(_))
    }
}

impl ToTree for CborObject {
    fn into_tree(self) -> Node {
        match self {
            CborObject::UnsignedInteger(x) => x.into_tree(),
            CborObject::NegativeInteger(x) => x.into_tree(),
            CborObject::ByteString(x) => x.into_tree(),
            CborObject::IndefiniteByteString(x) => x.into_tree(),
            CborObject::ByteStringWithEmbedded(x) => x.into_tree(),
            CborObject::TextString(x) => x.into_tree(),
            CborObject::IndefiniteTextString(x) => x.into_tree(),
            CborObject::Array(x) => x.into_tree(),
            CborObject::Map(x) => x.into_tree(),
            CborObject::Tag(x) => x.into_tree(),
            CborObject::SimpleValue(x) => x.into_tree(),
            CborObject::HalfPrecisionFloat(x) => x.into_tree(),
            CborObject::SinglePrecisionFloat(x) => x.into_tree(),
            CborObject::DoublePrecisionFloat(x) => x.into_tree(),
            CborObject::ReservedSimpleOrFloat(x) => x.into_tree(),
            CborObject::Break(x) => x.into_tree(),
        }
    }
}

impl From<UnsignedInteger> for CborObject {
    fn from(x: UnsignedInteger) -> Self {
        CborObject::UnsignedInteger(x)
    }
}

impl From<NegativeInteger> for CborObject {
    fn from(x: NegativeInteger) -> Self {
        CborObject::NegativeInteger(x)
    }
}

impl From<ByteString> for CborObject {
    fn from(x: ByteString) -> Self {
        CborObject::ByteString(x)
    }
}

impl From<IndefiniteByteString> for CborObject {
    fn from(x: IndefiniteByteString) -> Self {
        CborObject::IndefiniteByteString(x)
    }
}

impl From<ByteStringWithEmbedded> for CborObject {
    fn from(x: ByteStringWithEmbedded) -> Self {
        CborObject::ByteStringWithEmbedded(x)
    }
}

impl From<TextString> for CborObject {
    fn from(x: TextString) -> Self {
        CborObject::TextString(x)
    }
}

impl From<IndefiniteTextString> for CborObject {
    fn from(x: IndefiniteTextString) -> Self {
        CborObject::IndefiniteTextString(x)
    }
}

impl From<Array> for CborObject {
    fn from(x: Array) -> Self {
        CborObject::Array(x)
    }
}

impl From<Map> for CborObject {
    fn from(x: Map) -> Self {
        CborObject::Map(x)
    }
}

impl From<Tag> for CborObject {
    fn from(x: Tag) -> Self {
        CborObject::Tag(x)
    }
}

impl From<SimpleValue> for CborObject {
    fn from(x: SimpleValue) -> Self {
        CborObject::SimpleValue(x)
    }
}

impl From<HalfPrecisionFloat> for CborObject {
    fn from(x: HalfPrecisionFloat) -> Self {
        CborObject::HalfPrecisionFloat(x)
    }
}

impl From<SinglePrecisionFloat> for CborObject {
    fn from(x: SinglePrecisionFloat) -> Self {
        CborObject::SinglePrecisionFloat(x)
    }
}

impl From<DoublePrecisionFloat> for CborObject {
    fn from(x: DoublePrecisionFloat) -> Self {
        CborObject::DoublePrecisionFloat(x)
    }
}

impl From<ReservedSimpleOrFloat> for CborObject {
    fn from(x: ReservedSimpleOrFloat) -> Self {
        CborObject::ReservedSimpleOrFloat(x)
    }
}

impl From<Break> for CborObject {
    fn from(x: Break) -> Self {
        CborObject::Break(x)
    }
}
