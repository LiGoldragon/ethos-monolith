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
}
impl<A: Sized + datom_codec::Datomic> protos::Conceivable<datom_codec::Datom>
for Placed<A> {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                datom_codec::Datom::Struct(
                    vec![
                        protos::Conceivable::conceive(& self.0)
                        .expect("infallible datom ascent").1,
                        protos::Conceivable::conceive(& self.1)
                        .expect("infallible datom ascent").1
                    ],
                ),
            ),
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
}
impl protos::Conceivable<datom_codec::Datom> for Score {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                datom_codec::Datom::Struct(
                    vec![
                        protos::Conceivable::conceive(& self.0)
                        .expect("infallible datom ascent").1,
                        protos::Conceivable::conceive(& self.1)
                        .expect("infallible datom ascent").1,
                        protos::Conceivable::conceive(& self.2)
                        .expect("infallible datom ascent").1
                    ],
                ),
            ),
        )
    }
}
