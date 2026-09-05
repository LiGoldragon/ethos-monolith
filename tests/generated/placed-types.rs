#![allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placed<A: Sized>(pub Option<protos::Integer>, pub A);
impl<A: Sized + datomic::Datomic> protos::Conceivable<datomic::Datom> for Placed<A> {
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
impl<A: Sized + datomic::Datomic> datomic::Datomic for Placed<A> {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Struct(fields) => {
                match <[datomic::Datom; 2]>::try_from(fields) {
                    Ok([d0, d1]) => {
                        match <Option<
                            protos::Integer,
                        > as datomic::Datomic>::incorporate_from(d0) {
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                            Ok(p0) => {
                                match <A as datomic::Datomic>::incorporate_from(d1) {
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
impl<A: Sized + datomic::Datomic> protos::Incorporable<Placed<A>> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<Placed<A>, datomic::Fault> {
        <Placed<A> as datomic::Datomic>::incorporate_from(self)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Score(pub protos::Decimal, pub protos::Boolean, pub datomic::Meaning);
impl protos::Conceivable<datomic::Datom> for Score {
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
impl datomic::Datomic for Score {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Struct(fields) => {
                match <[datomic::Datom; 3]>::try_from(fields) {
                    Ok([d0, d1, d2]) => {
                        match <protos::Decimal as datomic::Datomic>::incorporate_from(
                            d0,
                        ) {
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                            Ok(p0) => {
                                match <protos::Boolean as datomic::Datomic>::incorporate_from(
                                    d1,
                                ) {
                                    Err(fault) => Err(datomic::Prepending::prepend(fault, 1)),
                                    Ok(p1) => {
                                        match <datomic::Meaning as datomic::Datomic>::incorporate_from(
                                            d2,
                                        ) {
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
impl protos::Incorporable<Score> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<Score, datomic::Fault> {
        <Score as datomic::Datomic>::incorporate_from(self)
    }
}
