#![allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placed<A: Sized>(pub Option<protos::Integer>, pub A);
impl<A: Sized + datom_codec::Datomic> datom_codec::Datomic for Placed<A> {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 2)?;
        let p0: Option<protos::Integer> = datom_codec::Positional::position(&mut p)?;
        let p1: A = datom_codec::Positional::position(&mut p)?;
        Ok(Self(p0, p1))
    }
    fn conceive(&self) -> datom_codec::Datom {
        datom_codec::Datom::Struct(
            vec![
                datom_codec::Datomic::conceive(& self.0),
                datom_codec::Datomic::conceive(& self.1)
            ],
        )
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Score(pub protos::Decimal, pub protos::Boolean, pub datom_codec::Meaning);
impl datom_codec::Datomic for Score {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 3)?;
        let p0: protos::Decimal = datom_codec::Positional::position(&mut p)?;
        let p1: protos::Boolean = datom_codec::Positional::position(&mut p)?;
        let p2: datom_codec::Meaning = datom_codec::Positional::position(&mut p)?;
        Ok(Self(p0, p1, p2))
    }
    fn conceive(&self) -> datom_codec::Datom {
        datom_codec::Datom::Struct(
            vec![
                datom_codec::Datomic::conceive(& self.0),
                datom_codec::Datomic::conceive(& self.1),
                datom_codec::Datomic::conceive(& self.2)
            ],
        )
    }
}
