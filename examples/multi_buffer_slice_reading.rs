// Copyright 2022 Thomas Wang and utf8conv contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use utf8conv::*;

/// Multi-buffer slice reading based UTF8 parsing converting to char
fn utf8_to_char_multi_buffer_slice_reading() {
    let mybuffers = ["Wx".as_bytes(), "".as_bytes(), "yz".as_bytes()];
    let mut parser = FromUtf8::new();
    for indx in 0 .. mybuffers.len() {
        parser.set_is_last_buffer(indx == mybuffers.len() - 1);
        let mut cur_slice = mybuffers[indx];
        loop {
            match parser.utf8_to_char(cur_slice) {
                Result::Ok((slice_pos, char_val)) => {
                    cur_slice = slice_pos;
                    println!("{}", char_val);
                    println!("{}", parser.has_invalid_sequence());
                }
                Result::Err(MoreEnum::More(_amt)) => {
                    // _amt equals to 0 when end of data
                    break;
                }
            }
        }
    }
}

/// Multi-buffer slice reading based UTF32 parsing converting to UTF8
fn utf32_to_utf8_multi_buffer_slice_reading() {
    let mybuffers = [[0x7Fu32, 0x80u32], [0x81u32, 0x82u32]];
    let mut parser = FromUnicode::new();
    for indx in 0 .. mybuffers.len() {
        parser.set_is_last_buffer(indx == mybuffers.len() - 1);
        let current_array = mybuffers[indx];
        let mut current_slice = & current_array[..];
        loop {
            match parser.utf32_to_utf8(current_slice) {
                Result::Ok((slice_pos, utf8_val)) => {
                    current_slice = slice_pos;
                    println!("{:02x}", utf8_val);
                    println!("{}", parser.has_invalid_sequence());
                }
                Result::Err(MoreEnum::More(_amt)) => {
                    // _amt equals to 0 when end of data
                    break;
                }
            }
        }
    }
}

fn main() {
    utf8_to_char_multi_buffer_slice_reading();
    println!("");
    utf32_to_utf8_multi_buffer_slice_reading();
}
