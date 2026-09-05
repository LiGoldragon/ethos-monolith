#![allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tree {
    Leaf(protos::Integer),
    Node(std::boxed::Box<Tree>, std::boxed::Box<Tree>),
    Many(std::vec::Vec<Tree>),
    Maybe(std::boxed::Box<std::option::Option<Tree>>),
}
impl datom_codec::Datomic for Tree {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Leaf" => {
                std::result::Result::Ok(Self::Leaf(datom_codec::Carrying::body(v)?))
            }
            "Node" => {
                let mut p = datom_codec::Headed::positions(v, 2)?;
                let p0: std::boxed::Box<Tree> = datom_codec::Positional::position(
                    &mut p,
                )?;
                let p1: std::boxed::Box<Tree> = datom_codec::Positional::position(
                    &mut p,
                )?;
                std::result::Result::Ok(Self::Node(p0, p1))
            }
            "Many" => {
                std::result::Result::Ok(Self::Many(datom_codec::Carrying::body(v)?))
            }
            "Maybe" => {
                std::result::Result::Ok(Self::Maybe(datom_codec::Carrying::body(v)?))
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
impl protos::Conceivable<datom_codec::Datom> for Tree {
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
                    Self::Leaf(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Leaf").expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Node(p0, p1) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Node").expect("static variant"),
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
                    Self::Many(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Many").expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Maybe(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Maybe").expect("static variant"),
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
pub struct Chain(pub protos::Text, pub std::boxed::Box<std::option::Option<Chain>>);
impl datom_codec::Datomic for Chain {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 2)?;
        let p0: protos::Text = datom_codec::Positional::position(&mut p)?;
        let p1: std::boxed::Box<std::option::Option<Chain>> = datom_codec::Positional::position(
            &mut p,
        )?;
        std::result::Result::Ok(Self(p0, p1))
    }
}
impl protos::Conceivable<datom_codec::Datom> for Chain {
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
pub struct Twin(pub std::boxed::Box<Twig>, pub std::boxed::Box<Twig>);
impl datom_codec::Datomic for Twin {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 2)?;
        let p0: std::boxed::Box<Twig> = datom_codec::Positional::position(&mut p)?;
        let p1: std::boxed::Box<Twig> = datom_codec::Positional::position(&mut p)?;
        std::result::Result::Ok(Self(p0, p1))
    }
}
impl protos::Conceivable<datom_codec::Datom> for Twin {
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
pub enum Twig {
    Tip,
    Grow(std::boxed::Box<Twin>),
}
impl datom_codec::Datomic for Twig {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Tip" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::Tip)
            }
            "Grow" => {
                std::result::Result::Ok(Self::Grow(datom_codec::Carrying::body(v)?))
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
impl protos::Conceivable<datom_codec::Datom> for Twig {
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
                    Self::Tip => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("Tip").expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::Grow(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Grow").expect("static variant"),
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
pub type Forest = std::vec::Vec<Tree>;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wrapped(
    pub std::option::Option<protos::Integer>,
    pub std::result::Result<protos::Text, protos::Integer>,
    pub std::vec::Vec<std::option::Option<protos::Text>>,
);
impl datom_codec::Datomic for Wrapped {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 3)?;
        let p0: std::option::Option<protos::Integer> = datom_codec::Positional::position(
            &mut p,
        )?;
        let p1: std::result::Result<protos::Text, protos::Integer> = datom_codec::Positional::position(
            &mut p,
        )?;
        let p2: std::vec::Vec<std::option::Option<protos::Text>> = datom_codec::Positional::position(
            &mut p,
        )?;
        std::result::Result::Ok(Self(p0, p1, p2))
    }
}
impl protos::Conceivable<datom_codec::Datom> for Wrapped {
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
                        .expect("infallible datom ascent").1,
                        protos::Conceivable::conceive(& self.2)
                        .expect("infallible datom ascent").1
                    ],
                ),
            ),
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NestedA {
    X,
    Y(protos::Integer),
}
impl datom_codec::Datomic for NestedA {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "X" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::X)
            }
            "Y" => std::result::Result::Ok(Self::Y(datom_codec::Carrying::body(v)?)),
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
impl protos::Conceivable<datom_codec::Datom> for NestedA {
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
                    Self::X => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("X").expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                    Self::Y(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Y").expect("static variant"),
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
pub enum Nested {
    A(NestedA),
    B(protos::Text),
}
impl datom_codec::Datomic for Nested {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "A" => std::result::Result::Ok(Self::A(datom_codec::Carrying::body(v)?)),
            "B" => {
                let mut p = datom_codec::Headed::positions(v, 1)?;
                let p0: protos::Text = datom_codec::Positional::position(&mut p)?;
                std::result::Result::Ok(Self::B(p0))
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
impl protos::Conceivable<datom_codec::Datom> for Nested {
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
                    Self::B(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("B").expect("static variant"),
                            std::boxed::Box::new(
                                datom_codec::Datom::Struct(
                                    vec![
                                        protos::Conceivable::conceive(p0)
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
pub type Deep = std::vec::Vec<
    std::vec::Vec<
        std::vec::Vec<
            std::option::Option<std::result::Result<protos::Text, protos::Integer>>,
        >,
    >,
>;
