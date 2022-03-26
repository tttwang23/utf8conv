// Thomas Wang and utf8conv contributors, Copyright 2022
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//!
//! Implementation of UTF 8 / UTF32 converters and converting iterators,
//! including the supporting recognition and translation functions.
//!
//! Works on a single buffer as well as multiple buffers without needing
//! heap allocation.
//!
//! An invalid Unicode decoding sequence is replaced with an Unicode Replacement codepoint.
//!
//! utf8conv is dual licensed under the Apache 2.0 license, or the MIT license.
//!
//! Code portion taken from github.com/Neal/Nom, under the Apache 2.0 license, or the MIT license.
//!
//! Example:
//!
//! fn iterator_example() {
//!     let mybuffer = "abc\n".as_bytes();
//!     let mut utf8_ref_iter = mybuffer.iter();
//!     let mut parser = FromUtf8::new();
//!     let iterator = parser.utf8_ref_to_char_with_iter(& mut utf8_ref_iter);
//!     for char_val in iterator {
//!         print!("{}", char_val);
//!     }
//! }


#[cfg(doctest)]
extern crate doc_comment;

/// Make common symbols available in our prelude.
///
/// use utf8conv::prelude::*;
pub mod prelude {
    pub use crate::utf8conv::REPLACE_UTF32;
    pub use crate::utf8conv::REPLACE_PART1;
    pub use crate::utf8conv::REPLACE_PART2;
    pub use crate::utf8conv::REPLACE_PART3;
    pub use crate::utf8conv::FromUtf8;
    pub use crate::utf8conv::FromUnicode;
    pub use crate::utf8conv::UtfParserCommon;
    pub use crate::utf8conv::Utf8IterToCharIter;
    pub use crate::utf8conv::Utf32IterToUtf8Iter;
    pub use crate::utf8conv::Utf8TypeEnum;
    pub use crate::utf8conv::Utf8EndEnum;
    pub use crate::utf8conv::MoreEnum;
    pub use crate::utf8conv::classify_utf32;
    pub use crate::utf8conv::utf8_decode;
    pub use crate::utf8conv::char_ref_iter_to_char_iter;
    pub use crate::utf8conv::utf32_ref_iter_to_utf32_iter;
    pub use crate::utf8conv::utf8_ref_iter_to_utf8_iter;
    pub use crate::utf8conv::char_iter_to_utf32_iter;
    pub use crate::utf8conv::buf::EightBytes;
}

mod utf8conv;
