#![allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sink(pub protos::Text, pub Vec<protos::Text>);
impl datom_codec::Datomic for Sink {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 2)?;
        let p0: protos::Text = datom_codec::Positional::position(&mut p)?;
        let p1: Vec<protos::Text> = datom_codec::Positional::position(&mut p)?;
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SinkError {
    Closed,
    Full,
}
impl datom_codec::Datomic for SinkError {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Closed" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Closed)
            }
            "Full" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Full)
            }
            _ => {
                Err(
                    datom_codec::Sited::refuse(
                        site,
                        datom_codec::Problem::UnknownVariant(v.name.to_owned()),
                    ),
                )
            }
        }
    }
    fn conceive(&self) -> datom_codec::Datom {
        match self {
            Self::Closed => datom_codec::Datom::Word("Closed".to_owned()),
            Self::Full => datom_codec::Datom::Word("Full".to_owned()),
        }
    }
}
const _: () = {
    fn assert_sink_summarizable<T: super::Summarizable>() {}
    let _ = assert_sink_summarizable::<Sink>;
    fn assert_sink_fillable<T: super::Fillable>() {}
    let _ = assert_sink_fillable::<Sink>;
};
