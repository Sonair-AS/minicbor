#![cfg(feature = "std")]

use minicbor::{CborLen, Encode, Decode};
use quickcheck::{Arbitrary, Gen, quickcheck};

#[derive(Encode, Decode, CborLen, Clone, Debug)]
#[cbor(array)]
enum SampleArrayEncoding {
    #[n(0)] Unit,
    #[n(1)] Struct {
        #[n(0)] field1: String,
        #[n(1)] field2: bool
    },
    #[n(2)] TupleStruct(#[n(0)] u32, #[n(1)] String),
    #[n(3)] Generic(#[n(0)] u64)
}

#[derive(Encode, Decode, CborLen, Clone, Debug)]
#[cbor(map)]
enum SampleMapEncoding {
    #[n(0)] Unit,
    #[n(1)] Struct {
        #[n(0)] field1: String,
        #[n(1)] field2: bool
    },
    #[n(2)] TupleStruct(#[n(0)] u32, #[n(1)] String),
    #[n(3)] Generic(#[n(0)] u64)
}

#[derive(Encode, Decode, CborLen, Clone, Debug)]
#[cbor(transparent)]
struct TransparentEncoding(#[cbor(n(0))] u8);

#[derive(Default, Encode, CborLen)]
#[cbor(map)]
struct OptionalMapEncoding {
    #[n(0)]  f00: Option<u8>,
    #[n(1)]  f01: Option<u8>,
    #[n(2)]  f02: Option<u8>,
    #[n(3)]  f03: Option<u8>,
    #[n(4)]  f04: Option<u8>,
    #[n(5)]  f05: Option<u8>,
    #[n(6)]  f06: Option<u8>,
    #[n(7)]  f07: Option<u8>,
    #[n(8)]  f08: Option<u8>,
    #[n(9)]  f09: Option<u8>,
    #[n(10)] f10: Option<u8>,
    #[n(11)] f11: Option<u8>,
    #[n(12)] f12: Option<u8>,
    #[n(13)] f13: Option<u8>,
    #[n(14)] f14: Option<u8>,
    #[n(15)] f15: Option<u8>,
    #[n(16)] f16: Option<u8>,
    #[n(17)] f17: Option<u8>,
    #[n(18)] f18: Option<u8>,
    #[n(19)] f19: Option<u8>,
    #[n(20)] f20: Option<u8>,
    #[n(21)] f21: Option<u8>,
    #[n(22)] f22: Option<u8>,
    #[n(23)] f23: Option<u8>
}

impl Arbitrary for SampleArrayEncoding {
    fn arbitrary(g: &mut Gen) -> Self {
        match g.choose(&[0, 1, 2, 3]).unwrap() {
            0 => SampleArrayEncoding::Unit,
            1 => SampleArrayEncoding::Struct {
                field1: Arbitrary::arbitrary(g),
                field2: Arbitrary::arbitrary(g)
            },
            2 => SampleArrayEncoding::TupleStruct(Arbitrary::arbitrary(g), Arbitrary::arbitrary(g)),
            _ => SampleArrayEncoding::Generic(Arbitrary::arbitrary(g))
        }
    }
}

impl Arbitrary for SampleMapEncoding {
    fn arbitrary(g: &mut Gen) -> Self {
        match g.choose(&[0, 1, 2, 3]).unwrap() {
            0 => SampleMapEncoding::Unit,
            1 => SampleMapEncoding::Struct {
                field1: Arbitrary::arbitrary(g),
                field2: Arbitrary::arbitrary(g)
            },
            2 => SampleMapEncoding::TupleStruct(Arbitrary::arbitrary(g), Arbitrary::arbitrary(g)),
            _ => SampleMapEncoding::Generic(Arbitrary::arbitrary(g))
        }
    }
}

impl Arbitrary for TransparentEncoding {
    fn arbitrary(g: &mut Gen) -> Self {
        TransparentEncoding(Arbitrary::arbitrary(g))
    }
}

#[test]
fn derived_map_length_uses_encoded_field_count() {
    let len = OptionalMapEncoding::default().cbor_len(&mut ());
    let bytes = minicbor::to_vec(OptionalMapEncoding::default()).unwrap();
    assert_eq!(bytes.len(), len);
}

quickcheck! {
    fn sample_array(val: SampleArrayEncoding) -> bool {
        let bytes = minicbor::to_vec(&val).unwrap();
        bytes.len() == minicbor::len(&val)
    }

    fn sample_map(val: SampleMapEncoding) -> bool {
        let bytes = minicbor::to_vec(&val).unwrap();
        bytes.len() == minicbor::len(&val)
    }

    fn sample_transparent(val: TransparentEncoding) -> bool {
        let bytes = minicbor::to_vec(&val).unwrap();
        bytes.len() == minicbor::len(&val)
    }
}
