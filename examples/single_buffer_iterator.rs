// Copyright 2022 Thomas Wang and utf8conv contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use utf8conv::*;

/// Single buffer iterator based UTF8 parsing converting to char
fn utf8_to_char_single_buffer_iterator() {
    let mybuffer = "abc".as_bytes();
    let mut utf8_ref_iter = mybuffer.iter();
    let mut parser = FromUtf8::new();
    let mut iterator = parser.utf8_ref_to_char_with_iter(& mut utf8_ref_iter);
    while let Some(char_val) = iterator.next()  {
        println!("{}", char_val);
        println!("{}", iterator.has_invalid_sequence());
    }
}

/// Single buffer iterator based char parsing converting to UTF8
fn char_to_utf8_single_buffer_iterator() {
    let mybuffer = [ '\u{7F}', '\u{80}', '\u{81}', '\u{82}' ];
    let mut char_ref_iter = mybuffer.iter();
    let mut parser = FromUnicode::new();
    let mut iterator = parser.char_ref_to_utf8_with_iter(& mut char_ref_iter);
    while let Some(utf8_val) = iterator.next()  {
        println!("{:#02x}", utf8_val);
        println!("{}", iterator.has_invalid_sequence());
    }
}

fn main() {
    utf8_to_char_single_buffer_iterator();
    println!("");
    char_to_utf8_single_buffer_iterator();
}
