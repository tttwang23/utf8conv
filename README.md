utf8conv
===


Parser for reading and converting between [UTF8](https://en.wikipedia.org/wiki/UTF-8) / UTF32 data without requiring heap memory, targeting **no_std** environments

Utf8conv can operate based on a single input buffer, or a series of input buffers.  The outputs are produced one at a time, directly delivered to client caller with minimum latency.

Utf8conv is dual licensed under the [MIT License](https://mit-license.org/), and [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).

Credits attribution of utf8conv is located at crate directory doc/utf8conv-credits.md.

#### Single buffer iterator based UTF8 parsing

```
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

```

#### Multi-buffer iterator based UTF8 parsing

```
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
```

#### Multi-buffer slice reading based UTF8 parsing

```
use utf8conv::prelude::*;

/// Multi-buffer slice reading based UTF8 parsing
/// converting to char
fn main() {
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
```
