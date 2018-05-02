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

pub struct SearcherSpaceUsage {
    segments: Vec<SegmentSpaceUsage>,
}

impl SearcherSpaceUsage {
    pub fn new(segments: Vec<SegmentSpaceUsage>) -> SearcherSpaceUsage {
        SearcherSpaceUsage { segments }
    }
}

pub struct SegmentSpaceUsage {
    pub(crate) num_docs: u32,

    pub(crate) termdict: PerFieldSpaceUsage,
    pub(crate) postings: PerFieldSpaceUsage,
    pub(crate) positions: PerFieldSpaceUsage,
    pub(crate) fast_fields: PerFieldSpaceUsage,
    pub(crate) fieldnorms: PerFieldSpaceUsage,

    pub(crate) store: StoreSpaceUsage,

    pub(crate) deletes: ByteCount,
}

pub struct StoreSpaceUsage {
    data: ByteCount,
    offsets: ByteCount,
}

impl StoreSpaceUsage {
    pub fn new(data: ByteCount, offsets: ByteCount) -> StoreSpaceUsage {
        StoreSpaceUsage { data, offsets }
    }
}

pub struct PerFieldSpaceUsage {
    fields: HashMap<Field, FieldUsage>,
    total: ByteCount
}

impl PerFieldSpaceUsage {
    pub fn new(fields: HashMap<Field, FieldUsage>, total: ByteCount) -> PerFieldSpaceUsage {
        PerFieldSpaceUsage { fields, total }
    }
}

pub struct FieldUsage {
    field: Field,
    weight: ByteCount,
    /// A field can be composed of more than one piece.
    /// These pieces are indexed by arbitrary numbers starting at zero.
    /// `self.weight` includes all of `self.sub_weights`.
    sub_weights: Vec<Option<ByteCount>>,
}

impl FieldUsage {
    pub fn empty(field: Field) -> FieldUsage {
        FieldUsage {
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