#![allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    Structural(protos::Fault),
    Conceptual(std::vec::Vec<protos::Integer>, Problem),
}
impl datom_codec::Datomic for Fault {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Structural" => {
                std::result::Result::Ok(
                    Self::Structural(datom_codec::Carrying::body(v)?),
                )
            }
            "Conceptual" => {
                let mut p = datom_codec::Headed::positions(v, 2)?;
                let p0: std::vec::Vec<protos::Integer> = datom_codec::Positional::position(
                    &mut p,
                )?;
                let p1: Problem = datom_codec::Positional::position(&mut p)?;
                std::result::Result::Ok(Self::Conceptual(p0, p1))
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
impl protos::Conceivable<datom_codec::Datom> for Fault {
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
                    Self::Structural(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Structural")
                                .expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Conceptual(p0, p1) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Conceptual")
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    Root,
    Arity(protos::Integer, protos::Integer),
    Expected(Form),
    Separator(protos::Separator),
    Name(protos::Text),
    Duplicate(protos::Text),
    Undeclared(protos::Text),
    Cycle(protos::Text),
    Yield,
    Empty,
    Depth,
    Role(protos::Text),
}
impl datom_codec::Datomic for Problem {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Root" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::Root)
            }
            "Arity" => {
                let mut p = datom_codec::Headed::positions(v, 2)?;
                let p0: protos::Integer = datom_codec::Positional::position(&mut p)?;
                let p1: protos::Integer = datom_codec::Positional::position(&mut p)?;
                std::result::Result::Ok(Self::Arity(p0, p1))
            }
            "Expected" => {
                std::result::Result::Ok(Self::Expected(datom_codec::Carrying::body(v)?))
            }
            "Separator" => {
                std::result::Result::Ok(Self::Separator(datom_codec::Carrying::body(v)?))
            }
            "Name" => {
                std::result::Result::Ok(Self::Name(datom_codec::Carrying::body(v)?))
            }
            "Duplicate" => {
                std::result::Result::Ok(Self::Duplicate(datom_codec::Carrying::body(v)?))
            }
            "Undeclared" => {
                std::result::Result::Ok(
                    Self::Undeclared(datom_codec::Carrying::body(v)?),
                )
            }
            "Cycle" => {
                std::result::Result::Ok(Self::Cycle(datom_codec::Carrying::body(v)?))
            }
            "Yield" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::Yield)
            }
            "Empty" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::Empty)
            }
            "Depth" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::Depth)
            }
            "Role" => {
                std::result::Result::Ok(Self::Role(datom_codec::Carrying::body(v)?))
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
impl protos::Conceivable<datom_codec::Datom> for Problem {
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
                    Self::Root => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("Root").expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::Arity(p0, p1) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Arity").expect("static variant"),
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
                    Self::Expected(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Expected")
                                .expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Separator(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Separator")
                                .expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Name(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Name").expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Duplicate(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Duplicate")
                                .expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Undeclared(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Undeclared")
                                .expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Cycle(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Cycle").expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Yield => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("Yield").expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::Empty => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("Empty").expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::Depth => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("Depth").expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::Role(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Role").expect("static variant"),
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    File,
    Section,
    Import,
    Name,
    Declaration,
    Variant,
    Reference,
    Constraint,
    Kind,
    Capability,
    Constant,
    Association,
}
impl datom_codec::Datomic for Form {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "File" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::File)
            }
            "Section" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::Section)
            }
            "Import" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::Import)
            }
            "Name" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::Name)
            }
            "Declaration" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::Declaration)
            }
            "Variant" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::Variant)
            }
            "Reference" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::Reference)
            }
            "Constraint" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::Constraint)
            }
            "Kind" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::Kind)
            }
            "Capability" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::Capability)
            }
            "Constant" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::Constant)
            }
            "Association" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::Association)
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
impl protos::Conceivable<datom_codec::Datom> for Form {
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
                    Self::File => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("File").expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::Section => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("Section").expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::Import => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("Import").expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::Name => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("Name").expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::Declaration => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("Declaration")
                                        .expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::Variant => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("Variant").expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::Reference => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("Reference").expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::Constraint => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("Constraint")
                                        .expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::Kind => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("Kind").expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::Capability => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("Capability")
                                        .expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::Constant => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("Constant").expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::Association => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("Association")
                                        .expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                },
            ),
        )
    }
}
