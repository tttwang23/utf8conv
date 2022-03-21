utf8conv
===


Parser for reading and converting between UTF8 / UTF32 data without requiring heap memory, targeting **no_std** environments

The need for utf8conv library occurred to me when I saw the conventional pattern for handling UTF8 data in the Rust standard library was to stream into potentially large string objects backed by heap memory.

There is value to provide an alternative [UTF8](https://en.wikipedia.org/wiki/UTF-8) / UTF32 parser that is based directly on data access and conversion.  Utf8conv can operate based on a single input buffer, or a series of input buffers.  The outputs are produced one at a time, directly delivered to client caller with minimum latency.

The design of the parser is inspired by [nom](https://github.com/Geal/nom).

The design of the finite state machine used by utf8conv was clarified after reading a technical design article written by [Henri Sivonen](https://hsivonen.fi/broken-utf-8/).

Utf8conv is dual licensed under the [MIT License](https://mit-license.org/), and [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).

#### Code Example

```
    fn simple_example() {
        let mybuffer = "abc\n".as_bytes();
        let mut parser = Utf8ArrayParser::new();
        let mut byte_slice = mybuffer;
        loop {
            match parser.parse_utf8_to_char(byte_slice)
            {
                Result::Ok((next_slice, ch)) => {
                    byte_slice = next_slice;
                    print!("{}", ch);
                }
                Result::Err(_) => {
                    // for a single buffer Err is always end of data
                    break;
                }
            }
        }
    }
```
