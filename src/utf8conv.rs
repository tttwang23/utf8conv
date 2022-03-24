// Copyright 2022 Thomas Wang and utf8conv contributors


// This is the representation of the replacement character in UTF8 encoding.
pub const REPLACE_UTF32:u32 = 0xFFFD;
pub const REPLACE_PART1:u8 = 0xEFu8;
pub const REPLACE_PART2:u8 = 0xBFu8;
pub const REPLACE_PART3:u8 = 0xBDu8;

pub const TYPE2_PREFIX:u32 = 0b1100_0000u32;
pub const TYPE3_PREFIX:u32 = 0b1110_0000u32;
pub const TYPE4_PREFIX:u32 = 0b1111_0000u32;

pub const BYTE2_PREFIX:u32 = 0b1000_0000u32;

// (v & SIX_ONES) << 6 is the same as
// (v << 6) & SIX_ONES_SHIFTED
// This breaks up the pattern of using shift units in the same cycle.
pub const SIX_ONES_SHIFTED:u32 = 0b111111000000u32; // 6 bits shifted 6 digits
pub const SIX_ONES:u32 = 0b111111u32;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
/// Indication for needing more data when parameter value greater than 0,
/// or end of data condition when parameter value is 0.
///
/// (These are not really error conditions.)
pub enum MoreEnum {
    More(u32),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
/// Indication for the type of UTF8 decoding when converting
/// from UTF32 to UTF8
pub enum Utf8TypeEnum {
    Type1(u8),
    // 1 byte type

    Type2((u8,u8)),
    // 2 byte type

    Type3((u8,u8,u8)),
    // 3 byte type

    Type4((u8,u8,u8,u8)),
    // 4 byte type

    Type0((u8,u8,u8)),
    // invalid codepoint; substituted with replacement characters
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(nightly, warn(rustdoc::missing_doc_code_examples))]
/// Utf8EndEnum is the result container for the UTF8 to char
/// finite state machine.
pub enum Utf8EndEnum {
    BadDecode(u32), // bad decode with failure sequence length: 1, 2, or 3
    Finish(u32), // Finished state with a valid codepoint
    TypeUnknown, // not enough characters: type unknown
}


#[inline]
/// Classify an UTF32 value into the type of UTF8 it belongs.
/// Returning Utf8TypeEnum indicates the sequence length.
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
> action 21: out = (arg << 6)+(v3 & 0x3F)
> action 24: out = (arg << 6)+(v4 & 0x3F)
>
>
> If buffer is empty then it could be EOF or need to signal for more data.
>
> We need to ensure the required number of bytes are available when
> the first byte is checked.  Otherwise it is TypeUnknown. (partial data)
> Different tituation when at the last buffer - we go in to process the
> remaining bytes even when we could run out mid-stream.
> This avoids a quote escaping attack, such as quote - F0 - quote - newline

*/

use core::iter::Empty;
use core::iter::empty;
use core::iter::Iterator;

use crate::utf8conv::buf::FifoBytes;


// Action 9 and 10 are different; action 9 can be an end state, while
// action 10 cannot.

#[inline]
/// Finite state machine action 9; expect 80 to bf
fn byte2_action9(mybuf: & mut FifoBytes, arg: u32) -> Utf8EndEnum {
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
fn byte2_action10(mybuf: & mut FifoBytes, arg: u32) -> Utf8EndEnum {
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
fn byte2_action11(mybuf: & mut FifoBytes, arg: u32) -> Utf8EndEnum {
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

/// Finite state machine action 12; expect 80 to bf
fn byte2_action12(mybuf: & mut FifoBytes, arg: u32) -> Utf8EndEnum {
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
fn byte2_action13(mybuf: & mut FifoBytes, arg: u32) -> Utf8EndEnum {
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
fn byte2_action14(mybuf: & mut FifoBytes, arg: u32) -> Utf8EndEnum {
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
fn byte2_action15(mybuf: & mut FifoBytes, arg: u32) -> Utf8EndEnum {
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
fn byte2_action16(mybuf: & mut FifoBytes, arg: u32) -> Utf8EndEnum {
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
fn byte3_action17(mybuf: & mut FifoBytes, arg: u32) -> Utf8EndEnum {
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
fn byte3_action20(mybuf: & mut FifoBytes, arg: u32) -> Utf8EndEnum {
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
fn byte3_action21(mybuf: & mut FifoBytes, arg: u32) -> Utf8EndEnum {
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
fn byte4_action24(mybuf: & mut FifoBytes, arg: u32) -> Utf8EndEnum {
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
/// parameters:
/// mybuf contains the bytes to be decoded
/// last_buffer is true when we are working on the last byte buffer.
///
/// When there are more pending data available than what is in 'mybuf', and
/// with 'last_buffer' being false, then the parser would refuse to work on
/// potential partial decodes, and returns Utf8EndEnum::TypeUnknown to
/// ask for more data.
///
/// When there are no more data than what is available in 'mybuf', and with
/// 'last_buffer' being true, then partial decodes results in
/// Utf8EndEnum:BadDecode(n) where n is length of error from 1 to 3 bytes.
pub fn utf8_decode(mybuf: & mut FifoBytes, last_buffer: bool) -> Utf8EndEnum {
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

/// a empty byte iterator
pub fn empty_byte_iter() -> Empty<u8> {
    empty::<u8>()
}

/// an empty UTF32 iterator
pub fn empty_utf32_iter() -> Empty<u32> {
    empty::<u32>()
}

/// an empty char iterator
pub fn empty_char_iter() -> Empty<char> {
    empty::<char>()
}

/// Most iterators on arrays allocated on the stack returns a reference
/// in order to save memory.  For our converter use-case this is a
/// problem because our conversion result is a temporary value that
/// is best delivered as a value, not as a reference.
/// This could cause two iterators fail to connect from one output to
/// the next input.
/// Proposed types of converters:
/// ref of char -> u32
/// ref of u32 -> u32
/// ref to byte -> byte
/// char -> u32
/// u32 -> byte
/// byte -> char
///
/// char reference to UTF32 iterator struct
pub struct CharRefToUtf32Struct<'b> {
    my_borrow_mut_iter: &'b mut dyn Iterator<Item = &'b char>,
}

// an adapter iterator to convert a char ref iterator to char iterator
impl<'b> Iterator for CharRefToUtf32Struct<'b> {
    type Item=u32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.my_borrow_mut_iter.next() {
            Option::None => { Option::None }
            Option::Some(v) => { Option::Some(* v as u32) }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.my_borrow_mut_iter.size_hint()
    }
}

/// Function char_ref_iter_to_utf32_iter() takes a mutable reference to
/// a char reference iterator, and return a UTF32 iterator in its place.
///
/// parameter
/// input: a mutable reference to a UTF32 reference iterator
#[inline]
pub fn char_ref_iter_to_utf32_iter<'a, I: 'a + Iterator>(input: &'a mut I)
-> CharRefToUtf32Struct<'a>
where I: Iterator<Item = &'a char>, {
    CharRefToUtf32Struct {
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.my_borrow_mut_iter.size_hint()
    }
}

/// Function utf32_ref_iter_to_utf32_iter() takes a mutable reference to
/// a UTF32 reference iterator, and return a UTF32 iterator in its place.
///
/// parameter
/// input: a mutable reference to a UTF32 reference iterator
#[inline]
pub fn utf32_ref_iter_to_utf32_iter<'a, I: 'a + Iterator>(input: &'a mut I)
-> Utf32RefToUtf32Struct<'a>
where I: Iterator<Item = &'a u32>, {
    Utf32RefToUtf32Struct {
        my_borrow_mut_iter: input,
    }
}

/// byte reference to byte iterator struct
pub struct ByteRefToByteStruct<'b> {
    my_borrow_mut_iter: &'b mut dyn Iterator<Item = &'b u8>,
}

/// an adapter iterator to convert a byte ref iterator to byte iterator
impl<'b> Iterator for ByteRefToByteStruct<'b> {
    type Item=u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.my_borrow_mut_iter.next() {
            Option::None => { Option::None }
            Option::Some(v) => { Option::Some(* v) }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.my_borrow_mut_iter.size_hint()
    }
}

/// Function byte_ref_iter_to_byte_iter() takes a mutable reference to
/// a byte reference iterator, and return a byte iterator in its place.
///
/// parameter
/// input: a mutable reference to a byte reference iterator
#[inline]
pub fn byte_ref_iter_to_byte_iter<'a, I: 'a + Iterator>(input: &'a mut I)
-> ByteRefToByteStruct<'a>
where I: Iterator<Item = &'a u8>, {
    ByteRefToByteStruct {
        my_borrow_mut_iter: input,
    }
}

// Struct of Utf8Parser
pub struct Utf8Parser {
    my_buf: FifoBytes,
    my_last_buffer: bool,
    my_invalid_sequence: bool,
}

/// Struct of Utf32Parser
pub struct Utf32Parser {
    my_buf: FifoBytes,
    my_last_buffer: bool,
    my_invalid_sequence: bool,
}

/// Struct of Utf8Iter
pub struct Utf8Iter<'b> {
    my_buf: FifoBytes,
    my_borrow_mut_iter: &'b mut dyn Iterator<Item = u8>,
    my_last_buffer: bool,
    my_invalid_sequence: bool,
}

/// Struct of Utf32Iter
pub struct Utf32Iter<'b> {
    my_buf: FifoBytes,
    my_borrow_mut_iter: &'b mut dyn Iterator<Item = u32>,
    my_last_buffer: bool,
    my_invalid_sequence: bool,
}




/// Map a char parsing result to a UTF32 parsing result.
pub fn char_to_u32_result_mapper(input: Result<(& [u8], char), MoreEnum>)
-> Result<(& [u8], u32), MoreEnum> {
    match input {
        Result::Err(e) => { Result::Err(e) }
        Result::Ok((new_spot, ch)) => { Ok((new_spot, ch as u32)) }
    }
}

/// Common operations for both parsers
pub trait UtfParserCommon {

    fn reset_parser(&mut self);

    fn set_is_last_buffer(&mut self, b: bool);

    fn is_last_buffer(&self) -> bool;

    fn has_invalid_sequence(&self) -> bool;

    fn reset_invalid_sequence(& mut self);
}

/// Implementations of common operations for Utf8Parser
impl<'b> UtfParserCommon for Utf8Parser {

    #[inline]
    /// If parameter b is true, then any input buffer to be presented will
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
    // This function resets the invalid decodes state.
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

/// Implementations of common operations for Utf32Parser
impl<'b> UtfParserCommon for Utf32Parser {

    #[inline]
    /// If parameter b is true, then any input buffer to be presented will
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
    /// This function returns true if invalid UTF8 decodes occurred in this
    /// parsing stream.
    fn has_invalid_sequence(&self) -> bool {
        self.my_invalid_sequence
    }

    #[inline]
    // This function resets the invalid sequence state.
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

/// Implementations of common operations for Utf8Iter
impl<'b> UtfParserCommon for Utf8Iter<'b> {

    #[inline]
    /// If parameter b is true, then any input buffer to be presented will
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
    /// This function returns true if invalid UTF8 decodes occurred in this
    /// parsing stream.
    fn has_invalid_sequence(&self) -> bool {
        self.my_invalid_sequence
    }

    #[inline]
    // This function resets the invalid sequence state.
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

/// Implementations of common operations for Utf32Iter
impl<'b> UtfParserCommon for Utf32Iter<'b> {

    #[inline]
    /// If parameter b is true, then any input buffer to be presented will
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
    /// This function returns true if invalid UTF8 decodes occurred in this
    /// parsing stream.
    fn has_invalid_sequence(&self) -> bool {
        self.my_invalid_sequence
    }

    #[inline]
    // This function resets the invalid sequence state.
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

/// Implementation of Utf8Parser
impl<'b> Utf8Parser {

    /// Make a new Utf8Parser
    pub fn new() -> Utf8Parser {
        Utf8Parser {
            my_buf : FifoBytes::new(),
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
    /// an invalid decode policy wise.
    pub fn parse_utf8_to_char(&mut self, input: &'b [u8])
    -> Result<(&'b [u8], char), MoreEnum> {
        let mut my_cursor: &[u8] = input;
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
            if self.my_last_buffer {
                // at end of data condition
                Result::Err(MoreEnum::More(0))
            }
            else {
                // Returning an indication to request a new buffer.
                Result::Err(MoreEnum::More(4096))
            }
        }
        else {
            match utf8_decode(& mut self.my_buf, self.my_last_buffer) {
                Utf8EndEnum::BadDecode(_) => {
                    self.my_invalid_sequence = true;
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
                    if self.my_last_buffer {
                        self.my_invalid_sequence = true;
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
    /// an invalid decode policy wise.
    pub fn parse_utf8_to_utf32(&mut self, input: &'b [u8])
    -> Result<(&'b [u8], u32), MoreEnum> {
        let char_parse_result = self.parse_utf8_to_char(input);
        char_to_u32_result_mapper(char_parse_result)
    }

}


/// Implementation of Utf32Parser
impl<'b> Utf32Parser {

    /// Make a new Utf32Parser
    pub fn new() -> Utf32Parser {
        Utf32Parser {
            my_buf : FifoBytes::new(),
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
    pub fn parse_char_to_ut8(&mut self, input: &'b [char])
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
            if self.my_last_buffer {
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
                self.my_invalid_sequence = true;
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
    pub fn parse_utf32_to_ut8(&mut self, input: &'b [u32])
    -> Result<(&'b [u32], u8), MoreEnum> {
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
            if self.my_last_buffer {
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
                self.my_invalid_sequence = true;
                self.my_buf.push_back(REPLACE_PART2);
                self.my_buf.push_back(REPLACE_PART3);
                Result::Ok((my_cursor, REPLACE_PART1))
            }
        }
    }

}

/// Implementation for Utf8Iter
impl<'b> Utf8Iter<'b> {

    /// Make a default Utf8Builder
    pub fn new_with_iter(iter: &'b mut dyn Iterator<Item = u8>)
    -> Utf8Iter {
        Utf8Iter {
            my_buf : FifoBytes::new(),
            my_borrow_mut_iter: iter,
            my_last_buffer : true,
            my_invalid_sequence : false,
        }
    }

    // Make the next Utf8Iter based on the current one.
    // The current one is invalidated.
    pub fn next_iter(self, iter: &'b mut dyn Iterator<Item = u8>)
    -> Utf8Iter {
        Utf8Iter {
            my_buf : self.my_buf,  // copy it
            my_borrow_mut_iter: iter,
            my_last_buffer : self.my_last_buffer,
            my_invalid_sequence : self.my_invalid_sequence,
        }
    }

}

impl<'b> Iterator for Utf8Iter<'b> {
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
            if self.my_buf.is_full() {
                break;
            }
            match self.my_borrow_mut_iter.next() {
                Option::None => {
                    break;
                }
                Option::Some(utf8) => {
                    // Save it in our scratch pad.
                    self.my_buf.push_back(utf8);
                }
            }
        }
        if self.my_buf.is_empty() {
            // This is either the end of data, or the current buffer
            // has run to the end without left-over data in the
            // scratch pad.
            Option::None
        }
        else {
            match utf8_decode(& mut self.my_buf, self.my_last_buffer) {
                Utf8EndEnum::BadDecode(_) => {
                    self.my_invalid_sequence = true;
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
                    if self.my_last_buffer {
                        self.my_invalid_sequence = true;
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.my_borrow_mut_iter.size_hint()
    }
}

/// Implementation for Utf32Iter
impl<'b> Utf32Iter<'b> {

    /// Make a default Utf8Builder
    pub fn new_with_iter(iter: &'b mut dyn Iterator<Item = u32>)
    -> Utf32Iter {
        Utf32Iter {
            my_buf : FifoBytes::new(),
            my_borrow_mut_iter: iter,
            my_last_buffer : true,
            my_invalid_sequence : false,
        }
    }

    // Make the next Utf8Iter based on the current one.
    pub fn next_iter(self, iter: &'b mut dyn Iterator<Item = u32>)
    -> Utf32Iter {
        Utf32Iter {
            my_buf : self.my_buf,  // copy it
            my_borrow_mut_iter: iter,
            my_last_buffer : self.my_last_buffer,
            my_invalid_sequence : self.my_invalid_sequence,
        }
    }

}

impl<'b> Iterator for Utf32Iter<'b> {
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
        match self.my_buf.pop_front() {
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
                        self.my_buf.push_back(v2);
                        Option::Some(v1)
                    }
                    Utf8TypeEnum::Type3((v1,v2,v3)) => {
                        self.my_buf.push_back(v2);
                        self.my_buf.push_back(v3);
                        Option::Some(v1)
                    }
                    Utf8TypeEnum::Type4((v1,v2,v3,v4)) => {
                        self.my_buf.push_back(v2);
                        self.my_buf.push_back(v3);
                        self.my_buf.push_back(v4);
                        Option::Some(v1)
                    }
                    _ => {
                        // Invalid UTF32 codepoint
                        // Emit replacement byte sequence.
                        self.my_invalid_sequence = true;
                        self.my_buf.push_back(REPLACE_PART2);
                        self.my_buf.push_back(REPLACE_PART3);
                        Option::Some(REPLACE_PART1)
                    }
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.my_borrow_mut_iter.size_hint()
    }

}



#[cfg(test)]
mod tests {
    extern crate std;
    extern crate stackfmt;

    use core::slice::from_ref;
    use core::slice::Iter;
    use core::str;
    use super::*;
    use super::UtfParserCommon;
    use super::Utf8Parser;
    use super::Utf32Parser;
    use super::Utf8Iter;
    use super::Utf32Iter;

    // use rand::Rng;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use rand::RngCore;

    fn verify_with_string(par: &mut Utf8Parser, b1:& [u8], b2:& [u8], b3:& [u8], b4:& [u8], truth: &str) {
        let mut panic_buf = [0u8; 12000];
        let mut ender_len:usize = 0;
        let mut truth_iter = truth.char_indices();
        par.set_is_last_buffer(false);
        for stage in 0 .. 4 {
            let mut the_slice: &[u8];
            match stage {
                0 => {
                    the_slice = &b1;
                }
                1 => {
                    the_slice = &b2;
                }
                2 => {
                    the_slice = &b3;
                }
                _ => {
                    the_slice = &b4;
                    par.set_is_last_buffer(true);
                }
            }
            loop {
                match par.parse_utf8_to_char(the_slice) {
                    Result::Ok((slice_pos, test_ch)) => {
                        the_slice = slice_pos;
                        match truth_iter.next() {
                            Option::Some((_pos, truth_ch)) => {
                                if test_ch != truth_ch {
                                    let formatted: &str = stackfmt::fmt_truncate(&mut panic_buf, format_args!(
"The truth string is different than the test vectors.\nTruth string:{}\nTest vector at index {} has code value {:#08x} vs truth {:#08x}"
                                    ,truth, ender_len, (test_ch as u32), (truth_ch as u32)));
                                    panic!("\n{}\n", formatted);
                                }
                            }
                            Option::None => {
                                let formatted: &str = stackfmt::fmt_truncate(&mut panic_buf, format_args!(
"The truth string is shorter than the combined test vectors.\nTruth string:{}\nTest vector at index {} has code value {:#08x}"
                                , truth, ender_len, (test_ch as u32)));
                                panic!("\n{}\n",formatted);
                            }
                        }
                        ender_len += 1;
                    }
                    Result::Err(en) => {
                        match en {
                            MoreEnum::More(i) => {
                                if i == 0 {
                                    match truth_iter.next() {
                                        Option::Some((_pos, truth_ch)) => {
                                            let formatted: &str = stackfmt::fmt_truncate(&mut panic_buf, format_args!(
"The truth string is longer than the test vectors (length {}).\nTruth string:{}\nTruth at index {} has code value {:#08x}"
                                            , ender_len, truth, ender_len+1, (truth_ch as u32)));
                                            panic!("\n{}\n", formatted);
                                        }
                                        Option::None => {
                                            // Truth and test vector ran out at the same time.  Test passed.
                                            return;
                                        }
                                    }
                                }
                                else {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }


    #[test]
    fn simple_example() {
        let mybuffer = "abc\n".as_bytes();
        let mut utf8_ref_iter = mybuffer.iter();
        let mut utf8_iter = byte_ref_iter_to_byte_iter(& mut utf8_ref_iter);
        let parser = Utf8Iter::new_with_iter(& mut utf8_iter);
        for char_val in parser {
            print!("{}", char_val);
        }

        // Convert a char iterator to a UTF32 iterator.
        let mchar:char = char::MAX;
        let mut mchar_iter = core::slice::from_ref(&mchar).iter();
        for indx in char_ref_iter_to_utf32_iter( & mut mchar_iter) {
            println!("{}", indx);
        }

        // Test empty char iterator
        let char_iter2 = empty_char_iter();
        for ch in char_iter2 {
            println!("{}", ch);
        }

        // Test empty UTF32 iterator
        let char_iter2 = empty_utf32_iter();
        for num2 in char_iter2 {
            println!("{}", num2);
        }

        // Test empty byte iterator
        let byte_iter2 = empty_byte_iter();
        for by2 in byte_iter2 {
            println!("{}", by2);
        }
    }

    // Print bytes in hex codes.
    fn _print_bytes(u8_slice: & [u8]) {
        for indx in 0 .. u8_slice.len() {
            let b = u8_slice[indx] as u32;
            print!(" {:#02x}", b);
        }
        println!("");
    }

    // Have a char value go through a round trip of conversions.
    fn round_trip(char_val: char) {
        let char_box: [char; 1] = [char_val; 1];

        let mut char_iter = char_box.iter();
        let mut code_iter = char_ref_iter_to_utf32_iter(& mut char_iter);
        let utf32_to_utf8 = Utf32Iter::new_with_iter(& mut code_iter);
        let mut byte_box: [u8; 8] = [0; 8];
        let mut byte_len:usize = 0;
        for b in utf32_to_utf8 {
            byte_box[byte_len] = b;
            byte_len += 1;
        }
        let mut byte_ref_iter = (&byte_box[0 .. byte_len]).iter();
        let mut byte_iter = byte_ref_iter_to_byte_iter(& mut byte_ref_iter);
        let utf8_to_char = Utf8Iter::new_with_iter(& mut byte_iter);
        for ch in utf8_to_char {
            assert_eq!(ch, char_val);
        }
    }

    #[test]
    // Test using both converters to convert back and forth.
    pub fn test_round_trip() {
        let mut code:u32 = 0;
        loop {
            let ch = char::from_u32(code).unwrap();
            round_trip(ch);
            code += 1;
            if code == 0xD800 {
                code = 0xE000; // skip UTF16 surrogate range
            }
            if code == 0x110000 {
                break;
            }
        }
    }

    #[test]
    /// Simple string conversion test
    fn test_utf8parsing_aaa() {
        let mut par:Utf8Parser = Utf8Parser::new();
        println!("case 1: all empty");
        let t1 = "";
        verify_with_string(&mut par, "".as_bytes(), "".as_bytes(), "".as_bytes(), "".as_bytes(), &t1);
        assert!(!par.has_invalid_sequence());

        println!("case 2, different length ASCII");
        let t1 = "abcdef\x7f\t\r\n";
        verify_with_string(&mut par, "a".as_bytes(), "bc".as_bytes(), "def".as_bytes(), "\x7f\t\r\n".as_bytes(), &t1);
        assert!(!par.has_invalid_sequence());

        println!("case 3: multi-language");
        let t1 = "寒い,감기,frío,студен";
        verify_with_string(&mut par, "寒い,".as_bytes(), "감기,".as_bytes(), "frío,".as_bytes(), "студен".as_bytes(), &t1);
        assert!(!par.has_invalid_sequence());

        println!("case 4: emoji and symbols");
        let t1 = "😀🐔🐣🇧🇷🇨🇦元∰⇲";
        verify_with_string(&mut par, "😀".as_bytes(), "🐔🐣".as_bytes(), "🇧🇷🇨🇦".as_bytes(), "元∰⇲".as_bytes(), &t1);
        assert!(!par.has_invalid_sequence());

        println!("case 5: long text");
        // long text
        let t1 = "The red fox jumped over the white fence in a stormy morning with seven chasing servants";
        verify_with_string(&mut par, "The red fox jumped over the white fence in a stormy morning with seven chasing servants".as_bytes(),
        "".as_bytes(), "".as_bytes(), "".as_bytes(), &t1);
        assert!(!par.has_invalid_sequence());

        par.reset_parser();
        println!("case 6: decode across buffer boundaries: ED/9F-bf, C2 / 80");
        let t1 = "\u{D7FF}\u{80}";
        verify_with_string(&mut par, & [0xEDu8], & [0x9Fu8, 0xbfu8], & [0xC2u8], & [0x80u8], &t1);
        assert!(!par.has_invalid_sequence());

        println!("case 7: long decode error followed by 2 byte decode");
        let t1 = "\u{FFFD}\u{FFFD}\u{7FF}";
        verify_with_string(&mut par, &[0xF0u8], "".as_bytes(), & [0x85u8], &[0xDFu8, 0xBFu8], &t1);
        assert!(par.has_invalid_sequence());

        par.reset_parser();
        println!("case 8: decode error in last byte, then an ASCII");
        let t1 = "\u{FFFD}\u{7f}?";
        verify_with_string(&mut par, & [0xF4u8], & [0x8Fu8], & [0x80u8, 0x7fu8], & [0x3fu8], &t1);
        assert!(par.has_invalid_sequence());

        par.reset_parser();
        println!("case 9: overlong encoding of the euro sign");
        let t1 = "\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}";
        verify_with_string(&mut par, & [0xF0u8], & [0x82u8], & [0x82u8], & [0xACu8], &t1);
        assert!(par.has_invalid_sequence());

        par.reset_parser();
        println!("case 10: invalid bytes from F5 to FF");
        let t1 = "\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}";
        verify_with_string(&mut par, & [245u8,246u8,247u8,248u8,249u8,250u8,251u8,252u8,253u8,254u8,255u8], & [], & [], & [], &t1);
        assert!(par.has_invalid_sequence());

        par.reset_parser();
        println!("case 11: accept some non-characters");
        let t1 = "\u{FFFE}\u{FFFF}\u{1FFFF}\u{2FFFE}\u{2FFFF}";
        verify_with_string(&mut par, "\u{FFFE}\u{FFFF}\u{1FFFF}\u{2FFFE}\u{2FFFF}".as_bytes(), & [], & [], & [], &t1);
        assert!(! par.has_invalid_sequence());

        par.reset_parser();
        println!("case 12: unicode 0, 16, 32, 48 ...");
        let t1 = "\u{0}\u{16}\u{32}\u{48}\u{64}\u{80}\u{96}\u{112}\u{128}\u{144}\u{160}";
        verify_with_string(&mut par, "\u{0}\u{16}\u{32}\u{48}\u{64}\u{80}\u{96}\u{112}\u{128}\u{144}\u{160}".as_bytes(), & [], & [], & [], &t1);
        assert!(! par.has_invalid_sequence());

        par.reset_parser();
        println!("case 13: < D0 D0 >");
        let t1 = "<\u{FFFD}\u{FFFD}>";
        verify_with_string(&mut par, "<".as_bytes(), & [0xD0u8], & [0xD0u8], ">".as_bytes(), &t1);
        assert!(par.has_invalid_sequence());

        par.reset_parser();
        println!("case 14: E1 A0 C0");
        let t1 = "\u{FFFD}\u{FFFD}\\";
        verify_with_string(&mut par, & [0xE1u8], & [0xA0u8], & [], & [0xC0, 0x5c], &t1);
        assert!(par.has_invalid_sequence());

        par.reset_parser();
        println!("case 15: over long null characters");
        let t1 = "\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}";
        verify_with_string(&mut par, & [0xE0u8,128u8,128u8], & [0xF0,128u8,128u8,128u8], & [0xC0u8,128u8], & [], &t1);
        assert!(par.has_invalid_sequence());

        par.reset_parser();
        println!("case 16: +U10000");
        let t1 = "\u{10000}";
        verify_with_string(&mut par, & [0b1111_0000u8], & [0b1001_0000u8], & [0b1000_0000u8], & [0b1000_0000u8], &t1);
        assert!(! par.has_invalid_sequence());

        par.reset_parser();
        println!("case 17: double quote, F0, double quote, NL");
        let t1 = "\"\u{FFFD}\"\n";
        verify_with_string(&mut par, & [34u8], & [0xF0u8], & [34u8], & [10u8], &t1);
        assert!(par.has_invalid_sequence());

        par.reset_parser();
        println!("case 18: +UD800");
        let t1 = "\u{FFFD}\u{FFFD}\u{FFFD}\n";
        verify_with_string(&mut par, & [0xEDu8], & [0xA0u8], & [0x80u8], & [10u8], &t1);
        assert!(par.has_invalid_sequence());

        par.reset_parser();
        println!("case 19: +UDFFF");
        let t1 = "\u{FFFD}\u{FFFD}\u{FFFD}\r";
        verify_with_string(&mut par, & [0xEDu8], & [0xbfu8], & [0xbfu8], & [13u8], &t1);
        assert!(par.has_invalid_sequence());

        par.reset_parser();
        println!("case 20: 0x80");
        let t1 = "G\u{FFFD}R\r";
        verify_with_string(&mut par, & [71u8], & [0x80u8], & [82u8], & [13u8], &t1);
        assert!(par.has_invalid_sequence());

        par.reset_parser();
        println!("case 21: 0xC0, 0xC1");
        let t1 = "G\u{FFFD}\u{FFFD}\n";
        verify_with_string(&mut par, & [71u8], & [0xC1u8], & [0xC0u8], & [10u8], &t1);
        assert!(par.has_invalid_sequence());

        par.reset_parser();
        println!("case 22: U+110000");
        let t1 = "\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\n";
        verify_with_string(&mut par, & [0xF5u8], & [0x80u8, 0x80u8], & [0x80u8], & [10u8], &t1);
        assert!(par.has_invalid_sequence());
    }

    // Chop up one u8 slice and distribute among 4 u8 slices.
    fn four_random_slice<'a>(byte_slice: &'a [u8], rng: &mut SmallRng) -> (&'a [u8], &'a [u8], &'a [u8], &'a [u8])
    {
        let remainder_bound:usize = match byte_slice.len() / 4 {
            0usize => { 1usize }
            n => { n }
        };
        let seg_1_len:usize = (rng.next_u32() as usize) % remainder_bound;
        let seg_2_len:usize = (rng.next_u32() as usize) % remainder_bound;
        let seg_3_len:usize = (rng.next_u32() as usize) % remainder_bound;
        let seg_4_len:usize = byte_slice.len() - seg_1_len - seg_2_len - seg_3_len;
        (& byte_slice[0 .. seg_1_len],
        & byte_slice[seg_1_len .. seg_1_len + seg_2_len],
        & byte_slice[seg_1_len + seg_2_len .. seg_1_len + seg_2_len + seg_3_len],
        & byte_slice[seg_1_len + seg_2_len + seg_3_len .. seg_1_len + seg_2_len + seg_3_len + seg_4_len])
    }

    fn spread_noise(byte_slice: &mut [u8], rng: &mut SmallRng) {
        for indx in 0 .. byte_slice.len() {
            if (rng.next_u32() % 10) == 0 {
                rng.fill_bytes(&mut byte_slice[indx .. indx + 1]);
            }
        }
    }

    // Calls char.encode_utf8() to convert a char slice to append to an u8 slice.
    // Returns the entire content of the target u8 slice.
    // Data in char is alredy valid Unicode codepoint.
    fn char_slice_to_u8_slice<'a>(char_slice: & [char], u8_slice: &'a mut [u8]) -> &'a mut [u8] {
        let mut cur_u8_len:usize = 0;
        for char_indx in 0usize .. char_slice.len() {
            let target = char_slice[char_indx].encode_utf8(&mut u8_slice[cur_u8_len .. ]);
            cur_u8_len += target.len();
        }
        &mut u8_slice[0 .. cur_u8_len]
    }

    // Copy from a source u8 slice to a target u8 slice.
    fn copy_u8_slice_to_u8_slice(from_slice: & [u8], to_slice: & mut [u8]) {
        // assert!(to_slice.len() >= from_slice.len());
        let len = from_slice.len();
        for indx in 0 .. len {
            to_slice[indx] = from_slice[indx];
        }
    }

    // Copy a replacement character to a target u8 slice.
    fn copy_replacement_to_u8_slice(to_slice: & mut [u8]) {
        // assert!(to_slice.len() >= 3);
        to_slice[0] = REPLACE_PART1;
        to_slice[1] = REPLACE_PART2;
        to_slice[2] = REPLACE_PART3;
    }

    // Calls str::from_utf8() to convert with replacement from one u8 buffer to another one.
    fn validify_u8_buffer<'a>(u8_slice: & [u8], dest_slice: &'a mut [u8]) -> &'a mut [u8] {
        let mut cur_slice = u8_slice;
        let mut output_len:usize = 0;
        while cur_slice.len() > 0 {
            match str::from_utf8(cur_slice) {
                Ok(str_ref) => {
                    let ref_len = str_ref.len();
                    copy_u8_slice_to_u8_slice(&cur_slice[0 .. ref_len], & mut dest_slice[output_len ..]);
                    cur_slice = &cur_slice[ref_len ..];
                    output_len += ref_len;
                }
                Err(en) => {
                    let valid_up_to = en.valid_up_to();
                    copy_u8_slice_to_u8_slice(&cur_slice[0 .. valid_up_to], &mut dest_slice[output_len ..]);
                    cur_slice = & cur_slice[valid_up_to ..];
                    output_len += valid_up_to;
                    match en.error_len() {
                        Option::Some(err_len) => {
                            copy_replacement_to_u8_slice(& mut dest_slice[output_len ..]);
                            output_len += 3;
                            cur_slice = & cur_slice[err_len .. ];
                        }
                        Option::None => {
                            copy_replacement_to_u8_slice(& mut dest_slice[output_len ..]);
                            output_len += 3;
                            return & mut dest_slice[0 .. output_len]; // EOF
                        }
                    }
                }
            }
        }
        & mut dest_slice[0 .. output_len]
    }

    // Populate a char slice with random codepoints.
    fn make_random_string(char_slice: &mut [char], rng: &mut SmallRng) {
        for indx in 0usize .. char_slice.len() {
            let val:u32 = rng.next_u32() % 0x111000u32;
            match char::from_u32(val) {
                Option::Some(ch) => {
                    char_slice[indx] = ch;
                }
                Option::None => {
                    assert!(!((val > 0xffffu32) && (val <= 0x10ffffu32)));
                    char_slice[indx] = char::REPLACEMENT_CHARACTER;
                }
            }
        }
    }

    #[test]
    fn test_utf8_monkey() {
        let mut par:Utf8Parser = Utf8Parser::new();
        let mut rng = SmallRng::seed_from_u64(0x17841d3a103c10b4u64);
        let mut char_buf = [char::REPLACEMENT_CHARACTER; 160];
        let mut byte_buf = [0u8; 160 * 4];
        let mut byte_buf2 = [0u8; 160 * 12];
        for _indx in 0 .. 40000 {
            make_random_string(& mut char_buf, &mut rng);
            let orig_slice: &mut [u8] = char_slice_to_u8_slice(&char_buf, & mut byte_buf);
            spread_noise(orig_slice, & mut rng);
            let (frag1, frag2, frag3, frag4) = four_random_slice(orig_slice, &mut rng);
            // This is calling str:from_utf8() which is similar to
            // String::from_utf8_lossy().
            let mod_buf2 = validify_u8_buffer(orig_slice, & mut byte_buf2);
            match str::from_utf8(mod_buf2) {
                Ok(mystr) => {
                    // Usually mystr would be longer because the fragments has
                    // errors that will lengthen to replacement characters.
                    verify_with_string(&mut par, frag1, frag2, frag3, frag4, mystr);
                }
                Err(_) => {
                    panic!("Unexpected from_utf8() failure");
                }
            }
        }
    }

}

pub mod buf;
