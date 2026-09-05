#![allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Generation(pub protos::Text, pub protos::Text);
impl datom_codec::Datomic for Generation {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 2)?;
        let p0: protos::Text = datom_codec::Positional::position(&mut p)?;
        let p1: protos::Text = datom_codec::Positional::position(&mut p)?;
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
pub enum Request {
    Generate(Generation),
}
impl datom_codec::Datomic for Request {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Generate" => Ok(Self::Generate(datom_codec::Carrying::body(v)?)),
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
            Self::Generate(p0) => {
                datom_codec::Datom::Variant(
                    "Generate".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Response {
    Generated(Vec<protos::Text>),
    Arguments(protos::Integer),
    Malformed(datom_codec::Fault),
    Unreadable(protos::Text, protos::Text),
    Faulty(protos::Text, datom_codec::Extent, ethos_zero::Fault),
    Unwritable(protos::Text, protos::Text),
}
impl datom_codec::Datomic for Response {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Generated" => Ok(Self::Generated(datom_codec::Carrying::body(v)?)),
            "Arguments" => Ok(Self::Arguments(datom_codec::Carrying::body(v)?)),
            "Malformed" => Ok(Self::Malformed(datom_codec::Carrying::body(v)?)),
            "Unreadable" => {
                let mut p = datom_codec::Headed::positions(v, 2)?;
                let p0: protos::Text = datom_codec::Positional::position(&mut p)?;
                let p1: protos::Text = datom_codec::Positional::position(&mut p)?;
                Ok(Self::Unreadable(p0, p1))
            }
            "Faulty" => {
                let mut p = datom_codec::Headed::positions(v, 3)?;
                let p0: protos::Text = datom_codec::Positional::position(&mut p)?;
                let p1: datom_codec::Extent = datom_codec::Positional::position(&mut p)?;
                let p2: ethos_zero::Fault = datom_codec::Positional::position(&mut p)?;
                Ok(Self::Faulty(p0, p1, p2))
            }
            "Unwritable" => {
                let mut p = datom_codec::Headed::positions(v, 2)?;
                let p0: protos::Text = datom_codec::Positional::position(&mut p)?;
                let p1: protos::Text = datom_codec::Positional::position(&mut p)?;
                Ok(Self::Unwritable(p0, p1))
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
            Self::Generated(p0) => {
                datom_codec::Datom::Variant(
                    "Generated".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Arguments(p0) => {
                datom_codec::Datom::Variant(
                    "Arguments".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Malformed(p0) => {
                datom_codec::Datom::Variant(
                    "Malformed".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Unreadable(p0, p1) => {
                datom_codec::Datom::Variant(
                    "Unreadable".to_owned(),
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
            Self::Faulty(p0, p1, p2) => {
                datom_codec::Datom::Variant(
                    "Faulty".to_owned(),
                    Box::new(
                        datom_codec::Datom::Struct(
                            vec![
                                datom_codec::Datomic::conceive(p0),
                                datom_codec::Datomic::conceive(p1),
                                datom_codec::Datomic::conceive(p2)
                            ],
                        ),
                    ),
                )
            }
            Self::Unwritable(p0, p1) => {
                datom_codec::Datom::Variant(
                    "Unwritable".to_owned(),
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
