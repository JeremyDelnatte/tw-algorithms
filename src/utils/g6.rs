
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("empty input")]
    Empty,

    #[error("unsupported graph6 format for large graphs")]
    UnsupportedLargeGraph,

    #[error("invalid graph6 format")]
    InvalidFormat,
}


pub fn get_size(bytes: &[u8]) -> Result<usize, Error> {
    let Some(first_byte) = bytes.get(0) else {
        return Err(Error::Empty);
    };

    Ok(match first_byte {
        63..=125 => {
            (first_byte - 63) as usize
        },
        126 => {
            if bytes.len() < 4 {
                    return Err(Error::InvalidFormat);
            }
            ((bytes[1] as usize - 63) << 12)
                | ((bytes[2] as usize - 63) << 6)
                | (bytes[3] as usize - 63)
        },
        _ => return Err(Error::InvalidFormat),
    })
}

pub fn to_size(n: usize) -> Vec<u8> {
    match n {
        0..=62 => vec![n as u8 + 63],
        63..=258047 => vec![
            126,
            ((n >> 12) & 0x3F) as u8 + 63,
            ((n >> 6) & 0x3F) as u8 + 63,
            (n & 0x3F) as u8 + 63,
        ],
        _ => panic!("unsupported graph size for graph6 format"),
    }
}

pub fn get_edges(bytes: &[u8], n: usize) -> Result<Vec<(usize, usize)>, Error> {
    let mut edges = Vec::new();
    let mut row = 0;
    let mut col = 1;

    for b in bytes {
        let Some(b) = b.checked_sub(63) else {
            return Err(Error::InvalidFormat);
        };

        for i in 0..6 {
            let bit = (b >> (5 - i)) & 1;

            if bit == 1 {
                edges.push((row, col));
            }

            if row == col - 1 {
                row = 0;
                col += 1;

            } else {
                row += 1;
            }

            if col >= n {
                break;
            }
        }
    }

    Ok(edges)
}

pub fn to_edges(edges: &[(usize, usize)], n: usize, buf: &mut Vec<u8>) {
    let start_index = buf.len();

    for &(row, col) in edges {
        if row >= n || col >= n || row >= col {
            panic!("invalid edge ({}, {}) for graph of size {}", row, col, n);
        }

        let edge_index = (col * (col - 1)) / 2 + row;
        let byte_index = edge_index / 6 + start_index;
        let bit_position = edge_index % 6;

        while byte_index >= buf.len() {
            buf.push(0);
        }

        buf[byte_index] |= 1 << (5 - bit_position);
    }

    for byte in &mut buf[start_index..] {
        *byte += 63;
    }
}
