// Copyright 2022 Thomas Wang and utf8conv contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::error::Error;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use utf8conv::*;

/// Example program demonstrating converting UTF8 to char while reading a file.
/// Exercise slice parsing style API.



/// Read bytes without I/O interrupts.
/// Returns the number of bytes read or an IO Error.
/// # Arguments
/// * `read_obj` - a mutable Read trait reference
/// * `buf` - a mutable byte slice
fn read_bytes_without_interrupt(
read_obj: & mut (impl Read + ?Sized), buf: & mut [u8]) -> io::Result<usize> {
    loop {
        match read_obj.read(buf) {
            Ok(num_bytes) => { return Ok(num_bytes); }
            Err(er) => {
                if er.kind() != io::ErrorKind::Interrupted {
                    return Err(er);
                }
            }
        }
    }
}

/// Print out the contents of a file.
/// Returns unit value or an IO error.
/// # Arguments
/// * `read_obj` - a mutable Read trait reference
/// * `buf` - a mutable byte slice
fn real_io_slice(
read_obj: & mut (impl Read + ?Sized), buf: & mut [u8]) -> io::Result<()> {
    let mut parser = FromUtf8::new();
    loop {
        // Handle one buffer operation.
        match read_bytes_without_interrupt(read_obj, buf) {
            Err(er) => { return Err(er); }
            Ok(num_bytes) => {
                // Indicate last buffer if end of file.
                // If no more data then parser will drain scratch-pad.
                parser.set_is_last_buffer(num_bytes == 0);
                let mut buf_slice = & buf[0 .. num_bytes];
                // Loop over characters.
                loop {
                    match parser.utf8_to_char(buf_slice) {
                        Result::Ok((slice_pos, char_val)) => {
                            // Update new read position.
                            buf_slice = slice_pos;
                            // Print the character.
                            print!("{}", char_val);
                        }
                        Result::Err(MoreEnum::More(amt)) => {
                            if amt == 0 {
                                // amt equals to 0 when end of data
                                // Scratch-pad drained and done
                                return Ok(());
                            }
                            else {
                                // Request more data.
                                // Drop to outer loop.
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// main function;
/// returns unit value or an IO error.
fn main()  -> Result<(), Box<dyn Error>> {
    // Small demo buffer; regular size would be 4096.
    let mut buf: [u8; 128] = [0u8; 128];
    // Print the source file itself.
    let mut f = File::open("examples/real_io_slice.rs") ?;
    real_io_slice(& mut f, & mut buf[..]) ?;
    Ok(())
}
