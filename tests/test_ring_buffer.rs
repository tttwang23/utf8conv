// Copyright 2022 Thomas Wang and utf8conv contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use utf8conv::prelude::*;
use core::cmp::Ordering;

#[test]
/// Simple ringbuffer test
fn test_ringbuffer_misc() {
    let mut b1:EightBytes = EightBytes::new();
    let mut b2:EightBytes = EightBytes::new();
    assert_eq!(true, b1.eq(&b2));
    assert_eq!(Option::Some(Ordering::Equal), b1.partial_cmp(&b2));
    b1.push_back(12u8);
    assert_eq!(Ordering::Greater, b1.cmp(&b2));
    assert_eq!(Option::Some(Ordering::Less), b2.partial_cmp(&b1));
    b2.push_back(13u8);
    assert_eq!(Ordering::Greater, b2.cmp(&b1));
    assert_eq!(Option::Some(Ordering::Less), b1.partial_cmp(&b2));
}
