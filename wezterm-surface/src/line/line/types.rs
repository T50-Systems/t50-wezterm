use crate::cellcluster::CellCluster;
use crate::hyperlink::Rule;
use crate::line::cellref::CellRef;
use crate::line::clusterline::ClusteredLine;
use crate::line::linebits::LineBits;
use crate::line::storage::{CellStorage, VisibleCellIter};
use crate::line::vecstorage::{VecStorage, VecStorageIter};
use crate::{Change, SequenceNo, SEQ_ZERO};
use alloc::borrow::Cow;
#[cfg(feature = "appdata")]
use alloc::sync::{Arc, Weak};
#[cfg(feature = "appdata")]
use core::any::Any;
use core::hash::Hash;
use core::ops::Range;
use finl_unicode::grapheme_clusters::Graphemes;
#[cfg(feature = "use_serde")]
use serde::{Deserialize, Serialize};
use siphasher::sip128::{Hasher128, SipHasher};
#[cfg(feature = "appdata")]
use std::sync::Mutex;
use wezterm_bidi::{Direction, ParagraphDirectionHint};
use wezterm_cell::{Cell, CellAttributes, SemanticType, UnicodeVersion};

extern crate alloc;
use crate::alloc::string::ToString;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneRange {
    pub semantic_type: SemanticType,
    pub range: Range<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DoubleClickRange {
    Range(Range<usize>),
    RangeWithWrap(Range<usize>),
}

#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct Line {
    pub(crate) cells: CellStorage,
    zones: Vec<ZoneRange>,
    seqno: SequenceNo,
    bits: LineBits,
    #[cfg(feature = "appdata")]
    #[cfg_attr(feature = "use_serde", serde(skip))]
    appdata: Mutex<Option<Weak<dyn Any + Send + Sync>>>,
}

impl Clone for Line {
    fn clone(&self) -> Self {
        Self {
            cells: self.cells.clone(),
            zones: self.zones.clone(),
            seqno: self.seqno,
            bits: self.bits,
            #[cfg(feature = "appdata")]
            appdata: Mutex::new(self.appdata.lock().unwrap().clone()),
        }
    }
}

impl PartialEq for Line {
    fn eq(&self, other: &Self) -> bool {
        self.seqno == other.seqno && self.bits == other.bits && self.cells == other.cells
    }
}
