use bytes::BufMut;
use itertools::Itertools;

struct DataChunk;

impl DataChunk {
    fn hash_codes(&self) -> Vec<u64> {
        vec![1, 2, 3]
    }

    fn size_hints(&self) -> Vec<usize> {
        vec![1, 2, 3]
    }

    fn serialize_into(&self, bufs: &mut [impl Buffer]) {
        let cols = [[1, 2, 3]];
        for col in cols {
            for (row, buf) in col.iter().zip(&mut *bufs) {
                buf.buf_mut().put_i32(*row);
            }
        }
    }
}

struct OwnedRow;

trait Buffer: 'static {
    type BufMut<'a>: BufMut
    where
        Self: 'a;

    type Sealed: AsRef<[u8]> + 'static;

    fn with_capacity(f: impl FnOnce() -> usize) -> Self;

    fn buf_mut(&mut self) -> Self::BufMut<'_>;

    fn seal(self) -> Self::Sealed;
}

struct HashKey<B: Buffer> {
    key: B::Sealed,
    hash_code: u64,
}

impl<B: Buffer> HashKey<B> {
    fn build_from_chunk(chunk: DataChunk) -> Vec<Self> {
        let hash_codes = chunk.hash_codes();
        let mut size_hints = None;

        let mut bufs = (0..hash_codes.len())
            .map(|i| {
                B::with_capacity(|| {
                    let size_hints = size_hints.get_or_insert_with(|| chunk.size_hints());
                    size_hints[i]
                })
            })
            .collect_vec();

        chunk.serialize_into(&mut bufs);

        hash_codes
            .into_iter()
            .zip(bufs)
            .map(|(hash_code, buf)| {
                let key = buf.seal();
                Self { key, hash_code }
            })
            .collect_vec()
    }
}

fn main() {}
