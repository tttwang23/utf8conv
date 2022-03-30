// Copyright 2022 Thomas Wang and utf8conv contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// This is the representation of the replacement character in UTF8 encoding.

/// replacement character (UTF32)
pub const REPLACE_UTF32:u32 = 0xFFFD;

/// byte 1 of replacement char in UTF8
pub const REPLACE_PART1:u8 = 0xEFu8;

/// byte 2 of replacement char in UTF8
pub const REPLACE_PART2:u8 = 0xBFu8;

/// byte 3 of replacement char in UTF8
pub const REPLACE_PART3:u8 = 0xBDu8;

/// leading bits of byte 1 for type 2 decode
const TYPE2_PREFIX:u32 = 0b1100_0000u32;

/// leading bits of byte 1 for type 3 decode
const TYPE3_PREFIX:u32 = 0b1110_0000u32;

/// leading bits of byte 1 for type 4 decode
const TYPE4_PREFIX:u32 = 0b1111_0000u32;

/// leading bits of byte 2 and onwards
const BYTE2_PREFIX:u32 = 0b1000_0000u32;

// (v & SIX_ONES) << 6 is the same as
// (v << 6) & SIX_ONES_SHIFTED
// This breaks up the pattern of using shift units in the same cycle.

/// 6 bits shifted 6 digits
const SIX_ONES_SHIFTED:u32 = 0b111111000000u32;

/// 0x3F bit mask
const SIX_ONES:u32 = 0b111111u32;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
/// Indication for needing more data when parameter value greater than 0,
/// or end of data condition when parameter value is 0.
///
/// (These are not really error conditions.)
pub enum MoreEnum {
    /// 0: end of data, greater than 0: need more data
    More(u32),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
/// Indication for the type of UTF8 decoding when converting
/// from UTF32 to UTF8
pub enum Utf8TypeEnum {
    /// 1 byte type
    Type1(u8),

    /// 2 byte type
    Type2((u8,u8)),

    /// 3 byte type
    Type3((u8,u8,u8)),

    /// 4 byte type
    Type4((u8,u8,u8,u8)),

    // invalid codepoint; substituted with replacement characters
    Type0((u8,u8,u8)),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
/// Utf8EndEnum is the result container for the UTF8 to char
/// finite state machine.
pub enum Utf8EndEnum {

    /// bad decode with failure sequence length: 1, 2, or 3
    BadDecode(u32),

    /// Finished state with a valid codepoint
    Finish(u32),

    /// not enough characters: type unknown
    TypeUnknown,
}


#[inline]
/// Classify an UTF32 value into the type of UTF8 it belongs.
///
/// Returning Utf8TypeEnum indicates the sequence length.
///
/// Returning Utf8TypeEnum::Type0 indicates error.
pub fn classify_utf32(code: u32) -> Utf8TypeEnum {
    if code < 0x80u32 {
        Utf8TypeEnum::Type1(code as u8)
    }
    else if code < 0x800u32 {
        let v1:u8 = ((code >> 6) + TYPE2_PREFIX) as u8;
        let v2:u8 = ((code & SIX_ONES) + BYTE2_PREFIX) as u8;
        Utf8TypeEnum::Type2((v1,v2))
    }
    else if (code >= 0xD800u32) && (code < 0xE000u32) {
        // Illegal UTF16 surrogate range
        Utf8TypeEnum::Type0((REPLACE_PART1, REPLACE_PART2, REPLACE_PART3))
    }
    else if code < 0x10000u32 {
        if code == REPLACE_UTF32 {
            // Treat it the same whether it is a fresh invalid codepoint
            // or an old one from the past.
            Utf8TypeEnum::Type0((REPLACE_PART1, REPLACE_PART2, REPLACE_PART3))
        }
        else {
            let v1:u8 = ((code >> 12) + TYPE3_PREFIX) as u8;
            let v2:u8 = (((code & SIX_ONES_SHIFTED) >> 6) + BYTE2_PREFIX) as u8;
            let v3:u8 = ((code & SIX_ONES) + BYTE2_PREFIX) as u8;
            Utf8TypeEnum::Type3((v1,v2,v3))
        }
    }
    else if code < 0x110000u32 {
        let v1:u8 = ((code >> 18) + TYPE4_PREFIX) as u8;
        let v2:u8 = (((code >> 12) & SIX_ONES) + BYTE2_PREFIX) as u8;
        let v3:u8 = (((code & SIX_ONES_SHIFTED) >> 6) + BYTE2_PREFIX) as u8;
        let v4:u8 = ((code & SIX_ONES) + BYTE2_PREFIX) as u8;
        Utf8TypeEnum::Type4((v1,v2,v3,v4))
    }
    else {
        // beyond valid UTF32 range
        Utf8TypeEnum::Type0((REPLACE_PART1, REPLACE_PART2, REPLACE_PART3))
    }
}


/*
Technical notes written by Henri Sivonen, selectely quoted

Unicode 9.0.0 (page 127) says: “An ill-formed subsequence consisting of more
than one code unit could be treated as a single error or as multiple errors.
For example, in processing the UTF-8 code unit sequence <F0 80 80 41>,
the only formal requirement mandated by Unicode conformance for a converter
is that the <41> be processed and correctly interpreted as <U+0041>.
The converter could return <U+FFFD, U+0041>, handling <F0 80 80> as a single
error, or <U+FFFD, U+FFFD, U+FFFD, U+0041>, handling each byte of <F0 80 80>
as a separate error, or could take other approaches to signalling <F0 80 80>
as an ill-formed code unit subsequence.” So as far as Unicode is concerned,
any number from one to the number of bytes in the number of bogus bytes
(inclusive) is OK. In other words, the precise number is
implementation-defined as far as Unicode is concerned.

> However, for the best compatibility with existing software, implementing
> the conversion with a finite state machine was the typical approach.

Code Points         First Byte   Second Byte  Third Byte  Fourth Byte
U+0000..U+007F      00..7F
>                   action 0

U+0080..U+07FF      C2..DF       80..bf
>                   action 1     action 9

U+0800..U+0FFF      E0           A0..bf       80..bf
>                   action 2     action 14    action 17

U+1000..U+CFFF      E1..EC       80..bf       80..bf
>                   action 3     action 10    action (17)

U+D000..U+D7FF      ED           80..9F       80..bf
>                   action 4     action 15    action (17)

U+E000..U+FFFF      EE..EF       80..bf       80..bf
>                   action 5     action 11    action 20 (containing FFFD)

U+10000..U+3FFFF    F0           90..bf       80..bf      80..bf
>                   action 6     action 16    action 21   action 24

U+40000..U+FFFFF    F1..F3       80..bf       80..bf      80..bf
>                   action 7     action 12    action (21) action (24)

U+100000..U+10FFFF  F4           80..8F       80..bf      80..bf
>                   action 8     action 13    action (21) action (24)

> The action number with parenthesis are duplicated actions.
> action 0: out = v1
> action 1: out = v1 & ox1F;
> action 2 to 5: out = v1 & 0xF;
> action 6 to 8: out = v1 & 0x7;
> action 9 to 13: out = (arg << 6)+(v2 & 0x3F)
> action 14: out = (arg << 6)+(v2 & 0x3F)
> action 15: out = (arg << 6)+(v2 & 0x3F)
> action 16: out = (arg << 6)+(v2 & 0x3F)
> action 17: out = (arg << 6)+(v3 & 0x3F)
> action 20: out = (arg << 6)+(v3 & 0x3F)
> action 21: out = (arg << 6)+(v3 & 0x3F)
> action 24: out = (arg << 6)+(v4 & 0x3F)
>
>
> If buffer is empty then it could be 'end of data' or need to signal
> for more data.
>
> We need to ensure the required number of bytes are available when
> the first byte is checked.  Otherwise it is TypeUnknown. (partial data)
>
> Different tituation when at the last buffer - we go in to process the
> remaining bytes even when we could run out mid-stream.
> This avoids a quote escaping attack, such as quote - F0 - quote - newline

*/

use core::iter::Iterator;

use crate::utf8conv::buf::EightBytes;


// Action 9 and 10 are different; action 9 can be an end state, while
// action 10 cannot.

#[inline]
/// Finite state machine action 9; expect 80 to bf
fn byte2_action9(mybuf: & mut EightBytes, arg: u32) -> Utf8EndEnum {
    match mybuf.front() {
        Option::Some(v) => {
            let v2 = v as u32;
            // println!("in action 9 with v2={:#02x}", v2);
            if (v2 >= 0x80) && (v2 <= 0xbf) {
                mybuf.pop_front(); // advance
                Utf8EndEnum::Finish((arg << 6)+(v2 & 0x3F))
            }
            else {
                // println!("not within 0x80 and 0xbf");
                Utf8EndEnum::BadDecode(1)
            }
        }
        Option::None => {
            // println!("TypeUnknown");
            Utf8EndEnum::TypeUnknown
        }
    }
}

// Action 10 and 12 are different; action 10 is for a 3 byte sequence,
// while action 12 is for a 4 byte sequence.

/// Finite state machine action 10; expect 80 to bf
fn byte2_action10(mybuf: & mut EightBytes, arg: u32) -> Utf8EndEnum {
    match mybuf.front() {
        Option::Some(v) => {
            let v2 = v as u32;
            // println!("in action 10 with v2={:#02x}", v2);
            if (v2 >= 0x80) && (v2 <= 0xbf) {
                mybuf.pop_front(); // advance
                byte3_action17(mybuf, (arg << 6)+(v2 & 0x3F))
            }
            else {
                // println!("not within 0x80 and 0xbf");
                Utf8EndEnum::BadDecode(1)
            }
        }
        Option::None => {
            // println!("TypeUnknown");
            Utf8EndEnum::TypeUnknown
        }
    }
}

/// Finite state machine action 11; expect 80 to bf
/// Codepoint E000 to FFFF
fn byte2_action11(mybuf: & mut EightBytes, arg: u32) -> Utf8EndEnum {
    match mybuf.front() {
        Option::Some(v) => {
            let v2 = v as u32;
            // println!("in action 10 with v2={:#02x}", v2);
            if (v2 >= 0x80) && (v2 <= 0xbf) {
                mybuf.pop_front(); // advance
                byte3_action20(mybuf, (arg << 6)+(v2 & 0x3F))
            }
            else {
                // println!("not within 0x80 and 0xbf");
                Utf8EndEnum::BadDecode(1)
            }
        }
        Option::None => {
            // println!("TypeUnknown");
            Utf8EndEnum::TypeUnknown
        }
    }
}

/// Finite state machine action 12; expect 80 to bf
fn byte2_action12(mybuf: & mut EightBytes, arg: u32) -> Utf8EndEnum {
    match mybuf.front() {
        Option::Some(v) => {
            let v2 = v as u32;
            // println!("in action 12 with v2={:#02x}", v2);
            if (v2 >= 0x80) && (v2 <= 0xbf) {
                mybuf.pop_front(); // advance
                byte3_action21(mybuf, (arg << 6)+(v2 & 0x3F))
            }
            else {
                // println!("not within 0x80 and 0xbf");
                Utf8EndEnum::BadDecode(1)
            }
        }
        Option::None => {
            // println!("TypeUnknown");
            Utf8EndEnum::TypeUnknown
        }
    }
}

/// Finite state machine action 13; expect 80 to 8F
fn byte2_action13(mybuf: & mut EightBytes, arg: u32) -> Utf8EndEnum {
    match mybuf.front() {
        Option::Some(v) => {
            let v2 = v as u32;
            // println!("in action 13 with v2={:#02x}", v2);
            if (v2 >= 0x80) && (v2 <= 0x8F) {
                mybuf.pop_front(); // advance
                byte3_action21(mybuf, (arg << 6)+(v2 & 0x3F))
            }
            else {
                // println!("not within 0x80 and 0x8F");
                Utf8EndEnum::BadDecode(1)
            }
        }
        Option::None => {
            // println!("TypeUnknown");
            Utf8EndEnum::TypeUnknown
        }
    }
}

#[inline]
/// Finite state machine action 14; expect A0 to bf
fn byte2_action14(mybuf: & mut EightBytes, arg: u32) -> Utf8EndEnum {
    match mybuf.front() {
        Option::Some(v) => {
            let v2 = v as u32;
            // println!("in action 14 with v2={:#02x}", v2);
            if (v2 >= 0xA0) && (v2 <= 0xbf) {
                mybuf.pop_front(); // advance
                byte3_action17(mybuf, (arg << 6)+(v2 & 0x3F))
            }
            else {
                // println!("not within 0xA0 and 0xbf");
                Utf8EndEnum::BadDecode(1)
            }
        }
        Option::None => {
            // println!("TypeUnknown");
            Utf8EndEnum::TypeUnknown
        }
    }
}

/// Finite state machine action 15; expect 80 to 9F
fn byte2_action15(mybuf: & mut EightBytes, arg: u32) -> Utf8EndEnum {
    match mybuf.front() {
        Option::Some(v) => {
            let v2 = v as u32;
            // println!("in action 15 with v2={:#02x}", v2);
            if (v2 >= 0x80) && (v2 <= 0x9F) {
                mybuf.pop_front(); // advance
                byte3_action17(mybuf, (arg << 6)+(v2 & 0x3F))
            }
            else {
                // println!("not within 0x80 and 0x9F");
                Utf8EndEnum::BadDecode(1)
            }
        }
        Option::None => {
            // println!("TypeUnknown");
            Utf8EndEnum::TypeUnknown
        }
    }
}

/// Finite state machine action 16; expect 90 to bf
fn byte2_action16(mybuf: & mut EightBytes, arg: u32) -> Utf8EndEnum {
    match mybuf.front() {
        Option::Some(v) => {
            let v2 = v as u32;
            // println!("in action 16 with v2={:#02x}", v2);
            if (v2 >= 0x90) && (v2 <= 0xbf) {
                mybuf.pop_front(); // advance
                byte3_action21(mybuf, (arg << 6)+(v2 & 0x3F))
            }
            else {
                // println!("not within 0x90 and 0xbf");
                Utf8EndEnum::BadDecode(1)
            }
        }
        Option::None => {
            // println!("TypeUnknown");
            Utf8EndEnum::TypeUnknown
        }
    }
}

#[inline]
/// Finite state machine action 17; expect 80 to bf
fn byte3_action17(mybuf: & mut EightBytes, arg: u32) -> Utf8EndEnum {
    match mybuf.front() {
        Option::Some(v) => {
            let v3 = v as u32;
            // println!("in action 17 with v3={:#02x}", v3);
            if (v3 >= 0x80) && (v3 <= 0xbf) {
                mybuf.pop_front(); // advance
                Utf8EndEnum::Finish((arg << 6)+(v3 & 0x3F))
            }
            else {
                // println!("not within 0x80 and 0xbf");
                Utf8EndEnum::BadDecode(2)
            }
        }
        Option::None => {
            // println!("TypeUnknown");
            Utf8EndEnum::TypeUnknown
        }
    }
}

#[inline]
/// Finite state machine action 20 expect 80 to bf
/// Codepoint E000 to FFFF
fn byte3_action20(mybuf: & mut EightBytes, arg: u32) -> Utf8EndEnum {
    match mybuf.front() {
        Option::Some(v) => {
            let v3 = v as u32;
            // println!("in action 20 with v3={:#02x}", v3);
            if (v3 >= 0x80) && (v3 <= 0xbf) {
                mybuf.pop_front(); // advance
                let codepoint = (arg << 6) + (v3 & 0x3F);
                if codepoint == REPLACE_UTF32 {
                    // special processing logic for replacement character:
                    //
                    // Logic was that a replacement character represents a
                    // former invalid encoding or decoding of a codepoint.
                    // We treat them the same whether this was triggered
                    // fresh or from historical data source.
                    //
                    // BadDecode(3) means this event was detected after
                    // parsing 3 bytes. (EF, BF, BD)
                    Utf8EndEnum::BadDecode(3)
                }
                else {
                    Utf8EndEnum::Finish(codepoint)
                }
            }
            else {
                // println!("not within 0x80 and 0xbf");
                Utf8EndEnum::BadDecode(2)
            }
        }
        Option::None => {
            // println!("TypeUnknown");
            Utf8EndEnum::TypeUnknown
        }
    }
}

#[inline]
/// Finite state machine action 21; expect 80 to bf
fn byte3_action21(mybuf: & mut EightBytes, arg: u32) -> Utf8EndEnum {
    match mybuf.front() {
        Option::Some(v) => {
            let v3 = v as u32;
            // println!("in action 21 with v3={:#02x}", v3);
            if (v3 >= 0x80) && (v3 <= 0xbf) {
                mybuf.pop_front(); // advance
                byte4_action24(mybuf, (arg << 6)+(v3 & 0x3F))
            }
            else {
                // println!("not within 0x80 and 0xbf");
                Utf8EndEnum::BadDecode(2)
            }
        }
        Option::None => {
            // println!("TypeUnknown");
            Utf8EndEnum::TypeUnknown
        }
    }
}

#[inline]
/// Finite state machine action 24; expect 80 to bf
fn byte4_action24(mybuf: & mut EightBytes, arg: u32) -> Utf8EndEnum {
    match mybuf.front() {
        Option::Some(v) => {
            let v4 = v as u32;
            // println!("in action 24 with v4={:#02x}", v4);
            if (v4 >= 0x80) && (v4 <= 0xbf) {
                mybuf.pop_front(); // advance
                Utf8EndEnum::Finish((arg << 6)+(v4 & 0x3F))
            }
            else {
                // println!("not within 0x80 and 0xbf");
                Utf8EndEnum::BadDecode(3)
            }
        }
        Option::None => {
            // println!("TypeUnknown");
            Utf8EndEnum::TypeUnknown
        }
    }
}

/// Decode from UTF8 to Unicode code point using a finate state machine.
///
/// # Arguments
///
/// * `mybuf` - contains the bytes to be decoded
///
/// * `last_buffer` - is true when we are working on the last byte buffer.
///
/// When 'last_buffer' is false, with additional buffers to be processed,
/// then the parser would refuse to work on potential partial decodes,
/// and returns Utf8EndEnum::TypeUnknown to ask for more data.
///
/// When 'last_buffer' is true, with no more data to process than
/// what is available in 'mybuf', then partial decodes results in
/// Utf8EndEnum:BadDecode(n) where n is length of error from 1 to 3 bytes.
pub fn utf8_decode(mybuf: & mut EightBytes, last_buffer: bool) -> Utf8EndEnum {
    match mybuf.front() {
        Option::Some(v) => {
            let v1 = v as u32;
            // println!("in start state with v1={:#02x} and len()={}", v1, mybuf.len());
            if v1 < 0xE0 {
                if v1 < 0xC2 {
                    mybuf.pop_front();
                    if v1 < 0x80 {
                        // Action 0
                        // 1 byte format: code point from 0x0 to 0x7F
                        // println!("in action 0 with v1={:#02x}", v1);
                        Utf8EndEnum::Finish(v1)
                    }
                    else {
                        // 80 to C1: not valid first byte
                        // println!("80 to C1 bad decode");
                        Utf8EndEnum::BadDecode(1)
                    }
                }
                else {
                    // Byte 1 is between 0xC2 and 0xDF
                    // 2 byte format
                    if (mybuf.len() < 2) && ! last_buffer {
                        // We wait for more bytes if not the last buffer.
                        // Our design cannot back-out procesed bytes.
                        // println!("TypeUnknown");
                        Utf8EndEnum::TypeUnknown
                    }
                    else {
                        // Action 1
                        // println!("in action 1 with v1={:#02x}", v1);
                        mybuf.pop_front();
                        byte2_action9(mybuf, v1 & 0x1F)
                    }
                }
            }
            else {
                if v1 < 0xF0 {
                    // 3 byte format
                    // Byte 1 is between 0xE0 and 0xEF
                    if (mybuf.len() < 3) && ! last_buffer {
                        // We wait for more bytes if not the last buffer.
                        // Our design cannot back-out procesed bytes.
                        // println!("TypeUnknown");
                        Utf8EndEnum::TypeUnknown
                    }
                    else if v1 < 0xED {
                        mybuf.pop_front();
                        if v1 == 0xE0 {
                            // Action 2
                            // v1 is 0xE0.
                            // println!("in action 2 with v1={:#02x}", v1);
                            byte2_action14(mybuf, v1 & 0xF)
                        }
                        else {
                            // Action 3
                            // v1 is between 0xE1 and 0xEC.
                            // println!("in action 3 with v1={:#02x}", v1);
                            byte2_action10(mybuf, v1 & 0xF)
                        }
                    }
                    else {
                        mybuf.pop_front();
                        if v1 == 0xED {
                            // Action 4
                            // println!("in action 4 with v1={:#02x}", v1);
                            byte2_action15(mybuf, v1 & 0xF)
                        }
                        else {
                            // Action 5
                            // v1 is 0xEE or 0xEF.
                            // println!("in action 5 with v1={:#02x}", v1);
                            byte2_action11(mybuf, v1 & 0xF)
                        }
                    }
                }
                else {
                    // 4 byte cases if byte 1 is between 0xF0 and 0xF4
                    if v1 > 0xF4 {
                        // codepoint too large
                        // println!("greater than F4 bad decode");
                        mybuf.pop_front();
                        Utf8EndEnum::BadDecode(1)
                    }
                    else if (mybuf.len() < 4) && ! last_buffer {
                        // We wait for more bytes if not the last buffer.
                        // Our design cannot back-out procesed bytes.
                        // println!("TypeUnknown");
                        Utf8EndEnum::TypeUnknown
                    }
                    else {
                        mybuf.pop_front();
                        if v1 == 0xF0 {
                            // Action 6
                            // println!("in action 6 with v1={:#02x}", v1);
                            byte2_action16(mybuf, v1 & 0x7)
                        }
                        else if v1 < 0xF4 {
                            // Action 7
                            // Byte 1 is between 0xF1 and 0xF3.
                            // println!("in action 7 with v1={:#02x}", v1);
                            byte2_action12(mybuf, v1 & 0x7)
                        }
                        else {
                            // Action 8
                            // Byte 1 is 0xF4.
                            // println!("in action 8 with v1={:#02x}", v1);
                            byte2_action13(mybuf, v1 & 0x7)
                        }
                    }
                }
            }
        }
        Option::None => {
            // println!("TypeUnknown");
            Utf8EndEnum::TypeUnknown
        }
    }
}


/// Most iterators on arrays allocated on the stack returns a reference
/// in order to save memory.  For our converter use-case this is a
/// problem because our conversion result is a temporary value that
/// is best delivered as a value, not as a reference.
/// This could cause two iterators failing to connect from one output to
/// the next input.
///
/// Proposed types of converters:
///
/// utf8 ref -> char (direct route)
///
/// char ref -> utf8 (another direct route)
///
/// ref of char -> char
///
/// utf32 ref -> utf32
///
/// utf8 ref -> utf8
///
/// char -> utf32
///
/// utf32 -> utf8
///
/// utf8 -> char
///
/// char reference to char iterator struct
pub struct CharRefToCharStruct<'b> {
    my_borrow_mut_iter: &'b mut dyn Iterator<Item = &'b char>,
}

/// an adapter iterator to convert a char ref iterator to char iterator
impl<'b> Iterator for CharRefToCharStruct<'b> {
    type Item=char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.my_borrow_mut_iter.next() {
            Option::None => { Option::None }
            Option::Some(v) => { Option::Some(* v) }
        }
    }

    /// sizing hint for iterator, with a lower bound and optional upperbound
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.my_borrow_mut_iter.size_hint()
    }
}

/// Function char_ref_iter_to_char_iter() takes a mutable reference to
/// a char ref iterator, and return a char iterator in its place.
///
/// # Arguments
///
/// * `input` - a mutable reference to a char ref iterator
#[inline]
pub fn char_ref_iter_to_char_iter<'a, I: 'a + Iterator>(input: &'a mut I)
-> CharRefToCharStruct<'a>
where I: Iterator<Item = &'a char>, {
    CharRefToCharStruct {
        my_borrow_mut_iter: input,
    }
}

/// UTF32 reference to UTF32 iterator struct
pub struct Utf32RefToUtf32Struct<'b> {
    my_borrow_mut_iter: &'b mut dyn Iterator<Item = &'b u32>,
}

/// an adapter iterator to convert a UTF32 ref iterator to UTF32 iterator
impl<'b> Iterator for Utf32RefToUtf32Struct<'b> {
    type Item=u32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.my_borrow_mut_iter.next() {
            Option::None => { Option::None }
            Option::Some(v) => { Option::Some(* v) }
        }
    }

    /// sizing hint for iterator, with a lower bound and optional upperbound
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.my_borrow_mut_iter.size_hint()
    }
}

/// Function utf32_ref_iter_to_utf32_iter() takes a mutable reference to
/// a UTF32 ref iterator, and return a UTF32 iterator in its place.
///
/// # Arguments
///
/// * `input` - a mutable reference to a UTF32 ref iterator
#[inline]
pub fn utf32_ref_iter_to_utf32_iter<'a, I: 'a + Iterator>(input: &'a mut I)
-> Utf32RefToUtf32Struct<'a>
where I: Iterator<Item = &'a u32>, {
    Utf32RefToUtf32Struct {
        my_borrow_mut_iter: input,
    }
}

/// UTF8 reference to UTF8 iterator struct
pub struct Utf8RefToUtf8Struct<'b> {
    my_borrow_mut_iter: &'b mut dyn Iterator<Item = &'b u8>,
}

/// an adapter iterator to convert a UTF8 ref iterator to UTF8 iterator
impl<'b> Iterator for Utf8RefToUtf8Struct<'b> {
    type Item=u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.my_borrow_mut_iter.next() {
            Option::None => { Option::None }
            Option::Some(v) => { Option::Some(* v) }
        }
    }

    /// sizing hint for iterator, with a lower bound and optional upperbound
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.my_borrow_mut_iter.size_hint()
    }
}

/// Function utf8_ref_iter_to_utf8_iter() takes a mutable reference to
/// a UTF8 ref iterator, and return a UTF8 iterator in its place.
///
/// # Arguments
///
/// * `input` - a mutable reference to a UTF8 ref iterator
#[inline]
pub fn utf8_ref_iter_to_utf8_iter<'a, I: 'a + Iterator>(input: &'a mut I)
-> Utf8RefToUtf8Struct<'a>
where I: Iterator<Item = &'a u8>, {
    Utf8RefToUtf8Struct {
        my_borrow_mut_iter: input,
    }
}

/// char to UTF32 iterator struct
pub struct CharToUtf32Struct<'b> {
    my_borrow_mut_iter: &'b mut dyn Iterator<Item = char>,
}

/// an adapter iterator to convert a char iterator to UTF32 iterator
impl<'b> Iterator for CharToUtf32Struct<'b> {
    type Item=u32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.my_borrow_mut_iter.next() {
            Option::None => { Option::None }
            Option::Some(v) => { Option::Some(v as u32) }
        }
    }

    /// sizing hint for iterator, with a lower bound and optional upperbound
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.my_borrow_mut_iter.size_hint()
    }
}

/// Function char_iter_to_utf32_iter() takes a mutable reference to
/// a char iterator, and return a UTF32 iterator in its place.
///
/// # Arguments
///
/// * `input` - a mutable reference to a char iterator
#[inline]
pub fn char_iter_to_utf32_iter<'a, I: 'a + Iterator>(input: &'a mut I)
-> CharToUtf32Struct<'a>
where I: Iterator<Item = char>, {
    CharToUtf32Struct {
        my_borrow_mut_iter: input,
    }
}

/// Common operations for UTF conversion parsers
pub trait UtfParserCommon {

    /// Reset all parser states to the initial value.
    /// Last buffer indication is set to true.
    /// Invalid decodes indication is cleared.
    fn reset_parser(&mut self);

    /// If argument `b` is true, then any input buffer to be presented will
    /// be the last buffer.
    fn set_is_last_buffer(&mut self, b: bool);

    /// Returns the last input buffer flag.
    fn is_last_buffer(&self) -> bool;

    /// This function signals the occurrence of an invalid conversion sequence.
    fn signal_invalid_sequence(& mut self);

    /// This function returns true if invalid conversion sequence occurred
    /// in this parsing stream.
    fn has_invalid_sequence(&self) -> bool;

    /// This function resets the invalid sequence state.
    fn reset_invalid_sequence(& mut self);
}

/// Provides conversion functions from UTF8 to char or UTF32
#[derive(Debug, Clone, Copy)]
pub struct FromUtf8 {
    my_buf: EightBytes,
    my_last_buffer: bool,
    my_invalid_sequence: bool,
}

/// Provides conversion functions from char or UTF32 to UTF8
#[derive(Debug, Clone, Copy)]
pub struct FromUnicode {
    my_buf: EightBytes,
    my_last_buffer: bool,
    my_invalid_sequence: bool,
}

/// adapter iterator converting from an UTF8 iterator to a char iterator
/// (This iterator contains a mutable borrow to the launching
/// FromUtf8 object while this iterator is alive.)
pub struct Utf8IterToCharIter<'p> {
    my_borrow_mut_iter: &'p mut dyn Iterator<Item = u8>,
    my_info: &'p mut FromUtf8,
}

/// adapter iterator converting from an UTF32 iterator to an UTF8 iterator
/// (This iterator contains a mutable borrow to the launching
/// FromUnicode object while this iterator is alive.)
pub struct Utf32IterToUtf8Iter<'q> {
    my_borrow_mut_iter: &'q mut dyn Iterator<Item = u32>,
    my_info: &'q mut FromUnicode,
}

/// adapter iterator converting from an UTF8 ref iterator to char iterator
/// (This iterator contains a mutable borrow to the launching
/// FromUtf8 object while this iterator is alive.)
pub struct Utf8RefIterToCharIter<'r> {
    my_borrow_mut_iter: &'r mut dyn Iterator<Item = &'r u8>,
    my_info: &'r mut FromUtf8,
}

/// adapter iterator converting from a char ref iterator to an UTF8 iterator
/// (This iterator contains a mutable borrow to the launching
/// FromUnicode object while this iterator is alive.)
pub struct CharRefIterToUtf8Iter<'s> {
    my_borrow_mut_iter: &'s mut dyn Iterator<Item = &'s char>,
    my_info: &'s mut FromUnicode,
}

/// Implementations of common operations for FromUtf8
impl<'b> UtfParserCommon for FromUtf8 {

    #[inline]
    /// If argument `b` is true, then any input buffer to be presented will
    /// be the last buffer.
    fn set_is_last_buffer(&mut self, b: bool) {
        self.my_last_buffer = b;
    }

    #[inline]
    /// Returns the last input buffer flag.
    fn is_last_buffer(&self) -> bool {
        self.my_last_buffer
    }

    #[inline]
    /// This function returns true if invalid UTF8 sequence occurred
    /// in this parsing stream.
    fn has_invalid_sequence(&self) -> bool {
        self.my_invalid_sequence
    }

    #[inline]
    /// This function signals the occurrence of an invalid UTF8 sequence.
    fn signal_invalid_sequence(&mut self) {
        self.my_invalid_sequence = true;
    }

    #[inline]
    /// This function resets the invalid decodes state.
    fn reset_invalid_sequence(& mut self) {
        self.my_invalid_sequence = false;
    }

    #[inline]
    /// Reset all parser states to the initial value.
    /// Last buffer indication is set to true.
    /// Invalid decodes indication is cleared.
    fn reset_parser(&mut self) {
        // Drain our buffer.
        self.my_buf.clear();
        self.set_is_last_buffer(true);
        self.reset_invalid_sequence();
    }

}

/// Implementations of common operations for FromUnicode
impl<'b> UtfParserCommon for FromUnicode {

    #[inline]
    /// If argument `b` is true, then any input buffer to be presented will
    /// be the last buffer.
    fn set_is_last_buffer(&mut self, b: bool) {
        self.my_last_buffer = b;
    }

    #[inline]
    /// Returns the last input buffer flag.
    fn is_last_buffer(&self) -> bool {
        self.my_last_buffer
    }

    #[inline]
    /// This function returns true if invalid UTF32 decodes occurred in this
    /// parsing stream.
    fn has_invalid_sequence(&self) -> bool {
        self.my_invalid_sequence
    }

    #[inline]
    /// This function signals the occurrence of an invalid UTF32 sequence.
    fn signal_invalid_sequence(&mut self) {
        self.my_invalid_sequence = true;
    }

    #[inline]
    /// This function resets the invalid sequence state.
    fn reset_invalid_sequence(&mut self) {
        self.my_invalid_sequence = false;
    }

    #[inline]
    /// Reset all parser states to the initial value.
    /// Last buffer indication is set to true.
    /// Invalid sequence indication is cleared.
    fn reset_parser(&mut self) {
        // Drain our buffer.
        self.my_buf.clear();
        self.set_is_last_buffer(true);
        self.reset_invalid_sequence();
    }

}

/// Map a char parsing result to a UTF32 parsing result.
pub fn parse_mapper_char_to_utf32(input: Result<(& [u8], char), MoreEnum>)
-> Result<(& [u8], u32), MoreEnum> {
    match input {
        Result::Err(e) => { Result::Err(e) }
        Result::Ok((new_spot, ch)) => { Ok((new_spot, ch as u32)) }
    }
}

/// Implementation of FromUtf8
impl FromUtf8 {

    /// Make a new FromUtf8
    pub fn new() -> FromUtf8 {
        FromUtf8 {
            my_buf : EightBytes::new(),
            my_last_buffer : true,
            my_invalid_sequence : false,
        }
    }

    /// A parser takes in byte slice, and returns a Result object with
    /// either the remaining input and the output char value, or an MoreEnum
    /// that requests additional data, or an end of data stream condition.
    ///
    /// Invalid UTF8 decodes are indicated by Unicode replacement characters.
    /// has_invalid_decodes() would return true after this event.
    /// Encountering a replacement character is considered the same as having
    /// an invalid decode.
    pub fn utf8_to_char<'b>(&mut self, input: &'b [u8])
    -> Result<(&'b [u8], char), MoreEnum> {
        let mut my_cursor: &[u8] = input;
        let last_buffer = self.my_last_buffer;
        // Fill buffer phase.
        loop {
            if self.my_buf.is_full() || (my_cursor.len() == 0) {
                break;
            }
            // Push a u8, and advance input position.
            self.my_buf.push_back(my_cursor[0]);
            my_cursor = &my_cursor[1..];
        }
        if self.my_buf.is_empty() {
            // Processing for buffer being empty case
            // Determine if we are at end of data.
            if last_buffer {
                // at end of data condition
                Result::Err(MoreEnum::More(0))
            }
            else {
                // Returning an indication to request a new buffer.
                Result::Err(MoreEnum::More(4096))
            }
        }
        else {
            match utf8_decode(& mut self.my_buf, last_buffer) {
                Utf8EndEnum::BadDecode(_) => {
                    self.signal_invalid_sequence();
                    Result::Ok((my_cursor, char::REPLACEMENT_CHARACTER))
                }
                Utf8EndEnum::Finish(code) => {
                    // Unsafe is justified because utf8_decode() finite state
                    // machine checks for all cases of invalid decodes.
                    let ch = unsafe { char::from_u32_unchecked(code) };
                    Result::Ok((my_cursor, ch))
                }
                Utf8EndEnum::TypeUnknown => {
                    // Insufficient data to decode.
                    if last_buffer {
                        self.signal_invalid_sequence();
                        // Buffer should be empty at this point.
                        Result::Ok((my_cursor, char::REPLACEMENT_CHARACTER))
                    }
                    else {
                        // Return an indication to request a new buffer.
                        Result::Err(MoreEnum::More(4096))
                    }
                }
            }
        }
    }

    /// A parser takes in byte slice, and returns a Result object with
    /// either the remaining input and the output u32 value, or an MoreEnum
    /// that requests additional data, or an end of data stream condition.
    ///
    /// Invalid UTF8 decodes are indicated by Unicode replacement characters.
    /// has_invalid_decodes() would return true after this event.
    /// Encountering a replacement character is considered the same as having
    /// an invalid decode.
    pub fn utf8_to_utf32<'c>(&mut self, input: &'c [u8])
    -> Result<(&'c [u8], u32), MoreEnum> {
        let char_parse_result = self.utf8_to_char(input);
        parse_mapper_char_to_utf32(char_parse_result)
    }

    /// Convert from UTF8 to char with a mutable reference
    /// to the source UTF8 iterator.
    pub fn utf8_to_char_with_iter<'d>(&'d mut self, iter: &'d mut dyn Iterator<Item = u8>)
    -> Utf8IterToCharIter {
        Utf8IterToCharIter {
            my_info : self,
            my_borrow_mut_iter: iter,
        }
    }

    /// Convert from UTF8 ref to char with a mutable reference
    /// to the source UTF8 iterator.
    pub fn utf8_ref_to_char_with_iter<'d>(&'d mut self, iter: &'d mut dyn Iterator<Item = &'d u8>)
    -> Utf8RefIterToCharIter {
        Utf8RefIterToCharIter {
            my_info : self,
            my_borrow_mut_iter: iter,
        }
    }

}


/// Implementation of FromUnicode
impl FromUnicode {

    /// Make a new FromUnicode
    pub fn new() -> FromUnicode {
        FromUnicode {
            my_buf : EightBytes::new(),
            my_last_buffer : true,
            my_invalid_sequence : false,
        }
    }

    /// A parser takes in char slice, and returns a Result object with
    /// either the remaining input and the output byte value, or an MoreEnum
    /// that requests additional data, or an end of data stream condition.
    ///
    /// Invalid UTF32 decodes are indicated by Unicode replacement characters.
    /// has_invalid_decodes() would return true after this event.
    /// Encountering a replacement character is considered the same as having
    /// an invalid decode.
    pub fn char_to_utf8<'b>(&mut self, input: &'b [char])
    -> Result<(&'b [char], u8), MoreEnum> {
        // Check if we can pull an u8 from our ring buffer
        match self.my_buf.pop_front() {
            Some(v1) => {
                return Result::Ok((input, v1));
            }
            None => {}
        }
        let mut my_cursor: &[char] = input;
        // Processing for input being empty case
        if my_cursor.len() == 0 {
            // Determine if we are at end of data.
            if self.is_last_buffer() {
                // at end of data condition
                return Result::Err(MoreEnum::More(0));
            }
            else {
                // Returning an indication to request a new buffer.
                return Result::Err(MoreEnum::More(1024));
            }
        }
        // Grab one UTF32 from input
        let cur_u32 = my_cursor[0] as u32;
        my_cursor = &my_cursor[1..];
        // Try to determine the type of UTF32 encoding.
        match classify_utf32(cur_u32) {
            Utf8TypeEnum::Type1(v1) => {
                Result::Ok((my_cursor, v1))
            }
            Utf8TypeEnum::Type2((v1,v2)) => {
                self.my_buf.push_back(v2);
                Result::Ok((my_cursor, v1))
            }
            Utf8TypeEnum::Type3((v1,v2,v3)) => {
                self.my_buf.push_back(v2);
                self.my_buf.push_back(v3);
                Result::Ok((my_cursor, v1))
            }
            Utf8TypeEnum::Type4((v1,v2,v3,v4)) => {
                self.my_buf.push_back(v2);
                self.my_buf.push_back(v3);
                self.my_buf.push_back(v4);
                Result::Ok((my_cursor, v1))
            }
            _ => {
                // Invalid UTF32 codepoint
                // Emit replacement byte sequence.
                self.signal_invalid_sequence();
                self.my_buf.push_back(REPLACE_PART2);
                self.my_buf.push_back(REPLACE_PART3);
                Result::Ok((my_cursor, REPLACE_PART1))
            }
        }
    }

    /// A parser takes in UTF32 slice, and returns a Result object with
    /// either the remaining input and the output byte value, or an MoreEnum
    /// that requests additional data, or an end of data stream condition.
    ///
    /// Invalid UTF32 decodes are indicated by Unicode replacement characters.
    /// has_invalid_decodes() would return true after this event.
    /// Encountering a replacement character is considered the same as having
    /// an invalid decode.
    pub fn utf32_to_utf8<'c>(&mut self, input: &'c [u32])
    -> Result<(&'c [u32], u8), MoreEnum> {
        // Check if we can pull an u8 from our ring buffer
        match self.my_buf.pop_front() {
            Some(v1) => {
                return Result::Ok((input, v1));
            }
            None => {}
        }
        let mut my_cursor: &[u32] = input;
        // Processing for input being empty case
        if my_cursor.len() == 0 {
            // Determine if we are at end of data.
            if self.is_last_buffer() {
                // at end of data condition
                return Result::Err(MoreEnum::More(0));
            }
            else {
                // Returning an indication to request a new buffer.
                return Result::Err(MoreEnum::More(1024));
            }
        }
        // Grab one UTF32 from input
        let cur_u32 = my_cursor[0];
        my_cursor = &my_cursor[1..];
        // Try to determine the type of UTF32 encoding.
        match classify_utf32(cur_u32) {
            Utf8TypeEnum::Type1(v1) => {
                Result::Ok((my_cursor, v1))
            }
            Utf8TypeEnum::Type2((v1,v2)) => {
                self.my_buf.push_back(v2);
                Result::Ok((my_cursor, v1))
            }
            Utf8TypeEnum::Type3((v1,v2,v3)) => {
                self.my_buf.push_back(v2);
                self.my_buf.push_back(v3);
                Result::Ok((my_cursor, v1))
            }
            Utf8TypeEnum::Type4((v1,v2,v3,v4)) => {
                self.my_buf.push_back(v2);
                self.my_buf.push_back(v3);
                self.my_buf.push_back(v4);
                Result::Ok((my_cursor, v1))
            }
            _ => {
                // Invalid UTF32 codepoint
                // Emit replacement byte sequence.
                self.signal_invalid_sequence();
                self.my_buf.push_back(REPLACE_PART2);
                self.my_buf.push_back(REPLACE_PART3);
                Result::Ok((my_cursor, REPLACE_PART1))
            }
        }
    }

    /// Convert from UTF32 iter to UTF8 iter with a mutable reference
    /// to the source UTF32 iterator.
    pub fn utf32_to_utf8_with_iter<'d>(&'d mut self, iter: &'d mut dyn Iterator<Item = u32>)
    -> Utf32IterToUtf8Iter {
        Utf32IterToUtf8Iter {
            my_borrow_mut_iter: iter,
            my_info: self,
        }
    }

    /// Convert from char ref iter to UTF8 iter with a mutable reference
    /// to the source char ref iterator.
    pub fn char_ref_to_utf8_with_iter<'d>(&'d mut self, iter: &'d mut dyn Iterator<Item = &'d char>)
    -> CharRefIterToUtf8Iter {
        CharRefIterToUtf8Iter {
            my_borrow_mut_iter: iter,
            my_info: self,
        }
    }

}

/// Implementations of common operations for Utf8IterToCharIter
impl<'g> UtfParserCommon for Utf8IterToCharIter<'g> {

    #[inline]
    /// If argument `b` is true, then any input buffer to be presented will
    /// be the last buffer.
    fn set_is_last_buffer(&mut self, b: bool) {
        self.my_info.set_is_last_buffer(b);
    }

    #[inline]
    /// Returns the last input buffer flag.
    fn is_last_buffer(&self) -> bool {
        self.my_info.is_last_buffer()
    }

    #[inline]
    /// This function returns true if invalid UTF8 sequence occurred
    /// in this parsing stream.
    fn has_invalid_sequence(&self) -> bool {
        self.my_info.has_invalid_sequence()
    }

    #[inline]
    /// This function signals the occurrence of an invalid UTF8 sequence.
    fn signal_invalid_sequence(&mut self) {
        self.my_info.signal_invalid_sequence();
    }

    #[inline]
    /// This function resets the invalid decodes state.
    fn reset_invalid_sequence(& mut self) {
        self.my_info.reset_invalid_sequence();
    }

    #[inline]
    /// Reset all parser states to the initial value.
    /// Last buffer indication is set to true.
    /// Invalid decodes indication is cleared.
    fn reset_parser(&mut self) {
        self.my_info.reset_parser();
    }
}

/// Iterator for Utf8IterToCharIter
impl<'g> Iterator for Utf8IterToCharIter<'g> {
    type Item = char;

    /// A parser takes in an iterator of UTF8 byte stream, and returns
    /// an iterator of char values.
    ///
    /// An invalid Unicode decode in the stream are substituted with
    /// an Unicode replacement character.
    ///
    /// has_invalid_sequence() would return true after observing
    /// invalid decodes, or observing a replacement character.
    fn next(&mut self) -> Option<Self::Item> {
        // Fill buffer phase.
        loop {
            if self.my_info.my_buf.is_full() {
                break;
            }
            match self.my_borrow_mut_iter.next() {
                Option::None => {
                    break;
                }
                Option::Some(utf8) => {
                    // Save it in our scratch pad.
                    self.my_info.my_buf.push_back(utf8);
                }
            }
        }
        if self.my_info.my_buf.is_empty() {
            // This is either the end of data, or the current buffer
            // has run to the end without left-over data in the
            // scratch pad.
            Option::None
        }
        else {
            let last_buffer = self.my_info.is_last_buffer();
            match utf8_decode(& mut self.my_info.my_buf, last_buffer) {
                Utf8EndEnum::BadDecode(_) => {
                    self.my_info.signal_invalid_sequence();
                    Option::Some(char::REPLACEMENT_CHARACTER)
                }
                Utf8EndEnum::Finish(code) => {
                    // Unsafe is justified because utf8_decode() finite state
                    // machine checks for all cases of invalid decodes.
                    let ch = unsafe { char::from_u32_unchecked(code) };
                    Option::Some(ch)
                }
                Utf8EndEnum::TypeUnknown => {
                    // Insufficient data to decode.
                    if last_buffer {
                        self.my_info.signal_invalid_sequence();
                        // Buffer should be empty at this point.
                        Option::Some(char::REPLACEMENT_CHARACTER)
                    }
                    else {
                        // Ready for next buffer
                        Option::None
                    }
                }
            }
        }
    }

    /// sizing hint for iterator, with a lower bound and optional upperbound
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.my_borrow_mut_iter.size_hint()
    }
}

/// Implementations of common operations for Utf8RefIterToCharIter
impl<'g> UtfParserCommon for Utf8RefIterToCharIter<'g> {

    #[inline]
    /// If argument `b` is true, then any input buffer to be presented will
    /// be the last buffer.
    fn set_is_last_buffer(&mut self, b: bool) {
        self.my_info.set_is_last_buffer(b);
    }

    #[inline]
    /// Returns the last input buffer flag.
    fn is_last_buffer(&self) -> bool {
        self.my_info.is_last_buffer()
    }

    #[inline]
    /// This function returns true if invalid UTF8 sequence occurred
    /// in this parsing stream.
    fn has_invalid_sequence(&self) -> bool {
        self.my_info.has_invalid_sequence()
    }

    #[inline]
    /// This function signals the occurrence of an invalid UTF8 sequence.
    fn signal_invalid_sequence(&mut self) {
        self.my_info.signal_invalid_sequence();
    }

    #[inline]
    /// This function resets the invalid decodes state.
    fn reset_invalid_sequence(& mut self) {
        self.my_info.reset_invalid_sequence();
    }

    #[inline]
    /// Reset all parser states to the initial value.
    /// Last buffer indication is set to true.
    /// Invalid decodes indication is cleared.
    fn reset_parser(&mut self) {
        self.my_info.reset_parser();
    }
}

/// Iterator for Utf8RefIterToCharIter
impl<'g> Iterator for Utf8RefIterToCharIter<'g> {
    type Item = char;

    /// A parser takes in an iterator of UTF8 byte stream, and returns
    /// an iterator of char values.
    ///
    /// An invalid Unicode decode in the stream are substituted with
    /// an Unicode replacement character.
    ///
    /// has_invalid_sequence() would return true after observing
    /// invalid decodes, or observing a replacement character.
    fn next(&mut self) -> Option<Self::Item> {
        // Fill buffer phase.
        loop {
            if self.my_info.my_buf.is_full() {
                break;
            }
            match self.my_borrow_mut_iter.next() {
                Option::None => {
                    break;
                }
                Option::Some(utf8) => {
                    // Save it in our scratch pad.
                    self.my_info.my_buf.push_back(* utf8);
                }
            }
        }
        if self.my_info.my_buf.is_empty() {
            // This is either the end of data, or the current buffer
            // has run to the end without left-over data in the
            // scratch pad.
            Option::None
        }
        else {
            let last_buffer = self.my_info.is_last_buffer();
            match utf8_decode(& mut self.my_info.my_buf, last_buffer) {
                Utf8EndEnum::BadDecode(_) => {
                    self.my_info.signal_invalid_sequence();
                    Option::Some(char::REPLACEMENT_CHARACTER)
                }
                Utf8EndEnum::Finish(code) => {
                    // Unsafe is justified because utf8_decode() finite state
                    // machine checks for all cases of invalid decodes.
                    let ch = unsafe { char::from_u32_unchecked(code) };
                    Option::Some(ch)
                }
                Utf8EndEnum::TypeUnknown => {
                    // Insufficient data to decode.
                    if last_buffer {
                        self.my_info.signal_invalid_sequence();
                        // Buffer should be empty at this point.
                        Option::Some(char::REPLACEMENT_CHARACTER)
                    }
                    else {
                        // Ready for next buffer
                        Option::None
                    }
                }
            }
        }
    }

    /// sizing hint for iterator, with a lower bound and optional upperbound
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.my_borrow_mut_iter.size_hint()
    }
}

/// Implementations of common operations for Utf32IterToUtf8Iter
impl<'h> UtfParserCommon for Utf32IterToUtf8Iter<'h> {

    #[inline]
    /// If argument `b` is true, then any input buffer to be presented will
    /// be the last buffer.
    fn set_is_last_buffer(&mut self, b: bool) {
        self.my_info.set_is_last_buffer(b);
    }

    #[inline]
    /// Returns the last input buffer flag.
    fn is_last_buffer(&self) -> bool {
        self.my_info.is_last_buffer()
    }

    #[inline]
    /// This function returns true if invalid UTF32 sequence occurred
    /// in this parsing stream.
    fn has_invalid_sequence(&self) -> bool {
        self.my_info.has_invalid_sequence()
    }

    #[inline]
    /// This function signals the occurrence of an invalid UTF32 sequence.
    fn signal_invalid_sequence(&mut self) {
        self.my_info.signal_invalid_sequence();
    }

    #[inline]
    /// This function resets the invalid decodes state.
    fn reset_invalid_sequence(& mut self) {
        self.my_info.reset_invalid_sequence();
    }

    #[inline]
    /// Reset all parser states to the initial value.
    /// Last buffer indication is set to true.
    /// Invalid decodes indication is cleared.
    fn reset_parser(&mut self) {
        self.my_info.reset_parser();
    }
}

/// Iterator for Utf32IterToUtf8Iter
impl<'h> Iterator for Utf32IterToUtf8Iter<'h> {
    type Item = u8;

    /// A parser takes in an iterator of Unicode codepoints, and returns
    /// the output UTF8 byte value.
    ///
    /// An invalid Unicode codepoint in the stream are substituted with
    /// an Unicode replacement character.
    ///
    /// has_invalid_sequence() would return true after observing
    /// invalid decodes, or observing a replacement character.
    fn next(&mut self) -> Option<Self::Item> {
        // Check if we can pull an u8 from our ring buffer.
        match self.my_info.my_buf.pop_front() {
            Option::Some(v1) => {
                return Option::Some(v1);
            }
            Option::None => {}
        }
        // Processing for input being empty case
        match self.my_borrow_mut_iter.next() {
            Option::None => {
                return Option::None;
            }
            Option::Some(utf32) => {
                // Try to determine the type of UTFf32 encoding.
                match classify_utf32(utf32) {
                    Utf8TypeEnum::Type1(v1) => {
                        Option::Some(v1)
                    }
                    Utf8TypeEnum::Type2((v1,v2)) => {
                        self.my_info.my_buf.push_back(v2);
                        Option::Some(v1)
                    }
                    Utf8TypeEnum::Type3((v1,v2,v3)) => {
                        self.my_info.my_buf.push_back(v2);
                        self.my_info.my_buf.push_back(v3);
                        Option::Some(v1)
                    }
                    Utf8TypeEnum::Type4((v1,v2,v3,v4)) => {
                        self.my_info.my_buf.push_back(v2);
                        self.my_info.my_buf.push_back(v3);
                        self.my_info.my_buf.push_back(v4);
                        Option::Some(v1)
                    }
                    _ => {
                        // Invalid UTF32 codepoint
                        // Emit replacement byte sequence.
                        self.my_info.signal_invalid_sequence();
                        self.my_info.my_buf.push_back(REPLACE_PART2);
                        self.my_info.my_buf.push_back(REPLACE_PART3);
                        Option::Some(REPLACE_PART1)
                    }
                }
            }
        }
    }

    /// sizing hint for iterator, with a lower bound and optional upperbound
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.my_borrow_mut_iter.size_hint()
    }

}

/// Implementations of common operations for CharRefIterToUtf8Iter
impl<'h> UtfParserCommon for CharRefIterToUtf8Iter<'h> {

    #[inline]
    /// If argument `b` is true, then any input buffer to be presented will
    /// be the last buffer.
    fn set_is_last_buffer(&mut self, b: bool) {
        self.my_info.set_is_last_buffer(b);
    }

    #[inline]
    /// Returns the last input buffer flag.
    fn is_last_buffer(&self) -> bool {
        self.my_info.is_last_buffer()
    }

    #[inline]
    /// This function returns true if invalid UTF32 sequence occurred
    /// in this parsing stream.
    fn has_invalid_sequence(&self) -> bool {
        self.my_info.has_invalid_sequence()
    }

    #[inline]
    /// This function signals the occurrence of an invalid UTF32 sequence.
    fn signal_invalid_sequence(&mut self) {
        self.my_info.signal_invalid_sequence();
    }

    #[inline]
    /// This function resets the invalid decodes state.
    fn reset_invalid_sequence(& mut self) {
        self.my_info.reset_invalid_sequence();
    }

    #[inline]
    /// Reset all parser states to the initial value.
    /// Last buffer indication is set to true.
    /// Invalid decodes indication is cleared.
    fn reset_parser(&mut self) {
        self.my_info.reset_parser();
    }
}

/// Iterator for CharRefIterToUtf8Iter
impl<'h> Iterator for CharRefIterToUtf8Iter<'h> {
    type Item = u8;

    /// A parser takes in an iterator of Unicode codepoints, and returns
    /// the output UTF8 byte value.
    ///
    /// An invalid Unicode codepoint in the stream are substituted with
    /// an Unicode replacement character.
    ///
    /// has_invalid_sequence() would return true after observing
    /// invalid decodes, or observing a replacement character.
    fn next(&mut self) -> Option<Self::Item> {
        // Check if we can pull an u8 from our ring buffer.
        match self.my_info.my_buf.pop_front() {
            Option::Some(v1) => {
                return Option::Some(v1);
            }
            Option::None => {}
        }
        // Processing for input being empty case
        match self.my_borrow_mut_iter.next() {
            Option::None => {
                return Option::None;
            }
            Option::Some(ch_ref) => {
                let utf32 = (* ch_ref) as u32;
                // Try to determine the type of UTFf32 encoding.
                match classify_utf32(utf32) {
                    Utf8TypeEnum::Type1(v1) => {
                        Option::Some(v1)
                    }
                    Utf8TypeEnum::Type2((v1,v2)) => {
                        self.my_info.my_buf.push_back(v2);
                        Option::Some(v1)
                    }
                    Utf8TypeEnum::Type3((v1,v2,v3)) => {
                        self.my_info.my_buf.push_back(v2);
                        self.my_info.my_buf.push_back(v3);
                        Option::Some(v1)
                    }
                    Utf8TypeEnum::Type4((v1,v2,v3,v4)) => {
                        self.my_info.my_buf.push_back(v2);
                        self.my_info.my_buf.push_back(v3);
                        self.my_info.my_buf.push_back(v4);
                        Option::Some(v1)
                    }
                    _ => {
                        // Invalid UTF32 codepoint
                        // Emit replacement byte sequence.
                        self.my_info.signal_invalid_sequence();
                        self.my_info.my_buf.push_back(REPLACE_PART2);
                        self.my_info.my_buf.push_back(REPLACE_PART3);
                        Option::Some(REPLACE_PART1)
                    }
                }
            }
        }
    }

    /// sizing hint for iterator, with a lower bound and optional upperbound
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.my_borrow_mut_iter.size_hint()
    }

}

#[cfg(test)]
mod tests {
    extern crate std;

    use crate::prelude::*;

    // Print bytes in hex codes.
    fn _print_bytes(u8_slice: & [u8]) {
        for indx in 0 .. u8_slice.len() {
            let b = u8_slice[indx] as u32;
            print!(" {:#02x}", b);
        }
        println!("");
    }

    // Have a char value go through a round trip of conversions.
    fn round_trip_parsing1(char_val: char) {
        let char_box: [char; 1] = [char_val; 1];
        let mut utf8_box: [u8; 4] = [0; 4];
        let mut utf8_len: usize = 0;

        let mut char_ref = & char_box[..];
        let mut utf32_parser = FromUnicode::new();
        loop {
            match utf32_parser.char_to_utf8(char_ref) {
                Result::Ok((char_pos, b)) => {
                    if char_val == char::REPLACEMENT_CHARACTER {
                        assert_eq!(true, utf32_parser.has_invalid_sequence());
                    }
                    utf8_box[utf8_len] = b;
                    utf8_len += 1;
                    char_ref = char_pos;
                }
                Result::Err(MoreEnum::More(_)) => {
                    break;
                }
            }
        }
        let mut utf8_ref = & utf8_box[0 .. utf8_len];
        let mut char_box2: [char; 1] = [char::MAX; 1];
        let mut char_len: usize = 0;
        let mut utf8_parser = FromUtf8::new();
        loop {
            match utf8_parser.utf8_to_char(utf8_ref) {
                Result::Ok((utf8_pos, ch)) => {
                    if char_val == char::REPLACEMENT_CHARACTER {
                        assert_eq!(true, utf8_parser.has_invalid_sequence());
                    }
                    char_box2[char_len] = ch;
                    char_len += 1;
                    utf8_ref = utf8_pos;
                }
                Result::Err(MoreEnum::More(_)) => {
                    break;
                }
            }
        }
        assert_eq!(1, char_len);
        assert_eq!(char_val, char_box2[0]);
    }

    // Have a char value go through a round trip of conversions.
    fn round_trip_parsing2(code_val: u32) {
        let utf32_box: [u32; 1] = [code_val; 1];
        let mut utf8_box: [u8; 4] = [0; 4];
        let mut utf8_len: usize = 0;

        let mut utf32_ref = & utf32_box[..];
        let mut utf32_parser = FromUnicode::new();
        loop {
            match utf32_parser.utf32_to_utf8(utf32_ref) {
                Result::Ok((utf32_pos, b)) => {
                    if code_val == REPLACE_UTF32 {
                        assert_eq!(true, utf32_parser.has_invalid_sequence());
                    }
                    utf8_box[utf8_len] = b;
                    utf8_len += 1;
                    utf32_ref = utf32_pos;
                }
                Result::Err(MoreEnum::More(_)) => {
                    break;
                }
            }
        }
        let mut utf8_ref = & utf8_box[0 .. utf8_len];
        let mut utf32_box2: [u32; 1] = [0; 1];
        let mut utf32_len: usize = 0;
        let mut utf8_parser = FromUtf8::new();
        loop {
            match utf8_parser.utf8_to_utf32(utf8_ref) {
                Result::Ok((utf8_pos, co)) => {
                    if code_val == REPLACE_UTF32 {
                        assert_eq!(true, utf8_parser.has_invalid_sequence());
                    }
                    utf32_box2[utf32_len] = co;
                    utf32_len += 1;
                    utf8_ref = utf8_pos;
                }
                Result::Err(MoreEnum::More(_)) => {
                    break;
                }
            }
        }
        assert_eq!(1, utf32_len);
        assert_eq!(code_val, utf32_box2[0]);
    }

    #[test]
    // Test using both parsing converters to convert back and forth.
    pub fn test_round_trip_parsing() {
        let mut code:u32 = 0;
        loop {
            let ch = char::from_u32(code).unwrap();
            round_trip_parsing1(ch);
            round_trip_parsing2(code);
            code += 1;
            if code == 0xD800 {
                code = 0xE000; // skip UTF16 surrogate range
            }
            if code == 0x110000 {
                break;
            }
        }
    }


}

pub mod buf;
