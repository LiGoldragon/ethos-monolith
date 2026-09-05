#![allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sink(pub protos::Text, pub Vec<protos::Text>);
impl protos::Conceivable<datomic::Datom> for Sink {
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
impl datomic::Datomic for Sink {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Struct(fields) => {
                match <[datomic::Datom; 2]>::try_from(fields) {
                    Ok([d0, d1]) => {
                        match <protos::Text as datomic::Datomic>::incorporate_from(d0) {
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                            Ok(p0) => {
                                match <Vec<
                                    protos::Text,
                                > as datomic::Datomic>::incorporate_from(d1) {
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
impl protos::Incorporable<Sink> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<Sink, datomic::Fault> {
        <Sink as datomic::Datomic>::incorporate_from(self)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SinkError {
    Closed,
    Full,
}
impl protos::Conceivable<datomic::Datom> for SinkError {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            match self {
                Self::Closed => datomic::Datom::Bare("Closed".to_owned()),
                Self::Full => datomic::Datom::Bare("Full".to_owned()),
            },
        )
    }
}
impl datomic::Datomic for SinkError {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Bare(symbol) => {
                match symbol.as_str() {
                    "Closed" => Ok(Self::Closed),
                    "Full" => Ok(Self::Full),
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
impl protos::Incorporable<SinkError> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<SinkError, datomic::Fault> {
        <SinkError as datomic::Datomic>::incorporate_from(self)
    }
}
const _: () = {
    fn assert_sink_summarizable<T: super::Summarizable>() {}
    let _ = assert_sink_summarizable::<Sink>;
    fn assert_sink_fillable<T: super::Fillable>() {}
    let _ = assert_sink_fillable::<Sink>;
};
