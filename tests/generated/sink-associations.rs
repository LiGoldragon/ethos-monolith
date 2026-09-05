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
}
impl protos::Conceivable<datom_codec::Datom> for Sink {
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
                    datom_codec::Headed::reject(
                        &v,
                        datom_codec::Problem::UnknownVariant(
                            protos::Word::try_from(v.name).expect("variant name"),
                        ),
                    ),
                )
            }
        }
    }
}
impl protos::Conceivable<datom_codec::Datom> for SinkError {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                match self {
                    Self::Closed => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("Closed").expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::Full => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("Full").expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                },
            ),
        )
    }
}
const _: () = {
    fn assert_sink_summarizable<T: super::Summarizable>() {}
    let _ = assert_sink_summarizable::<Sink>;
    fn assert_sink_fillable<T: super::Fillable>() {}
    let _ = assert_sink_fillable::<Sink>;
};
