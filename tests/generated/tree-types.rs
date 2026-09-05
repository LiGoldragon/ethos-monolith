#![allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tree {
    Leaf(protos::Integer),
    Node(Box<Tree>, Box<Tree>),
    Many(Vec<Tree>),
    Maybe(Box<Option<Tree>>),
}
impl protos::Conceivable<datomic::Datom> for Tree {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            match self {
                Self::Leaf(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Leaf".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
                Self::Node(p0, p1) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Node".to_owned()),
                        Box::new(
                            datomic::Datom::Struct(
                                Vec::from([
                                    protos::Conceivable::<datomic::Datom>::conceive(&**p0)?,
                                    protos::Conceivable::<datomic::Datom>::conceive(&**p1)?,
                                ]),
                            ),
                        ),
                    )
                }
                Self::Many(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Many".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
                Self::Maybe(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Maybe".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(&**p0)?),
                    )
                }
            },
        )
    }
}
impl datomic::Datomic for Tree {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Variant(protos::Head::Bare(head), body) => {
                match head.as_str() {
                    "Leaf" => {
                        match <protos::Integer as datomic::Datomic>::incorporate_from(
                            *body,
                        ) {
                            Ok(value) => Ok(Self::Leaf(value)),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    "Node" => {
                        match *body {
                            datomic::Datom::Struct(fields) => {
                                let incorporated = match <[datomic::Datom; 2]>::try_from(
                                    fields,
                                ) {
                                    Ok([d0, d1]) => {
                                        match <Tree as datomic::Datomic>::incorporate_from(d0) {
                                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                                            Ok(p0) => {
                                                let p0 = Box::new(p0);
                                                match <Tree as datomic::Datomic>::incorporate_from(d1) {
                                                    Err(fault) => Err(datomic::Prepending::prepend(fault, 1)),
                                                    Ok(p1) => {
                                                        let p1 = Box::new(p1);
                                                        Ok(Self::Node(p0, p1))
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(fields) => {
                                        Err(
                                            datomic::Fault::Corporate(
                                                vec![],
                                                datomic::Problem::Arity(2, fields.len() as protos::Integer),
                                            ),
                                        )
                                    }
                                };
                                match incorporated {
                                    Ok(value) => Ok(value),
                                    Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                                }
                            }
                            other => {
                                Err(
                                    datomic::Fault::Corporate(
                                        vec![0],
                                        datomic::Problem::Shape(datomic::Expected::Struct, other),
                                    ),
                                )
                            }
                        }
                    }
                    "Many" => {
                        match <Vec<Tree> as datomic::Datomic>::incorporate_from(*body) {
                            Ok(value) => Ok(Self::Many(value)),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    "Maybe" => {
                        match <Option<
                            Tree,
                        > as datomic::Datomic>::incorporate_from(*body) {
                            Ok(value) => Ok(Self::Maybe(Box::new(value))),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    _ => {
                        Err(
                            datomic::Fault::Corporate(
                                vec![],
                                datomic::Problem::UnknownVariant(head),
                            ),
                        )
                    }
                }
            }
            other => {
                Err(
                    datomic::Fault::Corporate(
                        vec![],
                        datomic::Problem::Shape(datomic::Expected::Variant, other),
                    ),
                )
            }
        }
    }
}
impl protos::Incorporable<Tree> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<Tree, datomic::Fault> {
        <Tree as datomic::Datomic>::incorporate_from(self)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chain(pub protos::Text, pub Box<Option<Chain>>);
impl protos::Conceivable<datomic::Datom> for Chain {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            datomic::Datom::Struct(
                Vec::from([
                    protos::Conceivable::<datomic::Datom>::conceive(&self.0)?,
                    protos::Conceivable::<datomic::Datom>::conceive(&*self.1)?,
                ]),
            ),
        )
    }
}
impl datomic::Datomic for Chain {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Struct(fields) => {
                match <[datomic::Datom; 2]>::try_from(fields) {
                    Ok([d0, d1]) => {
                        match <protos::Text as datomic::Datomic>::incorporate_from(d0) {
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                            Ok(p0) => {
                                match <Option<
                                    Chain,
                                > as datomic::Datomic>::incorporate_from(d1) {
                                    Err(fault) => Err(datomic::Prepending::prepend(fault, 1)),
                                    Ok(p1) => {
                                        let p1 = Box::new(p1);
                                        Ok(Self(p0, p1))
                                    }
                                }
                            }
                        }
                    }
                    Err(fields) => {
                        Err(
                            datomic::Fault::Corporate(
                                vec![],
                                datomic::Problem::Arity(2, fields.len() as protos::Integer),
                            ),
                        )
                    }
                }
            }
            other => {
                Err(
                    datomic::Fault::Corporate(
                        vec![],
                        datomic::Problem::Shape(datomic::Expected::Struct, other),
                    ),
                )
            }
        }
    }
}
impl protos::Incorporable<Chain> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<Chain, datomic::Fault> {
        <Chain as datomic::Datomic>::incorporate_from(self)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Twin(pub Box<Twig>, pub Box<Twig>);
impl protos::Conceivable<datomic::Datom> for Twin {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            datomic::Datom::Struct(
                Vec::from([
                    protos::Conceivable::<datomic::Datom>::conceive(&*self.0)?,
                    protos::Conceivable::<datomic::Datom>::conceive(&*self.1)?,
                ]),
            ),
        )
    }
}
impl datomic::Datomic for Twin {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Struct(fields) => {
                match <[datomic::Datom; 2]>::try_from(fields) {
                    Ok([d0, d1]) => {
                        match <Twig as datomic::Datomic>::incorporate_from(d0) {
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                            Ok(p0) => {
                                let p0 = Box::new(p0);
                                match <Twig as datomic::Datomic>::incorporate_from(d1) {
                                    Err(fault) => Err(datomic::Prepending::prepend(fault, 1)),
                                    Ok(p1) => {
                                        let p1 = Box::new(p1);
                                        Ok(Self(p0, p1))
                                    }
                                }
                            }
                        }
                    }
                    Err(fields) => {
                        Err(
                            datomic::Fault::Corporate(
                                vec![],
                                datomic::Problem::Arity(2, fields.len() as protos::Integer),
                            ),
                        )
                    }
                }
            }
            other => {
                Err(
                    datomic::Fault::Corporate(
                        vec![],
                        datomic::Problem::Shape(datomic::Expected::Struct, other),
                    ),
                )
            }
        }
    }
}
impl protos::Incorporable<Twin> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<Twin, datomic::Fault> {
        <Twin as datomic::Datomic>::incorporate_from(self)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Twig {
    Tip,
    Grow(Box<Twin>),
}
impl protos::Conceivable<datomic::Datom> for Twig {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            match self {
                Self::Tip => datomic::Datom::Bare("Tip".to_owned()),
                Self::Grow(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Grow".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(&**p0)?),
                    )
                }
            },
        )
    }
}
impl datomic::Datomic for Twig {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Bare(symbol) => {
                match symbol.as_str() {
                    "Tip" => Ok(Self::Tip),
                    _ => {
                        Err(
                            datomic::Fault::Corporate(
                                vec![],
                                datomic::Problem::UnknownVariant(symbol),
                            ),
                        )
                    }
                }
            }
            datomic::Datom::Variant(protos::Head::Bare(head), body) => {
                match head.as_str() {
                    "Grow" => {
                        match <Twin as datomic::Datomic>::incorporate_from(*body) {
                            Ok(value) => Ok(Self::Grow(Box::new(value))),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    _ => {
                        Err(
                            datomic::Fault::Corporate(
                                vec![],
                                datomic::Problem::UnknownVariant(head),
                            ),
                        )
                    }
                }
            }
            other => {
                Err(
                    datomic::Fault::Corporate(
                        vec![],
                        datomic::Problem::Shape(datomic::Expected::Variant, other),
                    ),
                )
            }
        }
    }
}
impl protos::Incorporable<Twig> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<Twig, datomic::Fault> {
        <Twig as datomic::Datomic>::incorporate_from(self)
    }
}
pub type Forest = Vec<Tree>;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wrapped(
    pub Option<protos::Integer>,
    pub Result<protos::Text, protos::Integer>,
    pub Vec<Option<protos::Text>>,
);
impl protos::Conceivable<datomic::Datom> for Wrapped {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            datomic::Datom::Struct(
                Vec::from([
                    protos::Conceivable::<datomic::Datom>::conceive(&self.0)?,
                    protos::Conceivable::<datomic::Datom>::conceive(&self.1)?,
                    protos::Conceivable::<datomic::Datom>::conceive(&self.2)?,
                ]),
            ),
        )
    }
}
impl datomic::Datomic for Wrapped {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Struct(fields) => {
                match <[datomic::Datom; 3]>::try_from(fields) {
                    Ok([d0, d1, d2]) => {
                        match <Option<
                            protos::Integer,
                        > as datomic::Datomic>::incorporate_from(d0) {
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                            Ok(p0) => {
                                match <Result<
                                    protos::Text,
                                    protos::Integer,
                                > as datomic::Datomic>::incorporate_from(d1) {
                                    Err(fault) => Err(datomic::Prepending::prepend(fault, 1)),
                                    Ok(p1) => {
                                        match <Vec<
                                            Option<protos::Text>,
                                        > as datomic::Datomic>::incorporate_from(d2) {
                                            Err(fault) => Err(datomic::Prepending::prepend(fault, 2)),
                                            Ok(p2) => Ok(Self(p0, p1, p2)),
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(fields) => {
                        Err(
                            datomic::Fault::Corporate(
                                vec![],
                                datomic::Problem::Arity(3, fields.len() as protos::Integer),
                            ),
                        )
                    }
                }
            }
            other => {
                Err(
                    datomic::Fault::Corporate(
                        vec![],
                        datomic::Problem::Shape(datomic::Expected::Struct, other),
                    ),
                )
            }
        }
    }
}
impl protos::Incorporable<Wrapped> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<Wrapped, datomic::Fault> {
        <Wrapped as datomic::Datomic>::incorporate_from(self)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NestedA {
    X,
    Y(protos::Integer),
}
impl protos::Conceivable<datomic::Datom> for NestedA {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            match self {
                Self::X => datomic::Datom::Bare("X".to_owned()),
                Self::Y(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Y".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
            },
        )
    }
}
impl datomic::Datomic for NestedA {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Bare(symbol) => {
                match symbol.as_str() {
                    "X" => Ok(Self::X),
                    _ => {
                        Err(
                            datomic::Fault::Corporate(
                                vec![],
                                datomic::Problem::UnknownVariant(symbol),
                            ),
                        )
                    }
                }
            }
            datomic::Datom::Variant(protos::Head::Bare(head), body) => {
                match head.as_str() {
                    "Y" => {
                        match <protos::Integer as datomic::Datomic>::incorporate_from(
                            *body,
                        ) {
                            Ok(value) => Ok(Self::Y(value)),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    _ => {
                        Err(
                            datomic::Fault::Corporate(
                                vec![],
                                datomic::Problem::UnknownVariant(head),
                            ),
                        )
                    }
                }
            }
            other => {
                Err(
                    datomic::Fault::Corporate(
                        vec![],
                        datomic::Problem::Shape(datomic::Expected::Variant, other),
                    ),
                )
            }
        }
    }
}
impl protos::Incorporable<NestedA> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<NestedA, datomic::Fault> {
        <NestedA as datomic::Datomic>::incorporate_from(self)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Nested {
    A(NestedA),
    B(protos::Text),
}
impl protos::Conceivable<datomic::Datom> for Nested {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            match self {
                Self::A(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("A".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
                Self::B(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("B".to_owned()),
                        Box::new(
                            datomic::Datom::Struct(
                                Vec::from([
                                    protos::Conceivable::<datomic::Datom>::conceive(p0)?,
                                ]),
                            ),
                        ),
                    )
                }
            },
        )
    }
}
impl datomic::Datomic for Nested {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Variant(protos::Head::Bare(head), body) => {
                match head.as_str() {
                    "A" => {
                        match datomic::Datomic::incorporate_from(*body) {
                            Ok(value) => Ok(Self::A(value)),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    "B" => {
                        match *body {
                            datomic::Datom::Struct(fields) => {
                                let incorporated = match <[datomic::Datom; 1]>::try_from(
                                    fields,
                                ) {
                                    Ok([d0]) => {
                                        match <protos::Text as datomic::Datomic>::incorporate_from(
                                            d0,
                                        ) {
                                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                                            Ok(p0) => Ok(Self::B(p0)),
                                        }
                                    }
                                    Err(fields) => {
                                        Err(
                                            datomic::Fault::Corporate(
                                                vec![],
                                                datomic::Problem::Arity(1, fields.len() as protos::Integer),
                                            ),
                                        )
                                    }
                                };
                                match incorporated {
                                    Ok(value) => Ok(value),
                                    Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                                }
                            }
                            other => {
                                Err(
                                    datomic::Fault::Corporate(
                                        vec![0],
                                        datomic::Problem::Shape(datomic::Expected::Struct, other),
                                    ),
                                )
                            }
                        }
                    }
                    _ => {
                        Err(
                            datomic::Fault::Corporate(
                                vec![],
                                datomic::Problem::UnknownVariant(head),
                            ),
                        )
                    }
                }
            }
            other => {
                Err(
                    datomic::Fault::Corporate(
                        vec![],
                        datomic::Problem::Shape(datomic::Expected::Variant, other),
                    ),
                )
            }
        }
    }
}
impl protos::Incorporable<Nested> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<Nested, datomic::Fault> {
        <Nested as datomic::Datomic>::incorporate_from(self)
    }
}
pub type Deep = Vec<Vec<Vec<Option<Result<protos::Text, protos::Integer>>>>>;
