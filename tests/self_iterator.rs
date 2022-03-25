// Copyright 2022 Thomas Wang and utf8conv contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use utf8conv::prelude::*;

// Test self reflecting iterators
#[test]
fn self_iterator_test() {

    let char_array: [char; 2] = [char::MAX, char::REPLACEMENT_CHARACTER];
    let mut char_ref_iter = char_array.iter();
    let mut char_ref_to_char_iter =
    char_ref_iter_to_char_iter(& mut char_ref_iter);

    let u32_array: [u32; 5] = [3, 90, 250, 870, 2300];
    let mut u32_ref_iter = u32_array.iter();
    let mut u32_ref_to_u32_iter =
    utf32_ref_iter_to_utf32_iter(& mut u32_ref_iter);

    let u8_array: [u8; 6] = [21, 33, 45, 58, 64, 90];
    let mut u8_ref_iter = u8_array.iter();
    let mut u8_ref_to_u8_iter =
    utf8_ref_iter_to_utf8_iter(& mut u8_ref_iter);

    assert_eq!(char::MAX, char_ref_to_char_iter.next().unwrap());
    assert_eq!(char::REPLACEMENT_CHARACTER, char_ref_to_char_iter.next().unwrap());
    assert_eq!(Option::None, char_ref_to_char_iter.next());

    assert_eq!(3, u32_ref_to_u32_iter.next().unwrap());
    assert_eq!(90, u32_ref_to_u32_iter.next().unwrap());
    assert_eq!(250, u32_ref_to_u32_iter.next().unwrap());
    assert_eq!(870, u32_ref_to_u32_iter.next().unwrap());
    assert_eq!(2300, u32_ref_to_u32_iter.next().unwrap());
    assert_eq!(Option::None, u32_ref_to_u32_iter.next());

    assert_eq!(21, u8_ref_to_u8_iter.next().unwrap());
    assert_eq!(33, u8_ref_to_u8_iter.next().unwrap());
    assert_eq!(45, u8_ref_to_u8_iter.next().unwrap());
    assert_eq!(58, u8_ref_to_u8_iter.next().unwrap());
    assert_eq!(64, u8_ref_to_u8_iter.next().unwrap());
    assert_eq!(90, u8_ref_to_u8_iter.next().unwrap());
    assert_eq!(Option::None, u8_ref_to_u8_iter.next());
}
