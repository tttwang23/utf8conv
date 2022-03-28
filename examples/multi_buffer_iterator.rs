// Copyright 2022 Thomas Wang and utf8conv contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use utf8conv::prelude::*;

/// Multi-buffer iterator based UTF8 parsing
/// converting to char
fn main() {
    let mybuffers = ["ab".as_bytes(), "".as_bytes(), "cde".as_bytes()];
    let mut parser = FromUtf8::new();
    for indx in 0 .. mybuffers.len() {
        parser.set_is_last_buffer(indx == mybuffers.len() - 1);
        let mut utf8_ref_iter = mybuffers[indx].iter();
        let mut iterator = parser.utf8_ref_to_char_with_iter(& mut utf8_ref_iter);
        while let Some(char_val) = iterator.next()  {
            println!("{}", char_val);
            println!("{}", iterator.has_invalid_sequence());
        }
    }
}
