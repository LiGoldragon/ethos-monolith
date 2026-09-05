#![allow(dead_code)]
pub type LockId = protos::Integer;
pub type LockName = protos::Text;
pub type FlowId = protos::Text;
pub type LockPath = protos::Text;
pub type LockPaths = Vec<LockPath>;
pub type LockReason = protos::Text;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockRequest(pub LockName, pub FlowId, pub LockPaths, pub LockReason);
impl protos::Conceivable<datomic::Datom> for LockRequest {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            datomic::Datom::Struct(
                Vec::from([
                    protos::Conceivable::<datomic::Datom>::conceive(&self.0)?,
                    protos::Conceivable::<datomic::Datom>::conceive(&self.1)?,
                    protos::Conceivable::<datomic::Datom>::conceive(&self.2)?,
                    protos::Conceivable::<datomic::Datom>::conceive(&self.3)?,
                ]),
            ),
        )
    }
}
impl datomic::Datomic for LockRequest {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Struct(fields) => {
                match <[datomic::Datom; 4]>::try_from(fields) {
                    Ok([d0, d1, d2, d3]) => {
                        match <LockName as datomic::Datomic>::incorporate_from(d0) {
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                            Ok(p0) => {
                                match <FlowId as datomic::Datomic>::incorporate_from(d1) {
                                    Err(fault) => Err(datomic::Prepending::prepend(fault, 1)),
                                    Ok(p1) => {
                                        match <LockPaths as datomic::Datomic>::incorporate_from(
                                            d2,
                                        ) {
                                            Err(fault) => Err(datomic::Prepending::prepend(fault, 2)),
                                            Ok(p2) => {
                                                match <LockReason as datomic::Datomic>::incorporate_from(
                                                    d3,
                                                ) {
                                                    Err(fault) => Err(datomic::Prepending::prepend(fault, 3)),
                                                    Ok(p3) => Ok(Self(p0, p1, p2, p3)),
                                                }
                                            }
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
                                datomic::Problem::Arity(4, fields.len() as protos::Integer),
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
impl protos::Incorporable<LockRequest> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<LockRequest, datomic::Fault> {
        <LockRequest as datomic::Datomic>::incorporate_from(self)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lock(pub LockId, pub LockName, pub FlowId, pub LockPaths, pub LockReason);
impl protos::Conceivable<datomic::Datom> for Lock {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            datomic::Datom::Struct(
                Vec::from([
                    protos::Conceivable::<datomic::Datom>::conceive(&self.0)?,
                    protos::Conceivable::<datomic::Datom>::conceive(&self.1)?,
                    protos::Conceivable::<datomic::Datom>::conceive(&self.2)?,
                    protos::Conceivable::<datomic::Datom>::conceive(&self.3)?,
                    protos::Conceivable::<datomic::Datom>::conceive(&self.4)?,
                ]),
            ),
        )
    }
}
impl datomic::Datomic for Lock {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Struct(fields) => {
                match <[datomic::Datom; 5]>::try_from(fields) {
                    Ok([d0, d1, d2, d3, d4]) => {
                        match <LockId as datomic::Datomic>::incorporate_from(d0) {
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                            Ok(p0) => {
                                match <LockName as datomic::Datomic>::incorporate_from(d1) {
                                    Err(fault) => Err(datomic::Prepending::prepend(fault, 1)),
                                    Ok(p1) => {
                                        match <FlowId as datomic::Datomic>::incorporate_from(d2) {
                                            Err(fault) => Err(datomic::Prepending::prepend(fault, 2)),
                                            Ok(p2) => {
                                                match <LockPaths as datomic::Datomic>::incorporate_from(
                                                    d3,
                                                ) {
                                                    Err(fault) => Err(datomic::Prepending::prepend(fault, 3)),
                                                    Ok(p3) => {
                                                        match <LockReason as datomic::Datomic>::incorporate_from(
                                                            d4,
                                                        ) {
                                                            Err(fault) => Err(datomic::Prepending::prepend(fault, 4)),
                                                            Ok(p4) => Ok(Self(p0, p1, p2, p3, p4)),
                                                        }
                                                    }
                                                }
                                            }
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
                                datomic::Problem::Arity(5, fields.len() as protos::Integer),
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
impl protos::Incorporable<Lock> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<Lock, datomic::Fault> {
        <Lock as datomic::Datomic>::incorporate_from(self)
    }
}
pub type DuplicateName = Lock;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockOverlap(pub LockPath, pub Lock);
impl protos::Conceivable<datomic::Datom> for LockOverlap {
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
impl datomic::Datomic for LockOverlap {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Struct(fields) => {
                match <[datomic::Datom; 2]>::try_from(fields) {
                    Ok([d0, d1]) => {
                        match <LockPath as datomic::Datomic>::incorporate_from(d0) {
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                            Ok(p0) => {
                                match <Lock as datomic::Datomic>::incorporate_from(d1) {
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
impl protos::Incorporable<LockOverlap> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<LockOverlap, datomic::Fault> {
        <LockOverlap as datomic::Datomic>::incorporate_from(self)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LockRejection {
    DuplicateName(Lock),
    PathOverlap(LockOverlap),
}
impl protos::Conceivable<datomic::Datom> for LockRejection {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            match self {
                Self::DuplicateName(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("DuplicateName".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
                Self::PathOverlap(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("PathOverlap".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
            },
        )
    }
}
impl datomic::Datomic for LockRejection {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Variant(protos::Head::Bare(head), body) => {
                match head.as_str() {
                    "DuplicateName" => {
                        match <Lock as datomic::Datomic>::incorporate_from(*body) {
                            Ok(value) => Ok(Self::DuplicateName(value)),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    "PathOverlap" => {
                        match <LockOverlap as datomic::Datomic>::incorporate_from(
                            *body,
                        ) {
                            Ok(value) => Ok(Self::PathOverlap(value)),
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
impl protos::Incorporable<LockRejection> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<LockRejection, datomic::Fault> {
        <LockRejection as datomic::Datomic>::incorporate_from(self)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReleaseRejection {
    UnknownLockId,
}
impl protos::Conceivable<datomic::Datom> for ReleaseRejection {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            match self {
                Self::UnknownLockId => datomic::Datom::Bare("UnknownLockId".to_owned()),
            },
        )
    }
}
impl datomic::Datomic for ReleaseRejection {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Bare(symbol) => {
                match symbol.as_str() {
                    "UnknownLockId" => Ok(Self::UnknownLockId),
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
impl protos::Incorporable<ReleaseRejection> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<ReleaseRejection, datomic::Fault> {
        <ReleaseRejection as datomic::Datomic>::incorporate_from(self)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserveSelection {
    Locks,
}
impl protos::Conceivable<datomic::Datom> for ObserveSelection {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            match self {
                Self::Locks => datomic::Datom::Bare("Locks".to_owned()),
            },
        )
    }
}
impl datomic::Datomic for ObserveSelection {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Bare(symbol) => {
                match symbol.as_str() {
                    "Locks" => Ok(Self::Locks),
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
impl protos::Incorporable<ObserveSelection> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<ObserveSelection, datomic::Fault> {
        <ObserveSelection as datomic::Datomic>::incorporate_from(self)
    }
}
pub type Locks = Vec<Lock>;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Observation {
    Locks(Locks),
}
impl protos::Conceivable<datomic::Datom> for Observation {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            match self {
                Self::Locks(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Locks".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
            },
        )
    }
}
impl datomic::Datomic for Observation {
    fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
        match datom {
            datomic::Datom::Variant(protos::Head::Bare(head), body) => {
                match head.as_str() {
                    "Locks" => {
                        match <Locks as datomic::Datomic>::incorporate_from(*body) {
                            Ok(value) => Ok(Self::Locks(value)),
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
impl protos::Incorporable<Observation> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<Observation, datomic::Fault> {
        <Observation as datomic::Datomic>::incorporate_from(self)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Lock(LockRequest),
    Release(LockId),
    Observe(ObserveSelection),
}
impl protos::Conceivable<datomic::Datom> for Request {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            match self {
                Self::Lock(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Lock".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
                Self::Release(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Release".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
                Self::Observe(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Observe".to_owned()),
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
                    "Lock" => {
                        match <LockRequest as datomic::Datomic>::incorporate_from(
                            *body,
                        ) {
                            Ok(value) => Ok(Self::Lock(value)),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    "Release" => {
                        match <LockId as datomic::Datomic>::incorporate_from(*body) {
                            Ok(value) => Ok(Self::Release(value)),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    "Observe" => {
                        match <ObserveSelection as datomic::Datomic>::incorporate_from(
                            *body,
                        ) {
                            Ok(value) => Ok(Self::Observe(value)),
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
    Locked(Lock),
    LockRejected(LockRejection),
    Released(Lock),
    ReleaseRejected(ReleaseRejection),
    Observed(Observation),
}
impl protos::Conceivable<datomic::Datom> for Response {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
        Ok(
            match self {
                Self::Locked(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Locked".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
                Self::LockRejected(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("LockRejected".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
                Self::Released(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Released".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
                Self::ReleaseRejected(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("ReleaseRejected".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    )
                }
                Self::Observed(p0) => {
                    datomic::Datom::Variant(
                        protos::Head::Bare("Observed".to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
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
                    "Locked" => {
                        match <Lock as datomic::Datomic>::incorporate_from(*body) {
                            Ok(value) => Ok(Self::Locked(value)),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    "LockRejected" => {
                        match <LockRejection as datomic::Datomic>::incorporate_from(
                            *body,
                        ) {
                            Ok(value) => Ok(Self::LockRejected(value)),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    "Released" => {
                        match <Lock as datomic::Datomic>::incorporate_from(*body) {
                            Ok(value) => Ok(Self::Released(value)),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    "ReleaseRejected" => {
                        match <ReleaseRejection as datomic::Datomic>::incorporate_from(
                            *body,
                        ) {
                            Ok(value) => Ok(Self::ReleaseRejected(value)),
                            Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                        }
                    }
                    "Observed" => {
                        match <Observation as datomic::Datomic>::incorporate_from(
                            *body,
                        ) {
                            Ok(value) => Ok(Self::Observed(value)),
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
impl protos::Incorporable<Response> for datomic::Datom {
    type Fault = datomic::Fault;
    fn incorporate(self) -> Result<Response, datomic::Fault> {
        <Response as datomic::Datomic>::incorporate_from(self)
    }
}
