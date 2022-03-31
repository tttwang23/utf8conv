// Copyright 2022 Thomas Wang and utf8conv contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Randomization based testing on UTF converters

extern crate stackfmt;
extern crate std;

use utf8conv::*;

use rand::SeedableRng;
use rand::rngs::SmallRng;
use rand::RngCore;

use core::str;

fn verify_style3(par: &mut FromUtf8, b1:& [u8], b2:& [u8], b3:& [u8], b4:& [u8], truth: &str) {
    let mut panic_buf = [0u8; 12000];
    let mut test_char_len:usize = 0;
    let mut truth_iter = truth.char_indices();
    par.reset_parser();
    par.set_is_last_buffer(false); // Set multi-buffer mode on.
    for stage in 0 .. 4 {
        let mut byte_ref_iter: std::slice::Iter<u8>;
        match stage {
            0 => {
                byte_ref_iter = b1.iter();
            }
            1 => {
                byte_ref_iter = b2.iter();
            }
            2 => {
                byte_ref_iter = b3.iter();
            }
            _ => {
                byte_ref_iter = b4.iter();
                // Signal no more buffer after the current one.
                par.set_is_last_buffer(true);
            }
        }
        let mut glue_iter = utf8_ref_iter_to_utf8_iter(& mut byte_ref_iter);
        for test_ch in par.utf8_to_char_with_iter(& mut glue_iter) {
            let test_utf32 = test_ch as u32;
            match truth_iter.next() {
                Option::Some((_pos, truth_ch)) => {
                    let truth_utf32 = truth_ch as u32;
                    if test_utf32 != truth_utf32 {
                        let formatted: &str = stackfmt::fmt_truncate(&mut panic_buf, format_args!(
"The truth string is different than the test vectors.\nTruth string:{}\nTest vector at index {} has code value {:#08x} vs truth {:#08x}"
                        ,truth, test_char_len, test_utf32, truth_utf32));
                        panic!("\n{}\n", formatted);
                    }
                }
                Option::None => {
                    let formatted: &str = stackfmt::fmt_truncate(&mut panic_buf, format_args!(
"The truth string is shorter than the combined test vectors.\nTruth string:{}\nTest vector at index {} has code value {:#08x}"
                    , truth, test_char_len, test_utf32));
                    panic!("\n{}\n",formatted);
                }
            }
            test_char_len += 1;
        }
        if par.is_last_buffer() {
            match truth_iter.next() {
                Option::Some((_pos, truth_ch)) => {
                    let formatted: &str = stackfmt::fmt_truncate(&mut panic_buf, format_args!(
"The truth string is longer than the test vectors (length {}).\nTruth string:{}\nTruth at index {} has code value {:#08x}"
                    , test_char_len, truth, test_char_len+1, (truth_ch as u32)));
                    panic!("\n{}\n", formatted);
                }
                Option::None => {
                    // Truth and test vector ran out at the same time.  Test passed.
                }
            }
        }
    }
}

fn verify_style2(par: &mut FromUtf8, b1:& [u8], b2:& [u8], b3:& [u8], b4:& [u8], truth: &str) {
    let mut panic_buf = [0u8; 12000];
    let mut test_char_len:usize = 0;
    let mut truth_iter = truth.char_indices();
    par.reset_parser();
    par.set_is_last_buffer(false); // Set multi-buffer mode on.
    for stage in 0 .. 4 {
        let mut byte_ref_iter: std::slice::Iter<u8>;
        match stage {
            0 => {
                byte_ref_iter = b1.iter();
            }
            1 => {
                byte_ref_iter = b2.iter();
            }
            2 => {
                byte_ref_iter = b3.iter();
            }
            _ => {
                byte_ref_iter = b4.iter();
                // Signal no more buffer after the current one.
                par.set_is_last_buffer(true);
            }
        }
        for test_ch in par.utf8_ref_to_char_with_iter(& mut byte_ref_iter) {
            match truth_iter.next() {
                Option::Some((_pos, truth_ch)) => {
                    if test_ch != truth_ch {
                        let formatted: &str = stackfmt::fmt_truncate(&mut panic_buf, format_args!(
"The truth string is different than the test vectors.\nTruth string:{}\nTest vector at index {} has code value {:#08x} vs truth {:#08x}"
                        ,truth, test_char_len, (test_ch as u32), (truth_ch as u32)));
                        panic!("\n{}\n", formatted);
                    }
                }
                Option::None => {
                    let formatted: &str = stackfmt::fmt_truncate(&mut panic_buf, format_args!(
"The truth string is shorter than the combined test vectors.\nTruth string:{}\nTest vector at index {} has code value {:#08x}"
                    , truth, test_char_len, (test_ch as u32)));
                    panic!("\n{}\n",formatted);
                }
            }
            test_char_len += 1;
        }
        if par.is_last_buffer() {
            match truth_iter.next() {
                Option::Some((_pos, truth_ch)) => {
                    let formatted: &str = stackfmt::fmt_truncate(&mut panic_buf, format_args!(
"The truth string is longer than the test vectors (length {}).\nTruth string:{}\nTruth at index {} has code value {:#08x}"
                    , test_char_len, truth, test_char_len+1, (truth_ch as u32)));
                    panic!("\n{}\n", formatted);
                }
                Option::None => {
                    // Truth and test vector ran out at the same time.  Test passed.
                }
            }
        }
    }
}

fn verify_with_string(par: &mut FromUtf8, b1:& [u8], b2:& [u8], b3:& [u8], b4:& [u8], truth: &str) {
    let mut panic_buf = [0u8; 12000];
    let mut test_char_len:usize = 0;
    let mut truth_iter = truth.char_indices();
    par.reset_parser();
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
                // Signal no more buffer after the current one.
                par.set_is_last_buffer(true);
            }
        }
        loop {
            match par.utf8_to_char(the_slice) {
                Result::Ok((slice_pos, test_ch)) => {
                    the_slice = slice_pos;
                    match truth_iter.next() {
                        Option::Some((_pos, truth_ch)) => {
                            if test_ch != truth_ch {
                                let formatted: &str = stackfmt::fmt_truncate(&mut panic_buf, format_args!(
"The truth string is different than the test vectors.\nTruth string:{}\nTest vector at index {} has code value {:#08x} vs truth {:#08x}"
                                ,truth, test_char_len, (test_ch as u32), (truth_ch as u32)));
                                panic!("\n{}\n", formatted);
                            }
                        }
                        Option::None => {
                            let formatted: &str = stackfmt::fmt_truncate(&mut panic_buf, format_args!(
"The truth string is shorter than the combined test vectors.\nTruth string:{}\nTest vector at index {} has code value {:#08x}"
                            , truth, test_char_len, (test_ch as u32)));
                            panic!("\n{}\n",formatted);
                        }
                    }
                    test_char_len += 1;
                }
                Result::Err(en) => {
                    match en {
                        MoreEnum::More(i) => {
                            if i == 0 {
                                match truth_iter.next() {
                                    Option::Some((_pos, truth_ch)) => {
                                        let formatted: &str = stackfmt::fmt_truncate(&mut panic_buf, format_args!(
"The truth string is longer than the test vectors (length {}).\nTruth string:{}\nTruth at index {} has code value {:#08x}"
                                        , test_char_len, truth, test_char_len+1, (truth_ch as u32)));
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
/// Simple string conversion test
fn test_utf8parsing_aaa() {
    let mut par:FromUtf8 = FromUtf8::new();
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
fn test_utf8_monkey1() {
    let mut par:FromUtf8 = FromUtf8::new();
    let mut rng = SmallRng::seed_from_u64(0x17e4bd3a163c10e4u64);
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

#[test]
fn test_utf8_monkey2() {
    let mut par:FromUtf8 = FromUtf8::new();
    let mut rng = SmallRng::seed_from_u64(0x37e47d3a163c62b7u64);
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
                verify_style2(&mut par, frag1, frag2, frag3, frag4, mystr);
            }
            Err(_) => {
                panic!("Unexpected from_utf8() failure");
            }
        }
    }
}

#[test]
fn test_utf8_monkey3() {
    let mut par:FromUtf8 = FromUtf8::new();
    let mut rng = SmallRng::seed_from_u64(0x87e17f3a9c3a1a07u64);
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
                verify_style3(&mut par, frag1, frag2, frag3, frag4, mystr);
            }
            Err(_) => {
                panic!("Unexpected from_utf8() failure");
            }
        }
    }
}
