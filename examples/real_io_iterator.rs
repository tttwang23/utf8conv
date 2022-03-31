// Copyright 2022 Thomas Wang and utf8conv contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// Example program demonstrating converting UTF8 to char while reading a file.
/// Exercise iterator converter style API.

use std::error::Error;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use utf8conv::*;

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
fn real_io_iterator(
read_obj: & mut (impl Read + ?Sized), buf: & mut [u8]) -> io::Result<()> {
    let mut parser = FromUtf8::new();
    loop {
        // Handle one buffer operation.
        match read_bytes_without_interrupt(read_obj, buf) {
            Err(er) => { return Err(er); }
            Ok(num_bytes) => {
                // Indicate last buffer if reading complete.
                // If no more data then parser will drain scratch-pad.
                parser.set_is_last_buffer(num_bytes == 0);
                let buf_slice = & buf[0 .. num_bytes];
                let mut utf8_ref_iter = buf_slice.iter();
                let mut iterator = parser.utf8_ref_to_char_with_iter(& mut utf8_ref_iter);
                // Loop over characters.
                while let Some(char_val) = iterator.next()  {
                    print!("{}", char_val);
                }
                // Finished with this buffer.  Check if we should return.
                if num_bytes == 0 {
                    break Ok(());
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
    let mut f = File::open("examples/real_io_iterator.rs") ?;
    real_io_iterator(& mut f, & mut buf[..]) ?;
    Ok(())
}
