use std::collections::HashSet;

pub type Bits = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BitSet {
    bits: Bits,
}

impl BitSet {
    pub fn new() -> Self {
        BitSet { bits: 0 }
    }

    pub fn from_bits(bits: Bits) -> Self {
        BitSet { bits }
    }

    pub fn insert(&mut self, pos: usize) {
        self.bits |= 1 << pos;
    }

    pub fn remove(&mut self, pos: usize) {
        self.bits &= !(1 << pos);
    }

    pub fn contains(&self, pos: usize) -> bool {
        (self.bits & (1 << pos)) != 0
    }

    pub fn has_one_bit(&self) -> bool {
        // This is true if bits is a power of two (i.e. one bit to 1).
        self.bits != 0 && (self.bits & (self.bits - 1)) == 0
    }

    pub fn first_bit(&self) -> Option<usize> {
        if self.bits == 0 {
            None
        } else {
            Some(self.bits.trailing_zeros() as usize)
        }
    }

    pub fn to_hashset(&self) -> HashSet<usize> {
        let mut set = HashSet::new();
        for bit in *self {
            set.insert(bit as usize);
        }
        set
    }

    pub fn to_vec(&self) -> Vec<usize> {
        let mut vec = Vec::new();
        for bit in *self {
            vec.push(bit as usize);
        }
        vec
    }

    pub fn len(&self) -> usize {
        self.bits.count_ones() as usize
    }
}

impl std::ops::BitOr for BitSet {
    type Output = BitSet;

    fn bitor(self, rhs: BitSet) -> BitSet {
        BitSet {
            bits: self.bits | rhs.bits,
        }
    }
}

impl std::ops::BitOrAssign for BitSet {
    fn bitor_assign(&mut self, rhs: BitSet) {
        self.bits |= rhs.bits;
    }
}

impl std::ops::BitAnd for BitSet {
    type Output = BitSet;

    fn bitand(self, rhs: BitSet) -> BitSet {
        BitSet {
            bits: self.bits & rhs.bits,
        }
    }
}

impl std::ops::Not for BitSet {
    type Output = BitSet;

    fn not(self) -> BitSet {
        BitSet {
            bits: !self.bits,
        }
    }
}

impl Iterator for BitSet {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            return None
        }

        let bit = self.bits.trailing_zeros();
        self.bits &= self.bits - 1;
        Some(bit as usize)
    }
}
