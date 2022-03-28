// Copyright 2022 Thomas Wang and utf8conv contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use utf8conv::prelude::*;

/// Single buffer iterator based UTF8 parsing
/// converting to char
fn main() {
    let mybuffer = "abc".as_bytes();
    let mut utf8_ref_iter = mybuffer.iter();
    let mut parser = FromUtf8::new();
    let mut iterator = parser.utf8_ref_to_char_with_iter(& mut utf8_ref_iter);
    while let Some(char_val) = iterator.next()  {
        println!("{}", char_val);
        println!("{}", iterator.has_invalid_sequence());
    }
}
