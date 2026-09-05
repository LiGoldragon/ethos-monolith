//! Actualization: Potential to File (the whole descent, its fault situated).
//!
//! A potential file is text that may become a File. Actualizing it
//! canonicalizes, delineates and conceives; whichever pass faults, the
//! fault is returned with the extent of the structure at fault in the
//! source text, the canonical seam mapped back out.

use protos::{
    Actualizable, Extent, Integer, Locating, Pathed, Potential, Protoform, Protosizable, Situated,
    Situation, Texted,
};

use crate::conception::Conceiving;
use crate::{Canonicalizable, Fault, File, Resituating};

/// The kind whose capability locates a fault extent in a situated protoform.
///
/// Ethos paths are Protos paths: a headed form carries its head at child zero
/// and its body at child one.  A qualified head keeps its arguments below that
/// head.  Conception and checking preserve those paths, so locating needs no
/// dialect-specific repair.
trait Faulted {
    fn fault_extent(&self, path: &[Integer]) -> Extent;
}

impl Faulted for Situated<Protoform> {
    fn fault_extent(&self, path: &[Integer]) -> Extent {
        let Situated(situation, _) = self;
        situation.locate(path).unwrap_or(situation.extent)
    }
}

impl Actualizable<File> for Potential<File> {
    type Fault = Situated<Fault>;
    type Budget = ();

    fn actualize(&self, (): Self::Budget) -> Result<File, Situated<Fault>> {
        let text: String = self.text().to_owned();
        let canonical = match text.canonicalize() {
            Ok(canonical) => canonical,
            Err(fault) => {
                return Err(Situated(
                    Situation {
                        extent: fault.extent,
                        children: vec![],
                    },
                    Fault::Structural(fault),
                ));
            }
        };
        let delineation = match <str as Protosizable>::protosize(&canonical.text) {
            Ok(delineation) => delineation,
            Err(fault) => {
                let extent = canonical.resituate(fault.extent);
                return Err(Situated(
                    Situation {
                        extent,
                        children: vec![],
                    },
                    Fault::Structural(fault),
                ));
            }
        };
        match Conceiving::<File>::conceive(&delineation) {
            Ok(file) => Ok(file),
            Err(fault) => {
                let extent = if let Some(situated) = delineation.0.first() {
                    let path = fault.path();
                    // Skip the first index (delineation element, always 0)
                    let inner = if path.first() == Some(&0) {
                        &path[1..]
                    } else {
                        path
                    };
                    canonical.resituate(situated.fault_extent(inner))
                } else {
                    Extent(0, 0)
                };
                Err(Situated(
                    Situation {
                        extent,
                        children: vec![],
                    },
                    fault,
                ))
            }
        }
    }
}
