/*!
Representations for the space usage of various parts of a Tantivy index.

This can be used programatically, and will also be exposed in a human readable fashion in
tantivy-cli.
*/

use schema::Field;
use std::collections::HashMap;
use std::ops::{Add, AddAssign};

/// Indicates space usage in bytes
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

/// Represents combined space usage of an entire searcher and its component segments.
#[derive(Clone, Debug)]
pub struct SearcherSpaceUsage {
    segments: Vec<SegmentSpaceUsage>,
    total: ByteCount,
}

impl SearcherSpaceUsage {
    pub(crate) fn new() -> SearcherSpaceUsage {
        SearcherSpaceUsage {
            segments: Vec::new(),
            total: ByteCount(0),
        }
    }

    /// Add a segment, to `self`.
    /// Performs no deduplication or other intelligence.
    pub(crate) fn add_segment(&mut self, segment: SegmentSpaceUsage) {
        self.total += segment.total();
        self.segments.push(segment);
    }

    /// Returns total byte usage of this searcher, including all large subcomponents.
    /// Does not account for smaller things like `meta.json`.
    pub fn total(&self) -> ByteCount {
        self.total
    }
}

/// Represents combined space usage for all of the large components comprising a segment.
#[derive(Clone, Debug)]
pub struct SegmentSpaceUsage {
    num_docs: u32,

    termdict: PerFieldSpaceUsage,
    postings: PerFieldSpaceUsage,
    positions: PerFieldSpaceUsage,
    fast_fields: PerFieldSpaceUsage,
    fieldnorms: PerFieldSpaceUsage,

    store: StoreSpaceUsage,

    deletes: ByteCount,

    total: ByteCount,
}

impl SegmentSpaceUsage {
    pub(crate) fn new(
        num_docs: u32,
        termdict: PerFieldSpaceUsage,
        postings: PerFieldSpaceUsage,
        positions: PerFieldSpaceUsage,
        fast_fields: PerFieldSpaceUsage,
        fieldnorms: PerFieldSpaceUsage,
        store: StoreSpaceUsage,
        deletes: ByteCount,
    ) -> SegmentSpaceUsage {
        let total = termdict.total()
            + postings.total()
            + positions.total()
            + fast_fields.total()
            + fieldnorms.total()
            + store.total()
            + deletes;
        SegmentSpaceUsage {
            num_docs,
            termdict,
            postings,
            positions,
            fast_fields,
            fieldnorms,
            store,
            deletes,
            total,
        }
    }

    /// Total space usage in bytes for this segment.
    pub fn total(&self) -> ByteCount {
        self.total
    }
}

/// Represents space usage for the Store for this segment.
///
/// This is composed of two parts.
/// `data` represents the compressed data itself.
/// `offsets` represents a lookup to find the start of a block
#[derive(Clone, Debug)]
pub struct StoreSpaceUsage {
    data: ByteCount,
    offsets: ByteCount,
}

impl StoreSpaceUsage {
    pub(crate) fn new(data: ByteCount, offsets: ByteCount) -> StoreSpaceUsage {
        StoreSpaceUsage { data, offsets }
    }

    /// Total space usage in bytes for this Store
    pub fn total(&self) -> ByteCount {
        self.data + self.offsets
    }
}

/// Represents space usage for all of the (field, index) pairs that appear in a CompositeFile.
///
/// A field can appear with a single index (typically 0) or with multiple indexes.
/// Multiple indexes are used to handle variable length things, where
#[derive(Clone, Debug)]
pub struct PerFieldSpaceUsage {
    fields: HashMap<Field, FieldUsage>,
    total: ByteCount
}

impl PerFieldSpaceUsage {
    pub(crate) fn new(fields: HashMap<Field, FieldUsage>) -> PerFieldSpaceUsage {
        let total = fields.values().map(|x| x.total()).fold(ByteCount(0), Add::add);
        PerFieldSpaceUsage { fields, total }
    }

    /// Bytes used by the represented file
    pub fn total(&self) -> ByteCount {
        self.total
    }
}

/// Represents space usage of a given field, breaking it down into the (field, index) pairs that
/// comprise it.
///
/// See documentation for PerFieldSpaceUsage for slightly more information.
#[derive(Clone, Debug)]
pub struct FieldUsage {
    field: Field,
    weight: ByteCount,
    /// A field can be composed of more than one piece.
    /// These pieces are indexed by arbitrary numbers starting at zero.
    /// `self.weight` includes all of `self.sub_weights`.
    sub_weights: Vec<Option<ByteCount>>,
}

impl FieldUsage {
    pub(crate) fn empty(field: Field) -> FieldUsage {
        FieldUsage {
            field,
            weight: ByteCount(0),
            sub_weights: Vec::new(),
        }
    }

    pub(crate) fn add_field_idx(&mut self, idx: usize, size: ByteCount) {
        if self.sub_weights.len() < idx {
            self.sub_weights.resize(idx, None);
        }
        assert!(self.sub_weights[idx].is_none());
        self.sub_weights[idx] = Some(size);
        self.weight += size
    }

    /// Total bytes used for this field in this context
    pub fn total(&self) -> ByteCount {
        self.weight
    }
}