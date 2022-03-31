utf8conv
===


Parser for reading and converting between [UTF8](https://en.wikipedia.org/wiki/UTF-8) / UTF32 data without requiring heap memory, targeting **no_std** environments

Utf8conv can operate based on a single input buffer, or a series of input buffers.  The outputs are produced one at a time, directly delivered to client caller with minimum latency.

Includes an adapter iterator to filter out Byte Order Mark at the beginning of a stream, and substituting carriage returns with newlines.

Utf8conv is dual licensed under the [MIT License](https://mit-license.org/), and [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).

Source Repository: [link](https://github.com/tttwang23/utf8conv)

Credits attribution of utf8conv is located in source directory doc/utf8conv-credits.md.

#### Single buffer iterator based parsing

```rust
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
```

#### Multi-buffer iterator based parsing

```rust
use utf8conv::*;

/// Multi-buffer iterator based UTF8 parsing converting to char
fn utf8_to_char_multi_buffer_iterator() {
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

/// Multi-buffer iterator based char parsing converting to UTF8
fn char_to_utf8_multi_buffer_iterator() {
    let mybuffers = [[ '\u{7F}', '\u{80}' ] , [ '\u{81}', '\u{82}' ]];
    let mut parser = FromUnicode::new();
    for indx in 0 .. mybuffers.len() {
        parser.set_is_last_buffer(indx == mybuffers.len() - 1);
        let mut char_ref_iter = mybuffers[indx].iter();
        let mut iterator = parser.char_ref_to_utf8_with_iter(& mut char_ref_iter);
        while let Some(utf8_val) = iterator.next()  {
            println!("{:#02x}", utf8_val);
            println!("{}", iterator.has_invalid_sequence());
        }
    }
}
```

#### Single-buffer slice based parsing

```rust
use utf8conv::*;

/// Single-buffer slice reading based UTF8 parsing converting to char
fn utf8_to_char_single_buffer_slice_reading() {
    let mybuffer = "Wxyz".as_bytes();
    let mut parser = FromUtf8::new();
    let mut cur_slice = mybuffer;
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

/// Single-buffer slice reading based UTF32 parsing converting to UTF8
fn utf32_to_utf8_single_buffer_slice_reading() {
    let mybuffer = [0x7Fu32, 0x80u32, 0x81u32, 0x82u32];
    let mut parser = FromUnicode::new();
    let mut current_slice = & mybuffer[..];
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
```

#### Multi-buffer slice based parsing

```rust
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
```
