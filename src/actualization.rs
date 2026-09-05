//! Actualization: Potential to File (the whole descent, its fault situated).
//!
//! A potential file is text that may become a File. Actualizing it
//! canonicalizes, delineates and conceives; whichever pass faults, the
//! fault is returned with the extent of the structure at fault in the
//! source text, the canonical seam mapped back out.

use protos::{
    Actualizable, Conceivable, Pathed, Potential, Protosizable, Situated, Situating, Text, Texted,
};

use crate::{Canonicalizable, Fault, File, Resituating};

impl Actualizable<File> for Potential<File> {
    type Fault = Situated<Fault>;

    fn actualize(&self) -> Result<File, Situated<Fault>> {
        let canonical = match self.text().to_owned().canonicalize() {
            Ok(canonical) => canonical,
            Err(fault) => return Err(Situated(Some(fault.extent), Fault::Structural(fault))),
        };
        let delineation = match <Text as Protosizable>::protosize(&canonical.text) {
            Ok(delineation) => delineation,
            Err(fault) => {
                let extent = canonical.resituate(fault.extent);
                return Err(Situated(Some(extent), Fault::Structural(fault)));
            }
        };
        match Conceivable::<File>::conceive(&delineation) {
            Ok(file) => Ok(file),
            Err(fault) => {
                let extent = match delineation.situate(fault.path()) {
                    Some(extent) => Some(canonical.resituate(extent)),
                    None => None,
                };
                Err(Situated(extent, fault))
            }
        }
    }
}
