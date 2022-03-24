// Copyright 2022 Thomas Wang and utf8conv contributors

// Module is crate::utf8conv::buf

const BUFFER_SIZE:u32 = 8;

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, Eq)]
/// This is an implementation of a simple FIFO buffer containing byte values
/// with storage size of 8.  Stored values can be retrieved
/// "first-in, first-out" order.  Single threaded usage is intended.
pub struct FifoBytes {
    buf: u64,
    mylen: u32,
}

/// PartialEq implementation
impl PartialEq for FifoBytes {
    fn eq(&self, other: &Self) -> bool {
        (self.mylen == other.mylen) && (self.buf == other.buf)
    }
}

/// Ord implementation
/// Longer length FifoBytes being greater, followed by
/// comparison of most recently pushed bytes
///
/// This object is mutable; do not put FifoBytes in a collection
/// if its state will change during its residence.
impl Ord for FifoBytes {
    fn cmp(&self, other: &Self) -> Ordering {
        let len1 = self.mylen;
        let len2 = other.mylen;
        if len1 > len2 {
            Ordering::Greater
        }
        else if len1 < len2 {
            Ordering::Less
        }
        else {
            let word1 = self.buf;
            let word2 = other.buf;
            if word1 > word2 {
                Ordering::Greater
            }
            else if word1 < word2 {
                Ordering::Less
            }
            else {
                Ordering::Equal
            }
        }
    }
}

/// PartialOrd implementation
impl PartialOrd for FifoBytes {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }

}

/// Hash implementation
///
/// This object is mutable; do not put FifoBytes in a collection
/// if its state will change during its residence.
impl Hash for FifoBytes {
    fn hash<H: Hasher>(&self, state: & mut H) {
        self.mylen.hash(state);
        self.buf.hash(state);
    }
}

/// Implementation of FifoBytes
impl FifoBytes {

    /// Creates a new FifoBytes.
    #[inline]
    pub fn new() -> FifoBytes {
        FifoBytes {
            buf: 0,
            mylen: 0,
        }
    }

    // Clears the contents of this FifoBytes.
    // The number of elements would become zero.
    #[inline]
    pub fn clear(& mut self) {
        self.buf = 0u64;
        self.mylen = 0u32;
    }

    // Returns the maximum capacity of this buffer.
    #[inline]
    pub fn capacity(&self) -> u32 {
        BUFFER_SIZE
    }

    // Returns the number of elements in this buffer.
    #[inline]
    pub fn len(&self) -> u32 {
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
        self.mylen >= self.capacity()
    }

    #[inline]
    /// Push a value to the back of the buffer.
    /// No action performed if buffer is full.
    pub fn push_back(& mut self, v:u8) {
        if ! self.is_full() {
            // curlen can be from 0 to 7 when it is not full
            // so curlen * 8 always less than 64
            let curlen = self.mylen;
            let opword = (v as u64) << (curlen << 3);
            self.buf += opword;
            self.mylen = curlen + 1;
        }
    }

    #[inline]
    /// Removes the first element and return it.
    /// 'None' is returned if buffer is empty.
    pub fn pop_front(& mut self) -> Option<u8> {
        if self.is_empty() {
            Option::None
        }
        else {
            let res = self.buf;
            self.buf = res >> 8;
            self.mylen -= 1;
            Option::Some(res as u8)
        }
    }

    #[inline]
    /// Peek at the first element without removing it.
    /// 'None' is returned if there is nothing stored there.
    pub fn front(&self) -> Option<u8> {
        if self.is_empty() {
            Option::None
        }
        else {
            Option::Some(self.buf as u8)
        }
    }
}

/// Implementation of Default trait
impl Default for FifoBytes {
    /// Return an empty array
    fn default() -> FifoBytes {
        FifoBytes::new()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use crate::utf8conv::buf::FifoBytes;

    #[test]
    /// Simple ringbuffer test
    fn test_fifo_aaa() {
        let mut b1:FifoBytes = FifoBytes::new();
        assert_eq!(b1.capacity(), 8);
        assert_eq!(b1.len(), 0);
        assert_eq!(b1.is_empty(), true);
        assert_eq!(b1.is_full(), false);
        assert_eq!(b1.pop_front(), Option::None);
        assert_eq!(b1.front(), Option::None);
        b1.push_back(11u8);
        assert_eq!(b1.len(), 1);
        assert_eq!(b1.is_empty(), false);
        assert_eq!(b1.is_full(), false);
        b1.clear();
        assert_eq!(b1.is_empty(), true);
        assert_eq!(b1.is_full(), false);
        assert_eq!(b1.len(), 0u32);
        b1.push_back(11u8);
        assert_eq!(b1.front(), Option::Some(11u8));
        b1.push_back(12u8);
        b1.push_back(13u8);
        b1.push_back(14u8);
        b1.push_back(15u8);
        b1.push_back(16u8);
        b1.push_back(17u8);
        b1.push_back(18u8);
        b1.push_back(19u8);
        assert_eq!(b1.pop_front(), Option::Some(11u8));
        assert_eq!(b1.pop_front(), Option::Some(12u8));
        assert_eq!(b1.pop_front(), Option::Some(13u8));
        assert_eq!(b1.pop_front(), Option::Some(14u8));
        assert_eq!(b1.pop_front(), Option::Some(15u8));
        assert_eq!(b1.pop_front(), Option::Some(16u8));
        assert_eq!(b1.pop_front(), Option::Some(17u8));
        assert_eq!(b1.pop_front(), Option::Some(18u8));
        assert_eq!(b1.pop_front(), Option::None);
    }

    #[test]
    /// Test pusing to full, then empty.
    fn test_fifobytes_add_del() {
        let mut b1:FifoBytes = FifoBytes::new();
        for indx in 0u32 .. b1.capacity() + 1 {
            if indx < b1.capacity() {
                assert_eq!(indx, b1.len());
                assert_eq!(! b1.is_full(), true);
                b1.push_back(indx as u8);
            }
            else {
                b1.push_back(indx as u8);
                assert_eq!(b1.is_full(), true);
            }
        }
        assert_eq!(Option::Some(0u8), b1.front());
        for indx in 0u32 .. b1.capacity() + 1 {
            if indx < b1.capacity() {
                assert_eq!(b1.capacity() - indx, b1.len());
                assert_eq!(! b1.is_empty(), true);
                assert_eq!(b1.pop_front(), Option::Some(indx as u8));
            }
            else {
                assert_eq!(b1.pop_front(), Option::None);
                assert_eq!(b1.is_empty(), true);
            }
        }
    }

    #[test]
    /// Randomized buffer push_back / pop_front / front.
    fn test_fifobytes_random() {
        use rand::Rng;
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        // use rand::RngCore;

        let mut b1:FifoBytes = FifoBytes::new();
        let mut rng = SmallRng::seed_from_u64(0x12e415a46274f230u64);
        for _indx in 0usize .. 3000usize {
            let dice: f64 = rng.gen();
            if dice < 0.33 {
                if ! b1.is_empty() {
                    let old_len = b1.len();
                    let x = b1.front();
                    match b1.pop_front() {
                        Some(y) => {
                            let new_len = b1.len();
                            assert_eq!(new_len + 1, old_len);
                            // Check deleted item is front(0).
                            assert_eq!(x, Option::Some(y));
                        }
                        None => {
                            panic!("pop_front did not remove element.");
                        }
                    }
                }
            }
            else if dice < 0.63 {
                if ! b1.is_full() {
                    let old_len = b1.len();
                    let val = rng.gen_range(0..255) as u8;
                    b1.push_back(val);
                    let new_len = b1.len();
                    assert_eq!(new_len - 1, old_len);
                }
            }
            else {
                if b1.len() >= 1 {
                    match b1.front() {
                        Some(_) => {}
                        None => {
                            panic!("front did not detect an element.");
                        }
                    }
                }
            }
        }
    }
}
