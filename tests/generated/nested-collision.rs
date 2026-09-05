#![allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XEthosNestedA {
    V,
}
impl datom_codec::Datomic for XEthosNestedA {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "V" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::V)
            }
            _ => {
                std::result::Result::Err(
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
impl protos::Conceivable<datom_codec::Datom> for XEthosNestedA {
    type Fault = std::convert::Infallible;
    fn conceive(
        &self,
    ) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                match self {
                    Self::V => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("V").expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                },
            ),
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum X {
    A(XEthosNestedA),
}
impl datom_codec::Datomic for X {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "A" => std::result::Result::Ok(Self::A(datom_codec::Carrying::body(v)?)),
            _ => {
                std::result::Result::Err(
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
impl protos::Conceivable<datom_codec::Datom> for X {
    type Fault = std::convert::Infallible;
    fn conceive(
        &self,
    ) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                match self {
                    Self::A(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("A").expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                },
            ),
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XA(pub protos::Text);
impl datom_codec::Datomic for XA {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 1)?;
        let p0: protos::Text = datom_codec::Positional::position(&mut p)?;
        std::result::Result::Ok(Self(p0))
    }
}
impl protos::Conceivable<datom_codec::Datom> for XA {
    type Fault = std::convert::Infallible;
    fn conceive(
        &self,
    ) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                datom_codec::Datom::Struct(
                    vec![
                        protos::Conceivable::conceive(& self.0)
                        .expect("infallible datom ascent").1
                    ],
                ),
            ),
        )
    }
}
