#![allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    Structural(protos::Fault),
    Conceptual(Vec<protos::Integer>, Problem),
}
impl protos::Conceivable<datomic::Datom> for Fault {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            match self {
                Self::Structural(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Structural".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
                Self::Conceptual(p0, p1) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Conceptual".to_owned()),
                        Box::new(
                            datomic::Datom::Struct(
                                Vec::from([
                                    protos::Conceivable::<datomic::Datom>::conceive(p0)?,
                                    protos::Conceivable::<datomic::Datom>::conceive(p1)?,
                                ]),
                            ),
                        ),
                    )
                }
            },
        )
    }
}
impl datomic::Datomic for Fault {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Variant(protos::Head::Bare(head), body) => {
                match head.as_str() {
                    "Structural" => {
                        match <protos::Fault as datomic::Datomic>::incorporate_from(
                            *body,
                        ) {
                            Ok(value) => Ok(Self::Structural(value)),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    "Conceptual" => {
                        match *body {
                            datomic::Datom::Struct(fields) => {
                                let incorporated = match <[datomic::Datom; 2]>::try_from(
                                    fields,
                                ) {
                                    Ok([d0, d1]) => {
                                        match <Vec<
                                            protos::Integer,
                                        > as datomic::Datomic>::incorporate_from(d0) {
                                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                                            Ok(p0) => {
                                                match <Problem as datomic::Datomic>::incorporate_from(d1) {
                                                    Err(fault) => Err(datomic::Prepending::prepend(fault, 1)),
                                                    Ok(p1) => Ok(Self::Conceptual(p0, p1)),
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
impl protos::Incorporable<Fault> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<Fault, datomic::Fault> {
        <Fault as datomic::Datomic>::incorporate_from(self)
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
impl protos::Conceivable<datomic::Datom> for Problem {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            match self {
                Self::Root => datomic::Datom::Bare("Root".to_owned()),
                Self::Arity(p0, p1) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Arity".to_owned()),
                        Box::new(
                            datomic::Datom::Struct(
                                Vec::from([
                                    protos::Conceivable::<datomic::Datom>::conceive(p0)?,
                                    protos::Conceivable::<datomic::Datom>::conceive(p1)?,
                                ]),
                            ),
                        ),
                    )
                }
                Self::Expected(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Expected".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
                Self::Separator(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Separator".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
                Self::Name(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Name".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
                Self::Duplicate(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Duplicate".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
                Self::Undeclared(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Undeclared".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
                Self::Cycle(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Cycle".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
                Self::Yield => datomic::Datom::Bare("Yield".to_owned()),
                Self::Empty => datomic::Datom::Bare("Empty".to_owned()),
                Self::Depth => datomic::Datom::Bare("Depth".to_owned()),
                Self::Role(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Role".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
            },
        )
    }
}
impl datomic::Datomic for Problem {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Bare(symbol) => {
                match symbol.as_str() {
                    "Root" => Ok(Self::Root),
                    "Yield" => Ok(Self::Yield),
                    "Empty" => Ok(Self::Empty),
                    "Depth" => Ok(Self::Depth),
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
                    "Arity" => {
                        match *body {
                            datomic::Datom::Struct(fields) => {
                                let incorporated = match <[datomic::Datom; 2]>::try_from(
                                    fields,
                                ) {
                                    Ok([d0, d1]) => {
                                        match <protos::Integer as datomic::Datomic>::incorporate_from(
                                            d0,
                                        ) {
                                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                                            Ok(p0) => {
                                                match <protos::Integer as datomic::Datomic>::incorporate_from(
                                                    d1,
                                                ) {
                                                    Err(fault) => Err(datomic::Prepending::prepend(fault, 1)),
                                                    Ok(p1) => Ok(Self::Arity(p0, p1)),
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
                    "Expected" => {
                        match <Form as datomic::Datomic>::incorporate_from(*body) {
                            Ok(value) => Ok(Self::Expected(value)),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    "Separator" => {
                        match <protos::Separator as datomic::Datomic>::incorporate_from(
                            *body,
                        ) {
                            Ok(value) => Ok(Self::Separator(value)),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    "Name" => {
                        match <protos::Text as datomic::Datomic>::incorporate_from(
                            *body,
                        ) {
                            Ok(value) => Ok(Self::Name(value)),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    "Duplicate" => {
                        match <protos::Text as datomic::Datomic>::incorporate_from(
                            *body,
                        ) {
                            Ok(value) => Ok(Self::Duplicate(value)),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    "Undeclared" => {
                        match <protos::Text as datomic::Datomic>::incorporate_from(
                            *body,
                        ) {
                            Ok(value) => Ok(Self::Undeclared(value)),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    "Cycle" => {
                        match <protos::Text as datomic::Datomic>::incorporate_from(
                            *body,
                        ) {
                            Ok(value) => Ok(Self::Cycle(value)),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    "Role" => {
                        match <protos::Text as datomic::Datomic>::incorporate_from(
                            *body,
                        ) {
                            Ok(value) => Ok(Self::Role(value)),
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
impl protos::Incorporable<Problem> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<Problem, datomic::Fault> {
        <Problem as datomic::Datomic>::incorporate_from(self)
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
impl protos::Conceivable<datomic::Datom> for Form {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            match self {
                Self::File => datomic::Datom::Bare("File".to_owned()),
                Self::Section => datomic::Datom::Bare("Section".to_owned()),
                Self::Import => datomic::Datom::Bare("Import".to_owned()),
                Self::Name => datomic::Datom::Bare("Name".to_owned()),
                Self::Declaration => datomic::Datom::Bare("Declaration".to_owned()),
                Self::Variant => datomic::Datom::Bare("Variant".to_owned()),
                Self::Reference => datomic::Datom::Bare("Reference".to_owned()),
                Self::Constraint => datomic::Datom::Bare("Constraint".to_owned()),
                Self::Kind => datomic::Datom::Bare("Kind".to_owned()),
                Self::Capability => datomic::Datom::Bare("Capability".to_owned()),
                Self::Constant => datomic::Datom::Bare("Constant".to_owned()),
                Self::Association => datomic::Datom::Bare("Association".to_owned()),
            },
        )
    }
}
impl datomic::Datomic for Form {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Bare(symbol) => {
                match symbol.as_str() {
                    "File" => Ok(Self::File),
                    "Section" => Ok(Self::Section),
                    "Import" => Ok(Self::Import),
                    "Name" => Ok(Self::Name),
                    "Declaration" => Ok(Self::Declaration),
                    "Variant" => Ok(Self::Variant),
                    "Reference" => Ok(Self::Reference),
                    "Constraint" => Ok(Self::Constraint),
                    "Kind" => Ok(Self::Kind),
                    "Capability" => Ok(Self::Capability),
                    "Constant" => Ok(Self::Constant),
                    "Association" => Ok(Self::Association),
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
impl protos::Incorporable<Form> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<Form, datomic::Fault> {
        <Form as datomic::Datomic>::incorporate_from(self)
    }
}
