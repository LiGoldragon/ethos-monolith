//! Canonicalization: sweet Text to canonical Text (the mechanical conversion).
//!
//! An ethos file is written in the sweet form: the root's head, then the
//! sections as siblings. The reader sees only the braced form. The
//! conversion is mechanical and structural: the sweet text is
//! delineated, and when its first structure is a bare head, `.{` is
//! inserted right after it and a closing brace appended on its own
//! line. A text already in the braced form is left as it is.

use protos::{Extent, Head, Protoform, Protosizable, Situated};

use crate::{Canonical, Canonicalizable, Resituating};

impl Canonicalizable for String {
    fn canonicalize(&self) -> Result<Canonical, protos::Fault> {
        let delineation = <str as Protosizable>::protosize(self)?;
        if let Some(Situated(situation, Protoform::Bare(Head::Symbol(_)))) = delineation.0.first() {
            let Extent(_, end) = situation.extent;
            let end = end as usize;
            let mut text = String::with_capacity(self.len() + 4);
            text.push_str(&self[..end]);
            text.push_str(".{");
            text.push_str(&self[end..]);
            text.push_str("\n}");
            return Ok(Canonical {
                text,
                seam: Extent(end as protos::Integer, end as protos::Integer + 2),
            });
        }
        Ok(Canonical {
            text: self.clone(),
            seam: Extent(0, 0),
        })
    }
}

/// The kind whose capability maps one position across the seam.
trait Shifting {
    fn shift(&self, position: protos::Integer) -> protos::Integer;
}

impl Shifting for Canonical {
    fn shift(&self, position: protos::Integer) -> protos::Integer {
        let Extent(start, end) = self.seam;
        let inserted = end - start;
        let source_end = self.text.len() as protos::Integer - 2 * inserted;
        if position <= start {
            position
        } else if position < end {
            start
        } else {
            (position - inserted).min(source_end)
        }
    }
}

impl Resituating for Canonical {
    fn resituate(&self, extent: Extent) -> Extent {
        Extent(self.shift(extent.0), self.shift(extent.1))
    }
}
