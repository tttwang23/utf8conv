// Copyright 2022 Thomas Wang and utf8conv contributors


// This is the representation of the replacement character in UTF8 encoding.
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
pub enum IncompleteEnum {
    Incomplete(u32),
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
        let v1:u8 = ((code >> 12) + TYPE3_PREFIX) as u8;
        let v2:u8 = (((code & SIX_ONES_SHIFTED) >> 6) + BYTE2_PREFIX) as u8;
        let v3:u8 = ((code & SIX_ONES) + BYTE2_PREFIX) as u8;
        Utf8TypeEnum::Type3((v1,v2,v3))
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
>                   action 5     action (10)  action (17)

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

use crate::utf8conv::buf::FifoU8;


// Action 9 and 10 are different; action 9 can be an end state, while
// action 10 cannot.

#[inline]
/// Finite state machine action 9; expect 80 to bf
fn byte2_action9(mybuf: & mut FifoU8, arg: u32) -> Utf8EndEnum {
    match mybuf.peek() {
        Option::Some(v) => {
            let v2 = v as u32;
            // println!("in action 9 with v2={:#02x}", v2);
            if (v2 >= 0x80) && (v2 <= 0xbf) {
                mybuf.pull(); // advance
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
fn byte2_action10(mybuf: & mut FifoU8, arg: u32) -> Utf8EndEnum {
    match mybuf.peek() {
        Option::Some(v) => {
            let v2 = v as u32;
            // println!("in action 10 with v2={:#02x}", v2);
            if (v2 >= 0x80) && (v2 <= 0xbf) {
                mybuf.pull(); // advance
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
fn byte2_action12(mybuf: & mut FifoU8, arg: u32) -> Utf8EndEnum {
    match mybuf.peek() {
        Option::Some(v) => {
            let v2 = v as u32;
            // println!("in action 12 with v2={:#02x}", v2);
            if (v2 >= 0x80) && (v2 <= 0xbf) {
                mybuf.pull(); // advance
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
fn byte2_action13(mybuf: & mut FifoU8, arg: u32) -> Utf8EndEnum {
    match mybuf.peek() {
        Option::Some(v) => {
            let v2 = v as u32;
            // println!("in action 13 with v2={:#02x}", v2);
            if (v2 >= 0x80) && (v2 <= 0x8F) {
                mybuf.pull(); // advance
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
fn byte2_action14(mybuf: & mut FifoU8, arg: u32) -> Utf8EndEnum {
    match mybuf.peek() {
        Option::Some(v) => {
            let v2 = v as u32;
            // println!("in action 14 with v2={:#02x}", v2);
            if (v2 >= 0xA0) && (v2 <= 0xbf) {
                mybuf.pull(); // advance
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
fn byte2_action15(mybuf: & mut FifoU8, arg: u32) -> Utf8EndEnum {
    match mybuf.peek() {
        Option::Some(v) => {
            let v2 = v as u32;
            // println!("in action 15 with v2={:#02x}", v2);
            if (v2 >= 0x80) && (v2 <= 0x9F) {
                mybuf.pull(); // advance
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
fn byte2_action16(mybuf: & mut FifoU8, arg: u32) -> Utf8EndEnum {
    match mybuf.peek() {
        Option::Some(v) => {
            let v2 = v as u32;
            // println!("in action 16 with v2={:#02x}", v2);
            if (v2 >= 0x90) && (v2 <= 0xbf) {
                mybuf.pull(); // advance
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
fn byte3_action17(mybuf: & mut FifoU8, arg: u32) -> Utf8EndEnum {
    match mybuf.peek() {
        Option::Some(v) => {
            let v3 = v as u32;
            // println!("in action 17 with v3={:#02x}", v3);
            if (v3 >= 0x80) && (v3 <= 0xbf) {
                mybuf.pull(); // advance
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
/// Finite state machine action 21; expect 80 to bf
fn byte3_action21(mybuf: & mut FifoU8, arg: u32) -> Utf8EndEnum {
    match mybuf.peek() {
        Option::Some(v) => {
            let v3 = v as u32;
            // println!("in action 21 with v3={:#02x}", v3);
            if (v3 >= 0x80) && (v3 <= 0xbf) {
                mybuf.pull(); // advance
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
fn byte4_action24(mybuf: & mut FifoU8, arg: u32) -> Utf8EndEnum {
    match mybuf.peek() {
        Option::Some(v) => {
            let v4 = v as u32;
            // println!("in action 24 with v4={:#02x}", v4);
            if (v4 >= 0x80) && (v4 <= 0xbf) {
                mybuf.pull(); // advance
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
pub fn utf8_decode(mybuf: & mut FifoU8, last_buffer: bool) -> Utf8EndEnum {
    match mybuf.peek() {
        Option::Some(v) => {
            let v1 = v as u32;
            // println!("in start state with v1={:#02x} and num_elem()={}", v1, mybuf.num_elem());
            if v1 < 0xE0 {
                if v1 < 0xC2 {
                    mybuf.pull();
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
                    if (mybuf.num_elem() < 2) && ! last_buffer {
                        // We wait for more bytes if not the last buffer.
                        // Our design cannot back-out procesed bytes.
                        // println!("TypeUnknown");
                        Utf8EndEnum::TypeUnknown
                    }
                    else {
                        // Action 1
                        // println!("in action 1 with v1={:#02x}", v1);
                        mybuf.pull();
                        byte2_action9(mybuf, v1 & 0x1F)
                    }
                }
            }
            else {
                if v1 < 0xF0 {
                    // 3 byte format
                    // Byte 1 is between 0xE0 and 0xEF
                    if (mybuf.num_elem() < 3) && ! last_buffer {
                        // We wait for more bytes if not the last buffer.
                        // Our design cannot back-out procesed bytes.
                        // println!("TypeUnknown");
                        Utf8EndEnum::TypeUnknown
                    }
                    else if v1 < 0xED {
                        mybuf.pull();
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
                        mybuf.pull();
                        if v1 == 0xED {
                            // Action 4
                            // println!("in action 4 with v1={:#02x}", v1);
                            byte2_action15(mybuf, v1 & 0xF)
                        }
                        else {
                            // Action 5
                            // v1 is 0xEE or 0xEF.
                            // println!("in action 5 with v1={:#02x}", v1);
                            byte2_action10(mybuf, v1 & 0xF)
                        }
                    }
                }
                else {
                    // 4 byte cases if byte 1 is between 0xF0 and 0xF4
                    if v1 > 0xF4 {
                        // codepoint too large
                        // println!("greater than F4 bad decode");
                        mybuf.pull();
                        Utf8EndEnum::BadDecode(1)
                    }
                    else if (mybuf.num_elem() < 4) && ! last_buffer {
                        // We wait for more bytes if not the last buffer.
                        // Our design cannot back-out procesed bytes.
                        // println!("TypeUnknown");
                        Utf8EndEnum::TypeUnknown
                    }
                    else {
                        mybuf.pull();
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

/// Map a char parsing result to a u32 parsing result.
#[inline]
pub fn char_to_u32_result_mapper(input: Result<(& [u8], char), IncompleteEnum>)
-> Result<(& [u8], u32), IncompleteEnum> {
    match input {
        Result::Err(e) => { Result::Err(e) }
        Result::Ok((new_spot, ch)) => { Ok((new_spot, ch as u32)) }
    }
}

#[derive(Debug, Clone, Copy)]
// Struct of Utf8ArrayParser
pub struct Utf8ArrayParser {
    mybuf: FifoU8,
    last_buffer: bool,
    invalid_decodes: bool,
}

#[derive(Debug, Clone, Copy)]
/// Struct of Utf32ArrayParser
pub struct Utf32ArrayParser {
    mybuf: FifoU8,
    last_buffer: bool,
    invalid_decodes: bool,
}

/// Common operations for both parsers
pub trait UtfParserCommon {

    fn reset_parser(&mut self);

    fn set_is_last_buffer(&mut self, b: bool);

    fn is_last_buffer(&self) -> bool;

    fn has_invalid_decodes(&self) -> bool;

    fn reset_invalid_decodes(& mut self);
}

/// Implementations of common operations for Utf8ArrayParser
impl UtfParserCommon for Utf8ArrayParser {

    #[inline]
    /// If parameter b is true, then any input buffer to be presented will
    /// be the last buffer.
    fn set_is_last_buffer(&mut self, b: bool) {
        self.last_buffer = b;
    }

    #[inline]
    /// Returns the last input buffer flag.
    fn is_last_buffer(&self) -> bool {
        self.last_buffer
    }

    #[inline]
    /// This function returns true if invalid UTF8 decodes occurred in this
    /// parsing stream.
    fn has_invalid_decodes(&self) -> bool {
        self.invalid_decodes
    }

    #[inline]
    // This function resets the invalid decodes state.
    fn reset_invalid_decodes(& mut self) {
        self.invalid_decodes = false;
    }

    #[inline]
    /// Reset all parser states to the initial value.
    /// Last buffer indication is set to true.
    /// Invalid decodes indication is cleared.
    fn reset_parser(&mut self) {
        // Drain our buffer.
        self.mybuf.clear_all();
        self.set_is_last_buffer(true);
        self.reset_invalid_decodes();
    }

}

/// Implementations of common operations for Utf32ArrayParser
impl UtfParserCommon for Utf32ArrayParser {

    #[inline]
    /// If parameter b is true, then any input buffer to be presented will
    /// be the last buffer.
    fn set_is_last_buffer(&mut self, b: bool) {
        self.last_buffer = b;
    }

    #[inline]
    /// Returns the last input buffer flag.
    fn is_last_buffer(&self) -> bool {
        self.last_buffer
    }

    #[inline]
    /// This function returns true if invalid UTF8 decodes occurred in this
    /// parsing stream.
    fn has_invalid_decodes(&self) -> bool {
        self.invalid_decodes
    }

    #[inline]
    // This function resets the invalid decodes state.
    fn reset_invalid_decodes(&mut self) {
        self.invalid_decodes = false;
    }

    #[inline]
    /// Reset all parser states to the initial value.
    /// Last buffer indication is set to true.
    /// Invalid decodes indication is cleared.
    fn reset_parser(&mut self) {
        // Drain our buffer.
        self.mybuf.clear_all();
        self.set_is_last_buffer(true);
        self.reset_invalid_decodes();
    }

}

/// Implementation of Utf8ArrayParser
impl<'a> Utf8ArrayParser {

    #[inline]
    /// Create a new Utf8ArrayParser instance.
    /// By default, parser is set to a single buffer.
    pub fn new() -> Self {
        Utf8ArrayParser {
            mybuf: FifoU8::new(),
            last_buffer: true,
            invalid_decodes: false,
        }
    }

    /// A parser takes in input type, and returns a 'Result' containing
    /// either the remaining input and the output char value, or an IncompleteEnum
    /// that requests additional data, or an end of data stream condition.
    ///
    /// Invalid UTF8 decodes are indicated by Unicode replacement characters.
    /// has_invalid_decodes() would return true after this event.
    pub fn parse_utf8_to_char(&mut self, input: &'a [u8]) -> Result<(&'a [u8], char), IncompleteEnum> {
        let mut my_cursor: &[u8] = input;
        // Fill buffer phase.
        loop {
            if self.mybuf.is_full() || (my_cursor.len() == 0) {
                break;
            }
            // Push a u8, and advance input position.
            self.mybuf.push(my_cursor[0]);
            my_cursor = &my_cursor[1..];
        }
        if self.mybuf.is_empty() {
            // Processing for buffer being empty case
            // Determine if we are at EOF.
            if self.last_buffer {
                // at EOF condition
                Result::Err(IncompleteEnum::Incomplete(0))
            }
            else {
                // Return an indication to request a new buffer.
                Result::Err(IncompleteEnum::Incomplete(1024))
            }
        }
        else {
            match utf8_decode(& mut self.mybuf, self.last_buffer) {
                Utf8EndEnum::BadDecode(_) => {
                    self.invalid_decodes = true;
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
                    if self.last_buffer {
                        self.invalid_decodes = true;
                        // Buffer should be empty at this point.
                        Result::Ok((my_cursor, char::REPLACEMENT_CHARACTER))
                    }
                    else {
                        // Return an indication to request a new buffer.
                        Result::Err(IncompleteEnum::Incomplete(1024))
                    }
                }
            }
        }
    }

    /// A parser takes in input type, and returns a 'Result' containing
    /// either the remaining input and the output u32 value, or an IncompleteEnum
    /// that requests additional data, or an end of data stream condition.
    ///
    /// Invalid UTF8 decodes are indicated by Unicode replacement characters.
    /// has_invalid_decodes() would return true after this event.
    pub fn parse_utf8_to_utf32(&mut self, input: &'a [u8]) -> Result<(&'a [u8], u32), IncompleteEnum> {
        let char_parse_result = self.parse_utf8_to_char(input);
        char_to_u32_result_mapper(char_parse_result)
    }
}

/// Implementation of Utf32ArrayParser
impl<'a> Utf32ArrayParser {

    /// Create a new Utf8ArrayParser instance.
    /// By default, parser is set to a single buffer.
    #[inline]
    pub fn new() -> Self {
        Utf32ArrayParser {
            mybuf: FifoU8::new(),
            last_buffer: true,
            invalid_decodes: false,
        }
    }

    /// A parser takes in input type, and returns a 'Result' containing
    /// either the remaining input and the output u8 value, or an IncompleteEnum
    /// that requests additional data, or an end of data stream condition.
    ///
    /// Invalid UTF32 decodes are indicated by Unicode replacement characters.
    /// has_invalid_decodes() would return true after this event.
    pub fn parse_utf32_to_utf8(&mut self, input: &'a [u32])
    -> Result<(&'a [u32], u8), IncompleteEnum> {
        // Check if we can pull an u8 from our ring buffer.
        match self.mybuf.pull() {
            Some(v1) => {
                return Result::Ok((input, v1));
            }
            None => {}
        }
        let mut my_cursor: &[u32] = input;
        // Processing for input being empty case
        if my_cursor.len() == 0 {
            // Determine if we are at EOF.
            if self.last_buffer {
                // at EOF condition
                return Result::Err(IncompleteEnum::Incomplete(0));
            }
            else {
                // Return an indication to request a new buffer.
                return Result::Err(IncompleteEnum::Incomplete(1024));
            }
        }
        // Grab one UTF32 from input.
        let cur_u32 = my_cursor[0];
        my_cursor = &my_cursor[1..];
        // Try to determine the type of UTFf32 encoding.
        match classify_utf32(cur_u32) {
            Utf8TypeEnum::Type1(v1) => {
                Result::Ok((my_cursor, v1))
            }
            Utf8TypeEnum::Type2((v1,v2)) => {
                self.mybuf.push(v2);
                Result::Ok((my_cursor, v1))
            }
            Utf8TypeEnum::Type3((v1,v2,v3)) => {
                self.mybuf.push(v2);
                self.mybuf.push(v3);
                Result::Ok((my_cursor, v1))
            }
            Utf8TypeEnum::Type4((v1,v2,v3,v4)) => {
                self.mybuf.push(v2);
                self.mybuf.push(v3);
                self.mybuf.push(v4);
                Result::Ok((my_cursor, v1))
            }
            _ => {
                // Invalid UTF32 codepoint
                // Emit replacement byte sequence.
                self.invalid_decodes = true;
                self.mybuf.push(REPLACE_PART2);
                self.mybuf.push(REPLACE_PART3);
                Result::Ok((my_cursor, REPLACE_PART1))
            }
        }
    }

    /// A parser takes in input type, and returns a 'Result' containing
    /// either the remaining input and the output u8 value, or an IncompleteEnum
    /// that requests additional data, or an end of data stream condition.
    ///
    /// Invalid UTF32 decodes are indicated by Unicode replacement characters.
    /// has_invalid_decodes() would return true after this event.
    pub fn parse_char_to_utf8(&mut self, input: &'a [char])
    -> Result<(&'a [char], u8), IncompleteEnum> {
        // Check if we can pull an u8 from our ring buffer.
        match self.mybuf.pull() {
            Some(v1) => {
                return Result::Ok((input, v1));
            }
            None => {}
        }
        let mut my_cursor: &[char] = input;
        // Processing for input being empty case
        if my_cursor.len() == 0 {
            // Determine if we are at EOF.
            if self.last_buffer {
                // at EOF condition
                return Result::Err(IncompleteEnum::Incomplete(0));
            }
            else {
                // Return an indication to request a new buffer.
                return Result::Err(IncompleteEnum::Incomplete(1024));
            }
        }
        // Grab one UTF32 from input.
        let cur_u32 = my_cursor[0] as u32;
        my_cursor = &my_cursor[1..];
        // Try to determine the type of utf8 encoding.
        match classify_utf32(cur_u32) {
            Utf8TypeEnum::Type1(v1) => {
                Result::Ok((my_cursor, v1))
            }
            Utf8TypeEnum::Type2((v1,v2)) => {
                self.mybuf.push(v2);
                Result::Ok((my_cursor, v1))
            }
            Utf8TypeEnum::Type3((v1,v2,v3)) => {
                self.mybuf.push(v2);
                self.mybuf.push(v3);
                Result::Ok((my_cursor, v1))
            }
            Utf8TypeEnum::Type4((v1,v2,v3,v4)) => {
                self.mybuf.push(v2);
                self.mybuf.push(v3);
                self.mybuf.push(v4);
                Result::Ok((my_cursor, v1))
            }
            _ => {
                // Invalid UTF32 codepoint
                // Emit replacement byte sequence.
                self.invalid_decodes = true;
                self.mybuf.push(REPLACE_PART2);
                self.mybuf.push(REPLACE_PART3);
                Result::Ok((my_cursor, REPLACE_PART1))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    extern crate stackfmt;

    use core::str;
    use super::*;
    use super::UtfParserCommon;
    use super::Utf8ArrayParser;
    use super::IncompleteEnum;
    // use rand::Rng;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use rand::RngCore;

    fn verify_with_string(par: &mut Utf8ArrayParser, b1:& [u8], b2:& [u8], b3:& [u8], b4:& [u8], truth: &str) {
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
                    Result::Ok((new_slice , test_ch)) => {
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
                        the_slice = new_slice;
                    }
                    Result::Err(en) => {
                        match en {
                            IncompleteEnum::Incomplete(i) => {
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
        let mut parser = Utf8ArrayParser::new();
        let mut byte_slice = mybuffer;
        loop {
            match parser.parse_utf8_to_char(byte_slice)
            {
                Result::Ok((next_slice, ch)) => {
                    byte_slice = next_slice;
                    print!("{}", ch);
                }
                Result::Err(_) => {
                    // for a single buffer Err is always end of data
                    break;
                }
            }
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
    fn round_trip(code: char, byte_par: & mut Utf8ArrayParser) {
        let char_val: u32 = code as u32;
        let mut char_par:Utf32ArrayParser = Utf32ArrayParser::new();
        let mut char_box: [char; 1] = [char::MAX; 1];
        let mut u32_box: [u32; 1] = [0; 1];
        let mut byte_box: [u8; 8] = [0; 8];
        let mut byte_len:usize = 0;

        char_box[0] = code;
        u32_box[0] = char_val;

        if char_val & 1 == 0 {
            // First convert from char to UTF8.
            let mut char_ptr = & char_box[..];
            loop {
                match char_par.parse_char_to_utf8(char_ptr) {
                    Ok((p, b)) => {
                        // Populate byte box
                        char_ptr = p;
                        byte_box[byte_len] = b;
                        byte_len += 1;
                    }
                    Err(_) => {
                        break; // always EOF when single buffer case
                    }
                }
            }
            // Then convert from UTF8 to char.
            let mut byte_ptr = & byte_box[0 .. byte_len];
            loop {
                match byte_par.parse_utf8_to_char(byte_ptr) {
                    Ok((p, ch)) => {
                        byte_ptr = p;
                        // Read back the char value.
                        assert_eq!(ch, code);
                    }
                    Err(_) => {
                        break; // always EOF when single buffer case
                    }
                }
            }
        }
        else {
            // First convert from UTF32 to UTF8.
            let mut utf32_ptr = & u32_box[..];
            loop {
                match char_par.parse_utf32_to_utf8(utf32_ptr) {
                    Ok((p, b)) => {
                        // Populate byte box
                        utf32_ptr = p;
                        byte_box[byte_len] = b;
                        byte_len += 1;
                    }
                    Err(_) => {
                        break; // always EOF when single buffer case
                    }
                }
            }
            // Then convert from UTF8 to UTF32.
            let mut byte_ptr = & byte_box[0 .. byte_len];
            loop {
                match byte_par.parse_utf8_to_utf32(byte_ptr) {
                    Ok((p, v)) => {
                        byte_ptr = p;
                        // Read back the char value.
                        assert_eq!(v, char_val);
                    }
                    Err(_) => {
                        break; // always EOF when single buffer case
                    }
                }
            }
        }
    }

    #[test]
    // Test using both converters to convert back and forth.
    pub fn test_round_trip() {
        let mut byte_par = Utf8ArrayParser::new();
        let mut code:u32 = 0;
        loop {
            let ch = char::from_u32(code).unwrap();
            round_trip(ch, & mut byte_par);
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
        let mut par:Utf8ArrayParser = Utf8ArrayParser::new();
        println!("case 1: all empty");
        let t1 = "";
        verify_with_string(&mut par, "".as_bytes(), "".as_bytes(), "".as_bytes(), "".as_bytes(), &t1);
        assert!(!par.has_invalid_decodes());

        println!("case 2, different length ASCII");
        let t1 = "abcdef\x7f\t\r\n";
        verify_with_string(&mut par, "a".as_bytes(), "bc".as_bytes(), "def".as_bytes(), "\x7f\t\r\n".as_bytes(), &t1);
        assert!(!par.has_invalid_decodes());

        println!("case 3: multi-language");
        let t1 = "寒い,감기,frío,студен";
        verify_with_string(&mut par, "寒い,".as_bytes(), "감기,".as_bytes(), "frío,".as_bytes(), "студен".as_bytes(), &t1);
        assert!(!par.has_invalid_decodes());

        println!("case 4: emoji and symbols");
        let t1 = "😀🐔🐣🇧🇷🇨🇦元∰⇲";
        verify_with_string(&mut par, "😀".as_bytes(), "🐔🐣".as_bytes(), "🇧🇷🇨🇦".as_bytes(), "元∰⇲".as_bytes(), &t1);
        assert!(!par.has_invalid_decodes());

        println!("case 5: long text");
        // long text
        let t1 = "The red fox jumped over the white fence in a stormy morning with seven chasing servants";
        verify_with_string(&mut par, "The red fox jumped over the white fence in a stormy morning with seven chasing servants".as_bytes(),
        "".as_bytes(), "".as_bytes(), "".as_bytes(), &t1);
        assert!(!par.has_invalid_decodes());

        par.reset_parser();
        println!("case 6: decode across buffer boundaries: ED/9F-bf, C2 / 80");
        let t1 = "\u{D7FF}\u{80}";
        verify_with_string(&mut par, & [0xEDu8], & [0x9Fu8, 0xbfu8], & [0xC2u8], & [0x80u8], &t1);
        assert!(!par.has_invalid_decodes());

        println!("case 7: long decode error followed by 2 byte decode");
        let t1 = "\u{FFFD}\u{FFFD}\u{7FF}";
        verify_with_string(&mut par, &[0xF0u8], "".as_bytes(), & [0x85u8], &[0xDFu8, 0xBFu8], &t1);
        assert!(par.has_invalid_decodes());

        par.reset_parser();
        println!("case 8: decode error in last byte, then an ASCII");
        let t1 = "\u{FFFD}\u{7f}?";
        verify_with_string(&mut par, & [0xF4u8], & [0x8Fu8], & [0x80u8, 0x7fu8], & [0x3fu8], &t1);
        assert!(par.has_invalid_decodes());

        par.reset_parser();
        println!("case 9: overlong encoding of the euro sign");
        let t1 = "\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}";
        verify_with_string(&mut par, & [0xF0u8], & [0x82u8], & [0x82u8], & [0xACu8], &t1);
        assert!(par.has_invalid_decodes());

        par.reset_parser();
        println!("case 10: invalid bytes from F5 to FF");
        let t1 = "\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}";
        verify_with_string(&mut par, & [245u8,246u8,247u8,248u8,249u8,250u8,251u8,252u8,253u8,254u8,255u8], & [], & [], & [], &t1);
        assert!(par.has_invalid_decodes());

        par.reset_parser();
        println!("case 11: accept some non-characters");
        let t1 = "\u{FFFE}\u{FFFF}\u{1FFFF}\u{2FFFE}\u{2FFFF}";
        verify_with_string(&mut par, "\u{FFFE}\u{FFFF}\u{1FFFF}\u{2FFFE}\u{2FFFF}".as_bytes(), & [], & [], & [], &t1);
        assert!(! par.has_invalid_decodes());

        par.reset_parser();
        println!("case 12: unicode 0, 16, 32, 48 ...");
        let t1 = "\u{0}\u{16}\u{32}\u{48}\u{64}\u{80}\u{96}\u{112}\u{128}\u{144}\u{160}";
        verify_with_string(&mut par, "\u{0}\u{16}\u{32}\u{48}\u{64}\u{80}\u{96}\u{112}\u{128}\u{144}\u{160}".as_bytes(), & [], & [], & [], &t1);
        assert!(! par.has_invalid_decodes());

        par.reset_parser();
        println!("case 13: < D0 D0 >");
        let t1 = "<\u{FFFD}\u{FFFD}>";
        verify_with_string(&mut par, "<".as_bytes(), & [0xD0u8], & [0xD0u8], ">".as_bytes(), &t1);
        assert!(par.has_invalid_decodes());

        par.reset_parser();
        println!("case 14: E1 A0 C0");
        let t1 = "\u{FFFD}\u{FFFD}\\";
        verify_with_string(&mut par, & [0xE1u8], & [0xA0u8], & [], & [0xC0, 0x5c], &t1);
        assert!(par.has_invalid_decodes());

        par.reset_parser();
        println!("case 15: over long null characters");
        let t1 = "\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}";
        verify_with_string(&mut par, & [0xE0u8,128u8,128u8], & [0xF0,128u8,128u8,128u8], & [0xC0u8,128u8], & [], &t1);
        assert!(par.has_invalid_decodes());

        par.reset_parser();
        println!("case 16: +U10000");
        let t1 = "\u{10000}";
        verify_with_string(&mut par, & [0b1111_0000u8], & [0b1001_0000u8], & [0b1000_0000u8], & [0b1000_0000u8], &t1);
        assert!(! par.has_invalid_decodes());

        par.reset_parser();
        println!("case 17: double quote, F0, double quote, NL");
        let t1 = "\"\u{FFFD}\"\n";
        verify_with_string(&mut par, & [34u8], & [0xF0u8], & [34u8], & [10u8], &t1);
        assert!(par.has_invalid_decodes());

        par.reset_parser();
        println!("case 18: +UD800");
        let t1 = "\u{FFFD}\u{FFFD}\u{FFFD}\n";
        verify_with_string(&mut par, & [0xEDu8], & [0xA0u8], & [0x80u8], & [10u8], &t1);
        assert!(par.has_invalid_decodes());

        par.reset_parser();
        println!("case 19: +UDFFF");
        let t1 = "\u{FFFD}\u{FFFD}\u{FFFD}\r";
        verify_with_string(&mut par, & [0xEDu8], & [0xbfu8], & [0xbfu8], & [13u8], &t1);
        assert!(par.has_invalid_decodes());

        par.reset_parser();
        println!("case 20: 0x80");
        let t1 = "G\u{FFFD}R\r";
        verify_with_string(&mut par, & [71u8], & [0x80u8], & [82u8], & [13u8], &t1);
        assert!(par.has_invalid_decodes());

        par.reset_parser();
        println!("case 21: 0xC0, 0xC1");
        let t1 = "G\u{FFFD}\u{FFFD}\n";
        verify_with_string(&mut par, & [71u8], & [0xC1u8], & [0xC0u8], & [10u8], &t1);
        assert!(par.has_invalid_decodes());

        par.reset_parser();
        println!("case 22: U+110000");
        let t1 = "\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\n";
        verify_with_string(&mut par, & [0xF5u8], & [0x80u8, 0x80u8], & [0x80u8], & [10u8], &t1);
        assert!(par.has_invalid_decodes());
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
        let mut par:Utf8ArrayParser = Utf8ArrayParser::new();
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
