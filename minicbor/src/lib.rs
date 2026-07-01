//! A small [CBOR] codec suitable for `no_std` environments.
//!
//! The crate is organised around the following entities:
//!
//! - [`Encoder`] and [`Decoder`] for type-directed encoding and decoding
//!   of values.
//!
//! - [`Encode`] and [`Decode`] traits which can be implemented for any
//!   type that should be encoded to or decoded from CBOR. They are similar
//!   to [serde]'s `Serialize` and `Deserialize` traits but do not abstract
//!   over the encoder/decoder.
//!
//! Encoding and decoding proceeds in a type-directed way, i.e.  by calling
//! methods for expected data item types, e.g. [`Decoder::u32`] or
//! [`Encoder::str`]. In addition there is support for data type inspection.
//! The `Decoder` can be queried for the current data type which returns a
//! [`data::Type`] that can represent every possible CBOR type and decoding
//! can thus proceed based on this information. The length in bytes of a
//! value's CBOR representation can be calculated if the value's type
//! implements the [`CborLen`] trait.
//!
//! Optionally, `Encode` and `Decode` can be derived for structs and enums
//! using the respective derive macros (*requires feature* `"derive"`).
//! See [`minicbor_derive`] for details.
//!
//! [CBOR]: https://datatracker.ietf.org/doc/html/rfc8949
//! [serde]: https://serde.rs
//!
//! # Feature flags
//!
//! The following feature flags are supported:
//!
//! - `"alloc"`: Enables most collection types in a `no_std` environment.
//!
//! - `"std"`: Implies `"alloc"` and enables more functionality that depends
//!   on the `std` crate.
//!
//! - `"derive"`: Allows deriving [`Encode`] and [`Decode`] traits.
//!
//! - `"certified_subset"`: Strips all `Debug`, `Display`, and `Error` trait
//!   implementations from the build. These traits rely on `core::fmt`
//!   formatting machinery which is not part of the certified on-device code
//!   path. They are purely diagnostic aids (logging, test output, error
//!   messages) and are never invoked by the functional encode/decode logic.
//!   Removing them from the certified build reduces the verified surface area
//!   to only the codec logic that actually executes on target.
//!
//! # Example: generic encoding and decoding
//!
//! ```
//! use minicbor::{Encode, Decode};
//!
//! let input = ["hello", "world"];
//! let mut buffer = [0u8; 128];
//!
//! minicbor::encode(&input, buffer.as_mut())?;
//! let output: [&str; 2] = minicbor::decode(buffer.as_ref())?;
//! assert_eq!(input, output);
//!
//! # Ok::<_, Box<dyn core::error::Error>>(())
//! ```
//!
//! # Example: ad-hoc encoding
//!
//! ```
//! use minicbor::Encoder;
//!
//! let mut buffer = [0u8; 128];
//! let mut encoder = Encoder::new(&mut buffer[..]);
//!
//! encoder.begin_map()? // using an indefinite map here
//!     .str("hello")?.str("world")?
//!     .str("submap")?.map(2)?
//!         .u8(1)?.bool(true)?
//!         .u8(2)?.bool(false)?
//!     .u16(34234)?.array(3)?.u8(1)?.u8(2)?.u8(3)?
//!     .bool(true)?.null()?
//! .end()?;
//!
//! # Ok::<_, Box<dyn core::error::Error>>(())
//! ```
//!
//! # Example: ad-hoc decoding
//!
//! ```
//! use minicbor::Decoder;
//! use minicbor::data::IanaTag;
//!
//! let input = [
//!     0xc0, 0x74, 0x32, 0x30, 0x31, 0x33, 0x2d, 0x30,
//!     0x33, 0x2d, 0x32, 0x31, 0x54, 0x32, 0x30, 0x3a,
//!     0x30, 0x34, 0x3a, 0x30, 0x30, 0x5a
//! ];
//!
//! let mut decoder = Decoder::new(&input);
//! assert_eq!(IanaTag::DateTime.tag(), decoder.tag()?);
//! assert_eq!("2013-03-21T20:04:00Z", decoder.str()?);
//! # Ok::<_, Box<dyn core::error::Error>>(())
//! ```

#![forbid(unused_variables)]
// Explicit lifetime annotations kept for clarity in codec APIs where borrowed data flows are non-trivial.
#![allow(clippy::needless_lifetimes)]
// The certified subset feature-gates out alloc/std code paths, leaving some items unused.
#![cfg_attr(feature = "certified_subset", allow(dead_code, unused_imports))]
// coverage(off) is applied to items gated behind `alloc`, `std`, or `not(certified_subset)`.
// These code paths are excluded from the certified on-device build and therefore from coverage
// measurement. Coverage is tracked only for the certified subset (no_std, no alloc).
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod data;
pub mod decode;
pub mod encode;

const UNSIGNED: u8 = 0x00;
const SIGNED: u8   = 0x20;
const BYTES: u8    = 0x40;
const TEXT: u8     = 0x60;
const ARRAY: u8    = 0x80;
const MAP: u8      = 0xa0;
const TAGGED: u8   = 0xc0;
const SIMPLE: u8   = 0xe0;
const BREAK: u8    = 0xff;

pub use decode::{Decode, Decoder};
pub use encode::{Encode, Encoder, CborLen};

#[cfg(feature = "derive")]
pub use minicbor_derive::*;
#[cfg(feature = "derive")]
mod derive;

#[cfg(feature = "alloc")]
use core::convert::Infallible;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Decode a type implementing [`Decode`] from the given byte slice.
pub fn decode<'b, T>(b: &'b [u8]) -> Result<T, decode::Error>
where
    T: Decode<'b, ()>
{
    Decoder::new(b).decode()
}

/// Decode a type implementing [`Decode`] from the given byte slice.
pub fn decode_with<'b, C, T>(b: &'b [u8], ctx: &mut C) -> Result<T, decode::Error>
where
    T: Decode<'b, C>
{
    Decoder::new(b).decode_with(ctx)
}

/// Encode a type implementing [`Encode`] to the given [`encode::Write`] impl.
pub fn encode<T, W>(x: T, w: W) -> Result<(), encode::Error<W::Error>>
where
    T: Encode<()>,
    W: encode::Write
{
    Encoder::new(w).encode(x)?.ok()
}

/// Encode a type implementing [`Encode`] to the given [`encode::Write`] impl.
pub fn encode_with<C, T, W>(x: T, w: W, ctx: &mut C) -> Result<(), encode::Error<W::Error>>
where
    T: Encode<C>,
    W: encode::Write
{
    Encoder::new(w).encode_with(x, ctx)?.ok()
}

/// Encode a type implementing [`Encode`] and return the encoded byte vector.
///
/// *Requires feature* `"alloc"`.
#[cfg(feature = "alloc")]
// Excluded from coverage — see lib.rs for rationale.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn to_vec<T>(x: T) -> Result<Vec<u8>, encode::Error<Infallible>>
where
    T: Encode<()>
{
    let mut e = Encoder::new(Vec::new());
    x.encode(&mut e, &mut ())?;
    Ok(e.into_writer())
}

/// Encode a type implementing [`Encode`] and return the encoded byte vector.
///
/// *Requires feature* `"alloc"`.
#[cfg(feature = "alloc")]
// Excluded from coverage — see lib.rs for rationale.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn to_vec_with<C, T>(x: T, ctx: &mut C) -> Result<Vec<u8>, encode::Error<Infallible>>
where
    T: Encode<C>
{
    let mut e = Encoder::new(Vec::new());
    x.encode(&mut e, ctx)?;
    Ok(e.into_writer())
}

/// Calculate the length in bytes of the given value's CBOR representation.
pub fn len<T>(x: T) -> usize
where
    T: CborLen<()>
{
    x.cbor_len(&mut ())
}

/// Calculate the length in bytes of the given value's CBOR representation.
pub fn len_with<C, T>(x: T, ctx: &mut C) -> usize
where
    T: CborLen<C>
{
    x.cbor_len(ctx)
}

// Ensure we can safely cast a `usize` to a `u64`.
const __USIZE_FITS_INTO_U64: () =
    assert!(core::mem::size_of::<usize>() <= core::mem::size_of::<u64>());
