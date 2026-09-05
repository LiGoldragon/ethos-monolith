#![allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tree {
    Leaf(protos::Integer),
    Node(Box<Tree>, Box<Tree>),
    Many(Vec<Tree>),
    Maybe(Box<Option<Tree>>),
}
impl datom_codec::Datomic for Tree {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Leaf" => Ok(Self::Leaf(datom_codec::Carrying::body(v)?)),
            "Node" => {
                let mut p = datom_codec::Headed::positions(v, 2)?;
                let p0: Box<Tree> = datom_codec::Positional::position(&mut p)?;
                let p1: Box<Tree> = datom_codec::Positional::position(&mut p)?;
                Ok(Self::Node(p0, p1))
            }
            "Many" => Ok(Self::Many(datom_codec::Carrying::body(v)?)),
            "Maybe" => Ok(Self::Maybe(datom_codec::Carrying::body(v)?)),
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
impl protos::Conceivable<datom_codec::Datom> for Tree {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                match self {
                    Self::Leaf(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Leaf").expect("static variant"),
                            Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Node(p0, p1) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Node").expect("static variant"),
                            Box::new(
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
                            Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Maybe(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Maybe").expect("static variant"),
                            Box::new(
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
pub struct Chain(pub protos::Text, pub Box<Option<Chain>>);
impl datom_codec::Datomic for Chain {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 2)?;
        let p0: protos::Text = datom_codec::Positional::position(&mut p)?;
        let p1: Box<Option<Chain>> = datom_codec::Positional::position(&mut p)?;
        Ok(Self(p0, p1))
    }
}
impl protos::Conceivable<datom_codec::Datom> for Chain {
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Twin(pub Box<Twig>, pub Box<Twig>);
impl datom_codec::Datomic for Twin {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 2)?;
        let p0: Box<Twig> = datom_codec::Positional::position(&mut p)?;
        let p1: Box<Twig> = datom_codec::Positional::position(&mut p)?;
        Ok(Self(p0, p1))
    }
}
impl protos::Conceivable<datom_codec::Datom> for Twin {
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Twig {
    Tip,
    Grow(Box<Twin>),
}
impl datom_codec::Datomic for Twig {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Tip" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Tip)
            }
            "Grow" => Ok(Self::Grow(datom_codec::Carrying::body(v)?)),
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
impl protos::Conceivable<datom_codec::Datom> for Twig {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        Ok(
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
                            Box::new(
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
pub type Forest = Vec<Tree>;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wrapped(
    pub Option<protos::Integer>,
    pub Result<protos::Text, protos::Integer>,
    pub Vec<Option<protos::Text>>,
);
impl datom_codec::Datomic for Wrapped {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 3)?;
        let p0: Option<protos::Integer> = datom_codec::Positional::position(&mut p)?;
        let p1: Result<protos::Text, protos::Integer> = datom_codec::Positional::position(
            &mut p,
        )?;
        let p2: Vec<Option<protos::Text>> = datom_codec::Positional::position(&mut p)?;
        Ok(Self(p0, p1, p2))
    }
}
impl protos::Conceivable<datom_codec::Datom> for Wrapped {
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NestedA {
    X,
    Y(protos::Integer),
}
impl datom_codec::Datomic for NestedA {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "X" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::X)
            }
            "Y" => Ok(Self::Y(datom_codec::Carrying::body(v)?)),
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
impl protos::Conceivable<datom_codec::Datom> for NestedA {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        Ok(
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
                            Box::new(
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
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "A" => Ok(Self::A(datom_codec::Carrying::body(v)?)),
            "B" => {
                let mut p = datom_codec::Headed::positions(v, 1)?;
                let p0: protos::Text = datom_codec::Positional::position(&mut p)?;
                Ok(Self::B(p0))
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
impl protos::Conceivable<datom_codec::Datom> for Nested {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                match self {
                    Self::A(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("A").expect("static variant"),
                            Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::B(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("B").expect("static variant"),
                            Box::new(
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
pub type Deep = Vec<Vec<Vec<Option<Result<protos::Text, protos::Integer>>>>>;
