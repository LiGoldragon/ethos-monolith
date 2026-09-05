#![allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    Structural(protos::Fault),
    Conceptual(Vec<protos::Integer>, Problem),
}
impl datom_codec::Datomic for Fault {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Structural" => Ok(Self::Structural(datom_codec::Carrying::body(v)?)),
            "Conceptual" => {
                let mut p = datom_codec::Headed::positions(v, 2)?;
                let p0: Vec<protos::Integer> = datom_codec::Positional::position(
                    &mut p,
                )?;
                let p1: Problem = datom_codec::Positional::position(&mut p)?;
                Ok(Self::Conceptual(p0, p1))
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
            Self::Structural(p0) => {
                datom_codec::Datom::Variant(
                    "Structural".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Conceptual(p0, p1) => {
                datom_codec::Datom::Variant(
                    "Conceptual".to_owned(),
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
        }
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
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Root" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Root)
            }
            "Arity" => {
                let mut p = datom_codec::Headed::positions(v, 2)?;
                let p0: protos::Integer = datom_codec::Positional::position(&mut p)?;
                let p1: protos::Integer = datom_codec::Positional::position(&mut p)?;
                Ok(Self::Arity(p0, p1))
            }
            "Expected" => Ok(Self::Expected(datom_codec::Carrying::body(v)?)),
            "Separator" => Ok(Self::Separator(datom_codec::Carrying::body(v)?)),
            "Name" => Ok(Self::Name(datom_codec::Carrying::body(v)?)),
            "Duplicate" => Ok(Self::Duplicate(datom_codec::Carrying::body(v)?)),
            "Undeclared" => Ok(Self::Undeclared(datom_codec::Carrying::body(v)?)),
            "Cycle" => Ok(Self::Cycle(datom_codec::Carrying::body(v)?)),
            "Yield" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Yield)
            }
            "Empty" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Empty)
            }
            "Depth" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Depth)
            }
            "Role" => Ok(Self::Role(datom_codec::Carrying::body(v)?)),
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
            Self::Root => datom_codec::Datom::Word("Root".to_owned()),
            Self::Arity(p0, p1) => {
                datom_codec::Datom::Variant(
                    "Arity".to_owned(),
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
            Self::Expected(p0) => {
                datom_codec::Datom::Variant(
                    "Expected".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Separator(p0) => {
                datom_codec::Datom::Variant(
                    "Separator".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Name(p0) => {
                datom_codec::Datom::Variant(
                    "Name".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Duplicate(p0) => {
                datom_codec::Datom::Variant(
                    "Duplicate".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Undeclared(p0) => {
                datom_codec::Datom::Variant(
                    "Undeclared".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Cycle(p0) => {
                datom_codec::Datom::Variant(
                    "Cycle".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Yield => datom_codec::Datom::Word("Yield".to_owned()),
            Self::Empty => datom_codec::Datom::Word("Empty".to_owned()),
            Self::Depth => datom_codec::Datom::Word("Depth".to_owned()),
            Self::Role(p0) => {
                datom_codec::Datom::Variant(
                    "Role".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
        }
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
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "File" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::File)
            }
            "Section" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Section)
            }
            "Import" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Import)
            }
            "Name" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Name)
            }
            "Declaration" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Declaration)
            }
            "Variant" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Variant)
            }
            "Reference" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Reference)
            }
            "Constraint" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Constraint)
            }
            "Kind" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Kind)
            }
            "Capability" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Capability)
            }
            "Constant" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Constant)
            }
            "Association" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Association)
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
            Self::File => datom_codec::Datom::Word("File".to_owned()),
            Self::Section => datom_codec::Datom::Word("Section".to_owned()),
            Self::Import => datom_codec::Datom::Word("Import".to_owned()),
            Self::Name => datom_codec::Datom::Word("Name".to_owned()),
            Self::Declaration => datom_codec::Datom::Word("Declaration".to_owned()),
            Self::Variant => datom_codec::Datom::Word("Variant".to_owned()),
            Self::Reference => datom_codec::Datom::Word("Reference".to_owned()),
            Self::Constraint => datom_codec::Datom::Word("Constraint".to_owned()),
            Self::Kind => datom_codec::Datom::Word("Kind".to_owned()),
            Self::Capability => datom_codec::Datom::Word("Capability".to_owned()),
            Self::Constant => datom_codec::Datom::Word("Constant".to_owned()),
            Self::Association => datom_codec::Datom::Word("Association".to_owned()),
        }
    }
}
