// Copyright 2022 Thomas Wang and utf8conv contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use utf8conv::*;


// Have a char value go through a round trip of conversions.
fn round_trip_iter(char_val: char) {
    let char_box: [char; 1] = [char_val; 1];

    let mut char_ref_iter = char_box.iter();
    let mut from_unicode = FromUnicode::new();
    let mut char_ref_to_utf8 =
        from_unicode.char_ref_to_utf8_with_iter(& mut char_ref_iter);
    let mut byte_box: [u8; 8] = [0; 8];
    let mut byte_len:usize = 0;

    while let Some(b) = char_ref_to_utf8.next() {
        byte_box[byte_len] = b;
        byte_len += 1;
    }

    let mut utf8_ref_iter = (&byte_box[0 .. byte_len]).iter();
    let mut from_utf8 = FromUtf8::new();
    let mut utf8_to_char =
        from_utf8.utf8_ref_to_char_with_iter(& mut utf8_ref_iter);
    while let Some(ch) = utf8_to_char.next() {
        assert_eq!(ch, char_val);
        if ch == char::REPLACEMENT_CHARACTER {
            assert_eq!(true, utf8_to_char.has_invalid_sequence());
            utf8_to_char.reset_invalid_sequence();
        }
    }
    if char_val == char::REPLACEMENT_CHARACTER {
        assert_eq!(true, char_ref_to_utf8.has_invalid_sequence());
        char_ref_to_utf8.reset_invalid_sequence();
    }
}

#[test]
// Test using both iterator converters to convert back and forth.
pub fn test_round_trip_iter() {
    let mut code:u32 = 0;
    loop {
        let ch = char::from_u32(code).unwrap();
        round_trip_iter(ch);
        code += 1;
        if code == 0xD800 {
            code = 0xE000; // skip UTF16 surrogate range
        }
        if code == 0x110000 {
            break;
        }
    }
}
