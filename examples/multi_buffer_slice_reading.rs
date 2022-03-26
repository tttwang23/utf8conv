// Copyright 2022 Thomas Wang and utf8conv contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use utf8conv::prelude::*;

/// Multi-buffer slice reading based UTF8 parsing
/// converting to char
fn main() {
    let mybuffers = ["Wx".as_bytes(), "y".as_bytes(), "z\n".as_bytes()];
    let mut parser = FromUtf8::new();
    for indx in 0 .. mybuffers.len() {
        parser.set_is_last_buffer(indx == mybuffers.len() - 1);
        let mut cur_slice = mybuffers[indx];
        loop {
            match parser.utf8_to_char(cur_slice) {
                Result::Ok((slice_pos, char_val)) => {
                    cur_slice = slice_pos;
                    print!("{}", char_val);
                }
                Result::Err(MoreEnum::More(_amt)) => {
                    // _amt equals to 0 when end of data
                    break;
                }
            }
        }
    }
}
