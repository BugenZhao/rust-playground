#[derive(Debug)]
enum Datum {
    String(String),
    Array(Vec<u8>),
}

impl Datum {
    fn to_ref(&self) -> DatumRef {
        match self {
            Datum::String(s) => DatumRef::String(s.as_str()),
            Datum::Array(a) => DatumRef::Array(a.as_slice()),
        }
    }
}

#[derive(Clone, Debug)]
enum DatumRef<'a> {
    String(&'a str),
    Array(&'a [u8]),
}

type Column = Vec<Datum>; // fake

struct Chunk {
    columns: Vec<Column>,
}

impl Chunk {
    fn row_at(&self, index: usize) -> ChunkRowRef {
        ChunkRowRef { chunk: self, index }
    }
}

trait Row {
    /// Randomly access the datum ref in `index`.
    fn datum_ref_at(&self, index: usize) -> DatumRef;

    /// Length.
    fn len(&self) -> usize;
}

pub struct ChunkRowRef<'a> {
    chunk: &'a Chunk,
    index: usize,
}

impl<'a> Row for ChunkRowRef<'a> {
    fn datum_ref_at(&self, index: usize) -> DatumRef {
        self.chunk.columns[index][self.index].to_ref()
    }
    fn len(&self) -> usize {
        self.chunk.columns.len()
    }
}

pub struct VecDatum(Vec<Datum>);

impl<'a> Row for &'a VecDatum {
    fn datum_ref_at(&self, index: usize) -> DatumRef {
        self.0[index].to_ref()
    }
    fn len(&self) -> usize {
        self.0.len()
    }
}

pub struct VecDatumRef<'a>(Vec<DatumRef<'a>>);

impl<'a> Row for VecDatumRef<'a> {
    fn datum_ref_at(&self, index: usize) -> DatumRef {
        self.0[index].clone()
    }
    fn len(&self) -> usize {
        self.0.len()
    }
}

pub struct ConcatRow<R1, R2>(R1, R2);

impl<R1: Row, R2: Row> Row for ConcatRow<R1, R2> {
    fn datum_ref_at(&self, index: usize) -> DatumRef {
        if index < self.0.len() {
            self.0.datum_ref_at(index)
        } else {
            self.1.datum_ref_at(index - self.0.len())
        }
    }
    fn len(&self) -> usize {
        self.0.len() + self.1.len()
    }
}

pub struct MapRow<'a, R> {
    row: R,
    indices: &'a [usize],
}

impl<'a, R: Row> Row for MapRow<'a, R> {
    fn datum_ref_at(&self, index: usize) -> DatumRef {
        self.row.datum_ref_at(self.indices[index])
    }
    fn len(&self) -> usize {
        self.indices.len()
    }
}

trait RowExt: Row + Sized {
    fn concat<R>(self, other: R) -> ConcatRow<Self, R>
    where
        R: Row,
    {
        ConcatRow(self, other)
    }

    fn map<'a>(self, indices: &'a [usize]) -> MapRow<'a, Self> {
        MapRow { row: self, indices }
    }
}

impl<R: Row> RowExt for R {}

fn print_row(row: impl Row) {
    for i in 0..row.len() {
        let datum = row.datum_ref_at(i);
        println!("{:?}", datum);
    }
    println!();
}

fn main() {
    let r1 = VecDatum(vec![
        Datum::String("hello".to_string()),
        Datum::Array(vec![1, 2, 3]),
    ]);
    let r2 = || VecDatumRef(vec![DatumRef::String("world"), DatumRef::Array(&[4, 5, 6])]);

    let chunk = Chunk {
        columns: vec![vec![
            Datum::String("rising".to_string()),
            Datum::String("wave".to_string()),
        ]],
    };
    let r3 = || chunk.row_at(1);

    print_row(&r1);
    print_row((&r1).concat(r2()));
    print_row((&r1).map(&[0]).concat(r2()));
    print_row((&r1).map(&[0]).concat(r2()).concat(r3().map(&[0])));
}
