#![allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Generation(pub protos::Text, pub protos::Text);
impl protos::Conceivable<datomic::Datom> for Generation {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            datomic::Datom::Struct(
                Vec::from([
                    protos::Conceivable::<datomic::Datom>::conceive(&self.0)?,
                    protos::Conceivable::<datomic::Datom>::conceive(&self.1)?,
                ]),
            ),
        )
    }
}
impl datomic::Datomic for Generation {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Struct(fields) => {
                match <[datomic::Datom; 2]>::try_from(fields) {
                    Ok([d0, d1]) => {
                        match <protos::Text as datomic::Datomic>::incorporate_from(d0) {
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                            Ok(p0) => {
                                match <protos::Text as datomic::Datomic>::incorporate_from(
                                    d1,
                                ) {
                                    Err(fault) => Err(datomic::Prepending::prepend(fault, 1)),
                                    Ok(p1) => Ok(Self(p0, p1)),
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
impl protos::Incorporable<Generation> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<Generation, datomic::Fault> {
        <Generation as datomic::Datomic>::incorporate_from(self)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Generate(Generation),
}
impl protos::Conceivable<datomic::Datom> for Request {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            match self {
                Self::Generate(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Generate".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
            },
        )
    }
}
impl datomic::Datomic for Request {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Variant(protos::Head::Bare(head), body) => {
                match head.as_str() {
                    "Generate" => {
                        match <Generation as datomic::Datomic>::incorporate_from(*body) {
                            Ok(value) => Ok(Self::Generate(value)),
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
impl protos::Incorporable<Request> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<Request, datomic::Fault> {
        <Request as datomic::Datomic>::incorporate_from(self)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Response {
    Generated(Vec<protos::Text>),
    Arguments(protos::Integer),
    Malformed(datomic::Situated<datomic::Fault>),
    Unreadable(protos::Text, protos::Text),
    Faulty(protos::Text, datomic::Situated<ethos_zero::Fault>),
    Unwritable(protos::Text, protos::Text),
}
impl protos::Conceivable<datomic::Datom> for Response {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            match self {
                Self::Generated(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Generated".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
                Self::Arguments(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Arguments".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
                Self::Malformed(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Malformed".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
                Self::Unreadable(p0, p1) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Unreadable".to_owned()),
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
                Self::Faulty(p0, p1) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Faulty".to_owned()),
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
                Self::Unwritable(p0, p1) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Unwritable".to_owned()),
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
impl datomic::Datomic for Response {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Variant(protos::Head::Bare(head), body) => {
                match head.as_str() {
                    "Generated" => {
                        match <Vec<
                            protos::Text,
                        > as datomic::Datomic>::incorporate_from(*body) {
                            Ok(value) => Ok(Self::Generated(value)),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    "Arguments" => {
                        match <protos::Integer as datomic::Datomic>::incorporate_from(
                            *body,
                        ) {
                            Ok(value) => Ok(Self::Arguments(value)),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    "Malformed" => {
                        match <datomic::Situated<
                            datomic::Fault,
                        > as datomic::Datomic>::incorporate_from(*body) {
                            Ok(value) => Ok(Self::Malformed(value)),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    "Unreadable" => {
                        match *body {
                            datomic::Datom::Struct(fields) => {
                                let incorporated = match <[datomic::Datom; 2]>::try_from(
                                    fields,
                                ) {
                                    Ok([d0, d1]) => {
                                        match <protos::Text as datomic::Datomic>::incorporate_from(
                                            d0,
                                        ) {
                                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                                            Ok(p0) => {
                                                match <protos::Text as datomic::Datomic>::incorporate_from(
                                                    d1,
                                                ) {
                                                    Err(fault) => Err(datomic::Prepending::prepend(fault, 1)),
                                                    Ok(p1) => Ok(Self::Unreadable(p0, p1)),
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
                    "Faulty" => {
                        match *body {
                            datomic::Datom::Struct(fields) => {
                                let incorporated = match <[datomic::Datom; 2]>::try_from(
                                    fields,
                                ) {
                                    Ok([d0, d1]) => {
                                        match <protos::Text as datomic::Datomic>::incorporate_from(
                                            d0,
                                        ) {
                                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                                            Ok(p0) => {
                                                match <datomic::Situated<
                                                    ethos_zero::Fault,
                                                > as datomic::Datomic>::incorporate_from(d1) {
                                                    Err(fault) => Err(datomic::Prepending::prepend(fault, 1)),
                                                    Ok(p1) => Ok(Self::Faulty(p0, p1)),
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
                    "Unwritable" => {
                        match *body {
                            datomic::Datom::Struct(fields) => {
                                let incorporated = match <[datomic::Datom; 2]>::try_from(
                                    fields,
                                ) {
                                    Ok([d0, d1]) => {
                                        match <protos::Text as datomic::Datomic>::incorporate_from(
                                            d0,
                                        ) {
                                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                                            Ok(p0) => {
                                                match <protos::Text as datomic::Datomic>::incorporate_from(
                                                    d1,
                                                ) {
                                                    Err(fault) => Err(datomic::Prepending::prepend(fault, 1)),
                                                    Ok(p1) => Ok(Self::Unwritable(p0, p1)),
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
impl protos::Incorporable<Response> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<Response, datomic::Fault> {
        <Response as datomic::Datomic>::incorporate_from(self)
    }
}
