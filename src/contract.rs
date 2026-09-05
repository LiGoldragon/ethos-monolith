#![allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Generation(pub protos::Text, pub protos::Text);
impl datom_codec::Datomic for Generation {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 2)?;
        let p0: protos::Text = datom_codec::Positional::position(&mut p)?;
        let p1: protos::Text = datom_codec::Positional::position(&mut p)?;
        std::result::Result::Ok(Self(p0, p1))
    }
}
impl protos::Conceivable<datom_codec::Datom> for Generation {
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
                        .expect("infallible datom ascent").1,
                        protos::Conceivable::conceive(& self.1)
                        .expect("infallible datom ascent").1
                    ],
                ),
            ),
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Generate(Generation),
}
impl datom_codec::Datomic for Request {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Generate" => {
                std::result::Result::Ok(Self::Generate(datom_codec::Carrying::body(v)?))
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
impl protos::Conceivable<datom_codec::Datom> for Request {
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
                    Self::Generate(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Generate")
                                .expect("static variant"),
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
pub enum Response {
    Generated(std::vec::Vec<protos::Text>),
    Arguments(protos::Integer),
    Malformed(datom_codec::Fault),
    Unreadable(protos::Text, protos::Text),
    Faulty(protos::Text, datom_codec::Extent, ethos_zero::Fault),
    Unwritable(protos::Text, protos::Text),
}
impl datom_codec::Datomic for Response {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Generated" => {
                std::result::Result::Ok(Self::Generated(datom_codec::Carrying::body(v)?))
            }
            "Arguments" => {
                std::result::Result::Ok(Self::Arguments(datom_codec::Carrying::body(v)?))
            }
            "Malformed" => {
                std::result::Result::Ok(Self::Malformed(datom_codec::Carrying::body(v)?))
            }
            "Unreadable" => {
                let mut p = datom_codec::Headed::positions(v, 2)?;
                let p0: protos::Text = datom_codec::Positional::position(&mut p)?;
                let p1: protos::Text = datom_codec::Positional::position(&mut p)?;
                std::result::Result::Ok(Self::Unreadable(p0, p1))
            }
            "Faulty" => {
                let mut p = datom_codec::Headed::positions(v, 3)?;
                let p0: protos::Text = datom_codec::Positional::position(&mut p)?;
                let p1: datom_codec::Extent = datom_codec::Positional::position(&mut p)?;
                let p2: ethos_zero::Fault = datom_codec::Positional::position(&mut p)?;
                std::result::Result::Ok(Self::Faulty(p0, p1, p2))
            }
            "Unwritable" => {
                let mut p = datom_codec::Headed::positions(v, 2)?;
                let p0: protos::Text = datom_codec::Positional::position(&mut p)?;
                let p1: protos::Text = datom_codec::Positional::position(&mut p)?;
                std::result::Result::Ok(Self::Unwritable(p0, p1))
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
impl protos::Conceivable<datom_codec::Datom> for Response {
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
                    Self::Generated(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Generated")
                                .expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Arguments(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Arguments")
                                .expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Malformed(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Malformed")
                                .expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Unreadable(p0, p1) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Unreadable")
                                .expect("static variant"),
                            std::boxed::Box::new(
                                datom_codec::Datom::Struct(
                                    vec![
                                        protos::Conceivable::conceive(p0)
                                        .expect("infallible datom ascent").1,
                                        protos::Conceivable::conceive(p1)
                                        .expect("infallible datom ascent").1
                                    ],
                                ),
                            ),
                        )
                    }
                    Self::Faulty(p0, p1, p2) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Faulty").expect("static variant"),
                            std::boxed::Box::new(
                                datom_codec::Datom::Struct(
                                    vec![
                                        protos::Conceivable::conceive(p0)
                                        .expect("infallible datom ascent").1,
                                        protos::Conceivable::conceive(p1)
                                        .expect("infallible datom ascent").1,
                                        protos::Conceivable::conceive(p2)
                                        .expect("infallible datom ascent").1
                                    ],
                                ),
                            ),
                        )
                    }
                    Self::Unwritable(p0, p1) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Unwritable")
                                .expect("static variant"),
                            std::boxed::Box::new(
                                datom_codec::Datom::Struct(
                                    vec![
                                        protos::Conceivable::conceive(p0)
                                        .expect("infallible datom ascent").1,
                                        protos::Conceivable::conceive(p1)
                                        .expect("infallible datom ascent").1
                                    ],
                                ),
                            ),
                        )
                    }
                },
            ),
        )
    }
}
