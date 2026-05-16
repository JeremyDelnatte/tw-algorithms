use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NewBitSet {
    Small(u64),
    Medium(u128),
    Large(Vec<u64>),
}

impl Hash for NewBitSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            NewBitSet::Small(x) => {
                0u8.hash(state);
                x.hash(state);
            }
            NewBitSet::Medium(x) => {
                1u8.hash(state);
                x.hash(state);
            }
            NewBitSet::Large(xs) => {
                2u8.hash(state);
                xs.hash(state);
            }
        }
    }
}

impl NewBitSet {
    pub fn new(n: usize) -> Self {
        match n {
            0..=64 => NewBitSet::Small(0),
            65..=128 => NewBitSet::Medium(0),
            _ => NewBitSet::Large(vec![0; (n + 63) / 64]),
        }
    }

    pub fn from_u64(bits: u64) -> Self {
        NewBitSet::Small(bits)
    }

    pub fn from_u128(bits: u128) -> Self {
        NewBitSet::Medium(bits)
    }

    pub fn from_blocks(blocks: Vec<u64>) -> Self {
        NewBitSet::Large(blocks)
    }

    pub fn insert(&mut self, pos: usize) {
        match self {
            NewBitSet::Small(bits) => *bits |= 1u64 << pos,
            NewBitSet::Medium(bits) => *bits |= 1u128 << pos,
            NewBitSet::Large(blocks) => blocks[pos / 64] |= 1u64 << (pos % 64),
        }
    }

    pub fn remove(&mut self, pos: usize) {
        match self {
            NewBitSet::Small(bits) => *bits &= !(1u64 << pos),
            NewBitSet::Medium(bits) => *bits &= !(1u128 << pos),
            NewBitSet::Large(blocks) => blocks[pos / 64] &= !(1u64 << (pos % 64)),
        }
    }

    pub fn contains(&self, pos: usize) -> bool {
        match self {
            NewBitSet::Small(bits) => (*bits & (1u64 << pos)) != 0,
            NewBitSet::Medium(bits) => (*bits & (1u128 << pos)) != 0,
            NewBitSet::Large(blocks) => {
                let idx = pos / 64;
                idx < blocks.len() && (blocks[idx] & (1u64 << (pos % 64))) != 0
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            NewBitSet::Small(bits) => bits.count_ones() as usize,
            NewBitSet::Medium(bits) => bits.count_ones() as usize,
            NewBitSet::Large(blocks) => blocks.iter().map(|b| b.count_ones() as usize).sum(),
        }
    }

    pub fn intersection_len(&self, rhs: &NewBitSet) -> usize {
        match (self, rhs) {
            (NewBitSet::Small(a), NewBitSet::Small(b)) => (*a & *b).count_ones() as usize,
            (NewBitSet::Medium(a), NewBitSet::Medium(b)) => (*a & *b).count_ones() as usize,
            (NewBitSet::Large(a), NewBitSet::Large(b)) => {
                a.iter()
                    .zip(b.iter())
                    .map(|(x, y)| (x & y).count_ones() as usize)
                    .sum()
            }
            _ => panic!("BitSet variants/capacities differ"),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            NewBitSet::Small(bits) => *bits == 0,
            NewBitSet::Medium(bits) => *bits == 0,
            NewBitSet::Large(blocks) => blocks.iter().all(|&b| b == 0),
        }
    }

    pub fn has_one_bit(&self) -> bool {
        self.len() == 1
    }

    pub fn first_bit(&self) -> Option<usize> {
        match self {
            NewBitSet::Small(bits) => {
                if *bits == 0 { None } else { Some(bits.trailing_zeros() as usize) }
            }
            NewBitSet::Medium(bits) => {
                if *bits == 0 { None } else { Some(bits.trailing_zeros() as usize) }
            }
            NewBitSet::Large(blocks) => {
                for (i, &block) in blocks.iter().enumerate() {
                    if block != 0 {
                        return Some(i * 64 + block.trailing_zeros() as usize);
                    }
                }
                None
            }
        }
    }

    pub fn to_vec(&self) -> Vec<usize> {
        self.iter().collect()
    }

    pub fn to_hashset(&self) -> HashSet<usize> {
        self.iter().collect()
    }

    pub fn iter(&self) -> BitSetIter<'_> {
        match self {
            NewBitSet::Small(bits) => BitSetIter::Small(*bits),
            NewBitSet::Medium(bits) => BitSetIter::Medium(*bits),
            NewBitSet::Large(blocks) => BitSetIter::Large {
                blocks,
                block_idx: 0,
                current: blocks.first().copied().unwrap_or(0),
            },
        }
    }

    // This method is used to shift all bits from max to pos to the right by one position, effectively removing the bit at pos and shifting all higher bits down.
    pub fn right_shift_from(&mut self, pos: usize) {
        match self {
            NewBitSet::Small(bits) => {
                let mask = (1u64 << pos) - 1;
                *bits = (*bits & mask) | ((*bits >> 1) & !mask);
            }
            NewBitSet::Medium(bits) => {
                let mask = (1u128 << pos) - 1;
                *bits = (*bits & mask) | ((*bits >> 1) & !mask);
            }
            NewBitSet::Large(blocks) => {
                let block_idx = pos / 64;
                let bit_idx = pos % 64;

                if block_idx >= blocks.len() {
                    return; // No bits to shift
                }

                // Shift the current block
                blocks[block_idx] = (blocks[block_idx] & ((1u64 << bit_idx) - 1)) | ((blocks[block_idx] >> 1) & !((1u64 << bit_idx) - 1));

                // Shift subsequent blocks
                for i in block_idx + 1..blocks.len() {
                    let carry = blocks[i] & 1; // Get the least significant bit to carry over
                    blocks[i] >>= 1; // Shift the current block
                    if carry != 0 {
                        blocks[i - 1] |= 1u64 << 63; // Set the most significant bit of the previous block if there was a carry
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum BitSetIter<'a> {
    Small(u64),
    Medium(u128),
    Large {
        blocks: &'a [u64],
        block_idx: usize,
        current: u64,
    },
}

impl Iterator for BitSetIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            BitSetIter::Small(bits) => {
                if *bits == 0 {
                    return None;
                }

                let bit = bits.trailing_zeros() as usize;
                *bits &= *bits - 1;
                Some(bit)
            }

            BitSetIter::Medium(bits) => {
                if *bits == 0 {
                    return None;
                }

                let bit = bits.trailing_zeros() as usize;
                *bits &= *bits - 1;
                Some(bit)
            }

            BitSetIter::Large {
                blocks,
                block_idx,
                current,
            } => loop {
                if *current != 0 {
                    let bit = current.trailing_zeros() as usize;
                    *current &= *current - 1;
                    return Some(*block_idx * 64 + bit);
                }

                *block_idx += 1;

                if *block_idx >= blocks.len() {
                    return None;
                }

                *current = blocks[*block_idx];
            },
        }
    }
}

impl std::ops::BitOr for NewBitSet {
    type Output = NewBitSet;

    fn bitor(self, rhs: NewBitSet) -> NewBitSet {
        match (self, rhs) {
            (NewBitSet::Small(a), NewBitSet::Small(b)) => NewBitSet::Small(a | b),
            (NewBitSet::Medium(a), NewBitSet::Medium(b)) => NewBitSet::Medium(a | b),
            (NewBitSet::Large(mut a), NewBitSet::Large(b)) => {
                for (x, y) in a.iter_mut().zip(b) {
                    *x |= y;
                }
                NewBitSet::Large(a)
            }
            _ => panic!("BitSet variants/capacities differ"),
        }
    }
}

impl std::ops::BitOrAssign for NewBitSet {
    fn bitor_assign(&mut self, rhs: NewBitSet) {
        match (self, rhs) {
            (NewBitSet::Small(a), NewBitSet::Small(b)) => *a |= b,
            (NewBitSet::Medium(a), NewBitSet::Medium(b)) => *a |= b,
            (NewBitSet::Large(a), NewBitSet::Large(b)) => {
                for (x, y) in a.iter_mut().zip(b) {
                    *x |= y;
                }
            }
            _ => panic!("BitSet variants/capacities differ"),
        }
    }
}

impl std::ops::BitAnd for NewBitSet {
    type Output = NewBitSet;

    fn bitand(self, rhs: NewBitSet) -> NewBitSet {
        match (self, rhs) {
            (NewBitSet::Small(a), NewBitSet::Small(b)) => NewBitSet::Small(a & b),
            (NewBitSet::Medium(a), NewBitSet::Medium(b)) => NewBitSet::Medium(a & b),
            (NewBitSet::Large(mut a), NewBitSet::Large(b)) => {
                for (x, y) in a.iter_mut().zip(b) {
                    *x &= y;
                }
                NewBitSet::Large(a)
            }
            _ => panic!("BitSet variants/capacities differ"),
        }
    }
}
