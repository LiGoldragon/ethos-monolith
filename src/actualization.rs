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

/// The kind whose capability locates a fault extent in a situated protoform,
/// translating the ethos path convention (headed body = child 0) to the protos
/// convention (headed body = child 1).
trait Faulted {
    fn fault_extent(&self, path: &[Integer]) -> Extent;
}

impl Faulted for Situated<Protoform> {
    fn fault_extent(&self, path: &[Integer]) -> Extent {
        let Situated(situation, form) = self;
        let mut here_sit: &Situation = situation;
        let mut here_form: &Protoform = form;
        for &index in path {
            match here_form {
                Protoform::Headed(_, _, body) if index == 0 => {
                    here_sit = here_sit.part(1);
                    here_form = body;
                }
                Protoform::Enclosed(_, children) => {
                    here_sit = here_sit.part(index);
                    if let Some(child) = children.get(index as usize) {
                        here_form = child;
                    } else {
                        return here_sit.extent;
                    }
                }
                _ => return here_sit.extent,
            }
        }
        here_sit.extent
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
