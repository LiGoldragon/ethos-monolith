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
            Self::Leaf(p0) => {
                datom_codec::Datom::Variant(
                    "Leaf".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Node(p0, p1) => {
                datom_codec::Datom::Variant(
                    "Node".to_owned(),
                    Box::new(
                        datom_codec::Datom::Struct(
                            vec![
                                datom_codec::Datomic::conceive(p0),
                                datom_codec::Datomic::conceive(p1)
                            ],
                        ),
                    ),
                )
            }
            Self::Many(p0) => {
                datom_codec::Datom::Variant(
                    "Many".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Maybe(p0) => {
                datom_codec::Datom::Variant(
                    "Maybe".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
        }
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
    fn conceive(&self) -> datom_codec::Datom {
        datom_codec::Datom::Struct(
            vec![
                datom_codec::Datomic::conceive(& self.0),
                datom_codec::Datomic::conceive(& self.1)
            ],
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
    fn conceive(&self) -> datom_codec::Datom {
        datom_codec::Datom::Struct(
            vec![
                datom_codec::Datomic::conceive(& self.0),
                datom_codec::Datomic::conceive(& self.1)
            ],
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
            Self::Tip => datom_codec::Datom::Word("Tip".to_owned()),
            Self::Grow(p0) => {
                datom_codec::Datom::Variant(
                    "Grow".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
        }
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
            Self::X => datom_codec::Datom::Word("X".to_owned()),
            Self::Y(p0) => {
                datom_codec::Datom::Variant(
                    "Y".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
        }
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
            Self::A(p0) => {
                datom_codec::Datom::Variant(
                    "A".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::B(p0) => {
                datom_codec::Datom::Variant(
                    "B".to_owned(),
                    Box::new(
                        datom_codec::Datom::Struct(
                            vec![datom_codec::Datomic::conceive(p0)],
                        ),
                    ),
                )
            }
        }
    }
}
pub type Deep = Vec<Vec<Vec<Option<Result<protos::Text, protos::Integer>>>>>;
