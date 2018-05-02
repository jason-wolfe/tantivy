use schema::Field;
use std::collections::HashMap;
use std::ops::{Add, AddAssign};

#[derive(Clone, Copy, Debug)]
pub struct ByteCount(pub usize);

impl Add for ByteCount {
    type Output = ByteCount;
    fn add(self, rhs: ByteCount) -> ByteCount {
        ByteCount(self.0 + rhs.0)
    }
}

impl AddAssign for ByteCount {
    fn add_assign(&mut self, rhs: ByteCount) {
        self.0 += rhs.0;
    }
}

pub struct SearcherWeight {
    segments: Vec<SegmentWeight>,
}

impl SearcherWeight {
    pub fn new(segments: Vec<SegmentWeight>) -> SearcherWeight {
        SearcherWeight { segments }
    }
}

pub struct SegmentWeight {
    pub(crate) num_docs: u32,

    pub(crate) termdict: PerFieldWeight,
    pub(crate) postings: PerFieldWeight,
    pub(crate) positions: PerFieldWeight,
    pub(crate) fast_fields: PerFieldWeight,
    pub(crate) fieldnorms: PerFieldWeight,

    pub(crate) store: StoreWeight,

    pub(crate) deletes: ByteCount,
}

pub struct StoreWeight {
    data: ByteCount,
    offsets: ByteCount,
}

impl StoreWeight {
    pub fn new(data: ByteCount, offsets: ByteCount) -> StoreWeight {
        StoreWeight { data, offsets }
    }
}

pub struct PerFieldWeight {
    fields: HashMap<Field, FieldWeight>,
    total: ByteCount
}

impl PerFieldWeight {
    pub fn new(fields: HashMap<Field, FieldWeight>, total: ByteCount) -> PerFieldWeight {
        PerFieldWeight { fields, total }
    }
}

pub struct FieldWeight {
    field: Field,
    weight: ByteCount,
    /// A field can be composed of more than one piece.
    /// These pieces are indexed by arbitrary numbers starting at zero.
    /// `self.weight` includes all of `self.sub_weights`.
    sub_weights: Vec<Option<ByteCount>>,
}

impl FieldWeight {
    pub fn empty(field: Field) -> FieldWeight {
        FieldWeight {
            field,
            weight: ByteCount(0),
            sub_weights: Vec::new(),
        }
    }

    pub fn add_field_idx(&mut self, idx: usize, size: ByteCount) {
        if self.sub_weights.len() < idx {
            self.sub_weights.resize(idx, None);
        }
        assert!(self.sub_weights[idx].is_none());
        self.sub_weights[idx] = Some(size);
        self.weight += size
    }
}