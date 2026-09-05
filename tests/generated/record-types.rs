#![allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record(pub protos::Text, pub protos::Integer);
impl datom_codec::Datomic for Record {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 2)?;
        let p0: protos::Text = datom_codec::Positional::position(&mut p)?;
        let p1: protos::Integer = datom_codec::Positional::position(&mut p)?;
        Ok(Self(p0, p1))
    }
}
impl protos::Conceivable<datom_codec::Datom> for Record {
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
