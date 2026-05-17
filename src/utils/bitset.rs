use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BitSet {
    Small(u64),
    Medium(u128),
    Large(Vec<u64>),
}

impl Hash for BitSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            BitSet::Small(x) => {
                0u8.hash(state);
                x.hash(state);
            }
            BitSet::Medium(x) => {
                1u8.hash(state);
                x.hash(state);
            }
            BitSet::Large(xs) => {
                2u8.hash(state);
                xs.hash(state);
            }
        }
    }
}

impl BitSet {
    pub fn new(n: usize) -> Self {
        match n {
            0..=64 => BitSet::Small(0),
            65..=128 => BitSet::Medium(0),
            _ => BitSet::Large(vec![0; (n + 63) / 64]),
        }
    }

    pub fn full(n: usize) -> Self {
        match n {
            0 => BitSet::Small(0),
            1..=63 => BitSet::Small((1u64 << n) - 1),
            64 => BitSet::Small(u64::MAX),
            65..=127 => BitSet::Medium((1u128 << n) - 1),
            128 => BitSet::Medium(u128::MAX),
            _ => {
                let num_blocks = (n + 63) / 64;
                let mut blocks = vec![u64::MAX; num_blocks];
                let rem = n % 64;
                if rem != 0 {
                    blocks[num_blocks - 1] = (1u64 << rem) - 1;
                }
                BitSet::Large(blocks)
            }
        }
    }

    pub fn from_u64(bits: u64) -> Self {
        BitSet::Small(bits)
    }

    pub fn from_u128(bits: u128) -> Self {
        BitSet::Medium(bits)
    }

    pub fn from_blocks(blocks: Vec<u64>) -> Self {
        BitSet::Large(blocks)
    }

    pub fn insert(&mut self, pos: usize) {
        match self {
            BitSet::Small(bits) => *bits |= 1u64 << pos,
            BitSet::Medium(bits) => *bits |= 1u128 << pos,
            BitSet::Large(blocks) => blocks[pos / 64] |= 1u64 << (pos % 64),
        }
    }

    pub fn remove(&mut self, pos: usize) {
        match self {
            BitSet::Small(bits) => *bits &= !(1u64 << pos),
            BitSet::Medium(bits) => *bits &= !(1u128 << pos),
            BitSet::Large(blocks) => blocks[pos / 64] &= !(1u64 << (pos % 64)),
        }
    }

    pub fn contains(&self, pos: usize) -> bool {
        match self {
            BitSet::Small(bits) => (*bits & (1u64 << pos)) != 0,
            BitSet::Medium(bits) => (*bits & (1u128 << pos)) != 0,
            BitSet::Large(blocks) => {
                let idx = pos / 64;
                idx < blocks.len() && (blocks[idx] & (1u64 << (pos % 64))) != 0
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            BitSet::Small(bits) => bits.count_ones() as usize,
            BitSet::Medium(bits) => bits.count_ones() as usize,
            BitSet::Large(blocks) => blocks.iter().map(|b| b.count_ones() as usize).sum(),
        }
    }

    pub fn intersection_len(&self, rhs: &BitSet) -> usize {
        match (self, rhs) {
            (BitSet::Small(a), BitSet::Small(b)) => (*a & *b).count_ones() as usize,
            (BitSet::Medium(a), BitSet::Medium(b)) => (*a & *b).count_ones() as usize,
            (BitSet::Large(a), BitSet::Large(b)) => {
                a.iter()
                    .zip(b.iter())
                    .map(|(x, y)| (x & y).count_ones() as usize)
                    .sum()
            }
            _ => panic!("BitSet variants/capacities differ"),
        }
    }

    pub fn difference(&self, rhs: &BitSet) -> BitSet {
        match (self, rhs) {
            (BitSet::Small(a), BitSet::Small(b)) => BitSet::Small(*a & !*b),
            (BitSet::Medium(a), BitSet::Medium(b)) => BitSet::Medium(*a & !*b),
            (BitSet::Large(a), BitSet::Large(b)) => {
                BitSet::Large(a.iter().zip(b.iter()).map(|(x, y)| x & !y).collect())
            }
            _ => panic!("BitSet variants/capacities differ"),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            BitSet::Small(bits) => *bits == 0,
            BitSet::Medium(bits) => *bits == 0,
            BitSet::Large(blocks) => blocks.iter().all(|&b| b == 0),
        }
    }

    pub fn has_one_bit(&self) -> bool {
        self.len() == 1
    }

    pub fn first_bit(&self) -> Option<usize> {
        match self {
            BitSet::Small(bits) => {
                if *bits == 0 { None } else { Some(bits.trailing_zeros() as usize) }
            }
            BitSet::Medium(bits) => {
                if *bits == 0 { None } else { Some(bits.trailing_zeros() as usize) }
            }
            BitSet::Large(blocks) => {
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
            BitSet::Small(bits) => BitSetIter::Small(*bits),
            BitSet::Medium(bits) => BitSetIter::Medium(*bits),
            BitSet::Large(blocks) => BitSetIter::Large {
                blocks,
                block_idx: 0,
                current: blocks.first().copied().unwrap_or(0),
            },
        }
    }

    // This method is used to shift all bits from max to pos to the right by one position, effectively removing the bit at pos and shifting all higher bits down.
    pub fn right_shift_from(&mut self, pos: usize) {
        match self {
            BitSet::Small(bits) => {
                let mask = (1u64 << pos) - 1;
                *bits = (*bits & mask) | ((*bits >> 1) & !mask);
            }
            BitSet::Medium(bits) => {
                let mask = (1u128 << pos) - 1;
                *bits = (*bits & mask) | ((*bits >> 1) & !mask);
            }
            BitSet::Large(blocks) => {
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

impl std::ops::BitOr for BitSet {
    type Output = BitSet;

    fn bitor(self, rhs: BitSet) -> BitSet {
        match (self, rhs) {
            (BitSet::Small(a), BitSet::Small(b)) => BitSet::Small(a | b),
            (BitSet::Medium(a), BitSet::Medium(b)) => BitSet::Medium(a | b),
            (BitSet::Large(mut a), BitSet::Large(b)) => {
                for (x, y) in a.iter_mut().zip(b) {
                    *x |= y;
                }
                BitSet::Large(a)
            }
            _ => panic!("BitSet variants/capacities differ"),
        }
    }
}

impl std::ops::BitOrAssign for BitSet {
    fn bitor_assign(&mut self, rhs: BitSet) {
        match (self, rhs) {
            (BitSet::Small(a), BitSet::Small(b)) => *a |= b,
            (BitSet::Medium(a), BitSet::Medium(b)) => *a |= b,
            (BitSet::Large(a), BitSet::Large(b)) => {
                for (x, y) in a.iter_mut().zip(b) {
                    *x |= y;
                }
            }
            _ => panic!("BitSet variants/capacities differ"),
        }
    }
}

impl std::ops::BitOr<&BitSet> for &BitSet {
    type Output = BitSet;

    fn bitor(self, rhs: &BitSet) -> BitSet {
        match (self, rhs) {
            (BitSet::Small(a), BitSet::Small(b)) => BitSet::Small(*a | *b),
            (BitSet::Medium(a), BitSet::Medium(b)) => BitSet::Medium(*a | *b),
            (BitSet::Large(a), BitSet::Large(b)) => {
                BitSet::Large(a.iter().zip(b.iter()).map(|(x, y)| x | y).collect())
            }
            _ => panic!("BitSet variants/capacities differ"),
        }
    }
}

impl std::ops::BitOrAssign<&BitSet> for BitSet {
    fn bitor_assign(&mut self, rhs: &BitSet) {
        match (self, rhs) {
            (BitSet::Small(a), BitSet::Small(b)) => *a |= *b,
            (BitSet::Medium(a), BitSet::Medium(b)) => *a |= *b,
            (BitSet::Large(a), BitSet::Large(b)) => {
                for (x, y) in a.iter_mut().zip(b.iter()) {
                    *x |= *y;
                }
            }
            _ => panic!("BitSet variants/capacities differ"),
        }
    }
}

impl std::ops::BitAnd for BitSet {
    type Output = BitSet;

    fn bitand(self, rhs: BitSet) -> BitSet {
        match (self, rhs) {
            (BitSet::Small(a), BitSet::Small(b)) => BitSet::Small(a & b),
            (BitSet::Medium(a), BitSet::Medium(b)) => BitSet::Medium(a & b),
            (BitSet::Large(mut a), BitSet::Large(b)) => {
                for (x, y) in a.iter_mut().zip(b) {
                    *x &= y;
                }
                BitSet::Large(a)
            }
            _ => panic!("BitSet variants/capacities differ"),
        }
    }
}
