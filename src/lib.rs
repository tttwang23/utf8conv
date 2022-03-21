
//! Thomas Wang and utf8conv contributors, Copyright 2022
//!
//! Implementation of Utf8ArrayParser, and Utf32ArrayParser, including the
//! supporting UTF8 / UTF32 recognition and translation functions.
//!
//! Works on a single buffer as well as multiple buffers without needing heap allocation.
//! This is often the most performant approach.
//!
//! An invalid Unicode decoding sequence is replaced with an Unicode Replacement codepoint.
//!
//! utf8conv is dual licensed under the Apache 2.0 license, or the MIT license.
//!
//! Code portion taken from github.com/Neal/Nom, under the Apache 2.0 license, or the MIT license.
//!
//! Example:
//!
//! fn simple_example() {
//!    let mybuffer = "abc\n".as_bytes();
//!    let mut parser = Utf8ArrayParser::new();
//!    let mut byte_slice = mybuffer;
//!    loop {
//!        match parser.parse_utf8_to_char(byte_slice)
//!        {
//!            Result::Ok((next_slice, ch)) => {
//!                byte_slice = next_slice;
//!                print!("{}", ch);
//!            }
//!            Result::Err(_) => {
//!                // for a single buffer Err is always end of data
//!                break;
//!            }
//!        }
//!    }
//! }

#[cfg(doctest)]
extern crate doc_comment;


pub use crate::utf8conv::Utf8ArrayParser;
pub use crate::utf8conv::Utf32ArrayParser;
pub use crate::utf8conv::UtfParserCommon;
pub use crate::utf8conv::IncompleteEnum;
pub use crate::utf8conv::Utf8TypeEnum;
pub use crate::utf8conv::Utf8EndEnum;
pub use crate::utf8conv::buf::FifoU8;

mod utf8conv;
