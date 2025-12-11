
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

    if *first_byte > 126 {
        return Err(Error::UnsupportedLargeGraph);

    } else if *first_byte < 63 {
        return Err(Error::InvalidFormat);
    }

    Ok((first_byte - 63) as usize)
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
