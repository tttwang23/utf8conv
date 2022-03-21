// Copyright 2022 Thomas Wang and utf8conv contributors

// Module is crate::utf8conv::buf

const BUFFER_SIZE:u32 = 8;
const SHIFT_MASK:u32 = 63; // maximum shift amount of a 64 bit word is 63

#[derive(Debug, Clone, Copy)]
/// This is an implementation of a simple FIFO buffer containing u8 values with
/// storage size of 8.  Stored values can be retrieved "first-in, first-out" order.
/// Single threaded usage is intended.
pub struct FifoU8 {
    buf: u64,
    mylen: u32,
    writeloc: u32,
}

impl FifoU8 {

    /// Creates a new FifoU8.
    #[inline]
    pub fn new() -> FifoU8 {
        FifoU8 {
            buf: 0,
            mylen: 0,
            writeloc: 0,
        }
    }

    // Clears the contents of this buffer.
    // The number of elements would become zero.
    #[inline]
    pub fn clear_all(& mut self) {
        self.buf = 0u64;
        self.mylen = 0u32;
        self.writeloc = 0u32;
    }

    // Returns the maximum capacity of this buffer.
    #[inline]
    pub fn max_capacity(&self) -> u32 {
        BUFFER_SIZE
    }

    // Returns the number of elements in this buffer.
    #[inline]
    pub fn num_elem(&self) -> u32 {
        self.mylen
    }

    /// Returns true if this buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.mylen == 0
    }

    /// Returns true if this buffer is full.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.mylen == self.max_capacity()
    }

    #[inline]
    /// Push a value into this buffer.  'None' is returned if buffer is full.
    pub fn push(& mut self, v:u8) -> Option<()> {
        if self.is_full() {
            Option::None
        }
        else {
            let opword = (v as u64) << self.writeloc;
            self.buf += opword;
            self.mylen += 1;
            self.writeloc = (self.writeloc + 8) & SHIFT_MASK;
            Option::Some(())
        }
    }

    #[inline]
    /// Pull a value out of this buffer.  'None' is returned if buffer is empty.
    pub fn pull(& mut self) -> Option<u8> {
        if self.is_empty() {
            Option::None
        }
        else {
            let res = self.buf as u8;
            self.buf >>= 8;
            self.mylen -= 1;
            self.writeloc = self.writeloc.wrapping_sub(8) & SHIFT_MASK;
            Option::Some(res)
        }
    }

    #[inline]
    /// Peek at the stored value.  'None' is returned if
    /// there is nothing stored there.
    pub fn peek(&self) -> Option<u8> {
        if self.is_empty() {
            Option::None
        }
        else {
            Option::Some(self.buf as u8)
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use crate::utf8conv::buf::FifoU8;

    #[test]
    /// Simple ringbuffer test
    fn test_fifo_aaa() {
        let mut b1:FifoU8 = FifoU8::new();
        assert_eq!(b1.max_capacity(), 8);
        assert_eq!(b1.num_elem(), 0);
        assert_eq!(b1.is_empty(), true);
        assert_eq!(b1.is_full(), false);
        assert_eq!(b1.pull(), Option::None);
        assert_eq!(b1.peek(), Option::None);
        assert_eq!(b1.push(11u8), Option::Some(()));
        assert_eq!(b1.num_elem(), 1);
        assert_eq!(b1.is_empty(), false);
        assert_eq!(b1.is_full(), false);
        b1.clear_all();
        assert_eq!(b1.is_empty(), true);
        assert_eq!(b1.is_full(), false);
        assert_eq!(b1.num_elem(), 0u32);
        b1.push(11u8);
        assert_eq!(b1.peek(), Option::Some(11u8));
        assert_eq!(b1.push(12u8), Option::Some(()));
        assert_eq!(b1.push(13u8), Option::Some(()));
        assert_eq!(b1.push(14u8), Option::Some(()));
        assert_eq!(b1.push(15u8), Option::Some(()));
        assert_eq!(b1.push(16u8), Option::Some(()));
        assert_eq!(b1.push(17u8), Option::Some(()));
        assert_eq!(b1.push(18u8), Option::Some(()));
        assert_eq!(b1.push(19u8), Option::None);
        assert_eq!(b1.pull(), Option::Some(11u8));
        assert_eq!(b1.pull(), Option::Some(12u8));
        assert_eq!(b1.pull(), Option::Some(13u8));
        assert_eq!(b1.pull(), Option::Some(14u8));
        assert_eq!(b1.pull(), Option::Some(15u8));
        assert_eq!(b1.pull(), Option::Some(16u8));
        assert_eq!(b1.pull(), Option::Some(17u8));
        assert_eq!(b1.pull(), Option::Some(18u8));
        assert_eq!(b1.pull(), Option::None);
    }

    #[test]
    /// Test pusing to full, then empty.
    fn test_fifou8_add_del() {
        let mut b1:FifoU8 = FifoU8::new();
        for indx in 0u32 .. b1.max_capacity() + 1 {
            if indx < b1.max_capacity() {
                assert_eq!(indx, b1.num_elem());
                assert_eq!(! b1.is_full(), true);
                assert_eq!(b1.push(indx as u8), Option::Some(()));
            }
            else {
                assert_eq!(b1.push(indx as u8), Option::None);
                assert_eq!(b1.is_full(), true);
            }
        }
        assert_eq!(Option::Some(0u8), b1.peek());
        for indx in 0u32 .. b1.max_capacity() + 1 {
            if indx < b1.max_capacity() {
                assert_eq!(b1.max_capacity() - indx, b1.num_elem());
                assert_eq!(! b1.is_empty(), true);
                assert_eq!(b1.pull(), Option::Some(indx as u8));
            }
            else {
                assert_eq!(b1.pull(), Option::None);
                assert_eq!(b1.is_empty(), true);
            }
        }
    }

    #[test]
    /// Randomized buffer push / pull / peek.
    fn test_fifou8_random() {
        use rand::Rng;
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        // use rand::RngCore;

        let mut b1:FifoU8 = FifoU8::new();
        let mut rng = SmallRng::seed_from_u64(0x12e415a46274f230u64);
        for _indx in 0usize .. 3000usize {
            let dice: f64 = rng.gen();
            if dice < 0.33 {
                if ! b1.is_empty() {
                    let old_len = b1.num_elem();
                    let x = b1.peek();
                    match b1.pull() {
                        Some(y) => {
                            let new_len = b1.num_elem();
                            assert_eq!(new_len + 1, old_len);
                            // Check deleted item is peek(0).
                            assert_eq!(x, Option::Some(y));
                        }
                        None => {
                            panic!("Pull did not remove element.");
                        }
                    }
                }
            }
            else if dice < 0.63 {
                if ! b1.is_full() {
                    let old_len = b1.num_elem();
                    let val = rng.gen_range(0..255) as u8;
                    match b1.push(val) {
                        Some(_) => {}
                        None => {
                            panic!("Push did not add element.");
                        }
                    }
                    let new_len = b1.num_elem();
                    assert_eq!(new_len - 1, old_len);
                }
            }
            else {
                if b1.num_elem() >= 1 {
                    match b1.peek() {
                        Some(_) => {}
                        None => {
                            panic!("Peek did not detect an element.");
                        }
                    }
                }
            }
        }
    }
}
