#![allow(dead_code)]
pub type LockId = protos::Integer;
pub type LockName = protos::Text;
pub type FlowId = protos::Text;
pub type LockPath = protos::Text;
pub type LockPaths = Vec<LockPath>;
pub type LockReason = protos::Text;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockRequest(pub LockName, pub FlowId, pub LockPaths, pub LockReason);
impl datom_codec::Datomic for LockRequest {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 4)?;
        let p0: LockName = datom_codec::Positional::position(&mut p)?;
        let p1: FlowId = datom_codec::Positional::position(&mut p)?;
        let p2: LockPaths = datom_codec::Positional::position(&mut p)?;
        let p3: LockReason = datom_codec::Positional::position(&mut p)?;
        Ok(Self(p0, p1, p2, p3))
    }
    fn conceive(&self) -> datom_codec::Datom {
        datom_codec::Datom::Struct(
            vec![
                datom_codec::Datomic::conceive(& self.0),
                datom_codec::Datomic::conceive(& self.1),
                datom_codec::Datomic::conceive(& self.2),
                datom_codec::Datomic::conceive(& self.3)
            ],
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lock(pub LockId, pub LockName, pub FlowId, pub LockPaths, pub LockReason);
impl datom_codec::Datomic for Lock {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 5)?;
        let p0: LockId = datom_codec::Positional::position(&mut p)?;
        let p1: LockName = datom_codec::Positional::position(&mut p)?;
        let p2: FlowId = datom_codec::Positional::position(&mut p)?;
        let p3: LockPaths = datom_codec::Positional::position(&mut p)?;
        let p4: LockReason = datom_codec::Positional::position(&mut p)?;
        Ok(Self(p0, p1, p2, p3, p4))
    }
    fn conceive(&self) -> datom_codec::Datom {
        datom_codec::Datom::Struct(
            vec![
                datom_codec::Datomic::conceive(& self.0),
                datom_codec::Datomic::conceive(& self.1),
                datom_codec::Datomic::conceive(& self.2),
                datom_codec::Datomic::conceive(& self.3),
                datom_codec::Datomic::conceive(& self.4)
            ],
        )
    }
}
pub type DuplicateName = Lock;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockOverlap(pub LockPath, pub Lock);
impl datom_codec::Datomic for LockOverlap {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 2)?;
        let p0: LockPath = datom_codec::Positional::position(&mut p)?;
        let p1: Lock = datom_codec::Positional::position(&mut p)?;
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
pub enum LockRejection {
    DuplicateName(Lock),
    PathOverlap(LockOverlap),
}
impl datom_codec::Datomic for LockRejection {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "DuplicateName" => Ok(Self::DuplicateName(datom_codec::Carrying::body(v)?)),
            "PathOverlap" => Ok(Self::PathOverlap(datom_codec::Carrying::body(v)?)),
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
            Self::DuplicateName(p0) => {
                datom_codec::Datom::Variant(
                    "DuplicateName".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::PathOverlap(p0) => {
                datom_codec::Datom::Variant(
                    "PathOverlap".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReleaseRejection {
    UnknownLockId,
}
impl datom_codec::Datomic for ReleaseRejection {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "UnknownLockId" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::UnknownLockId)
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
            Self::UnknownLockId => datom_codec::Datom::Word("UnknownLockId".to_owned()),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserveSelection {
    Locks,
}
impl datom_codec::Datomic for ObserveSelection {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Locks" => {
                datom_codec::Headed::nothing(v)?;
                Ok(Self::Locks)
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
            Self::Locks => datom_codec::Datom::Word("Locks".to_owned()),
        }
    }
}
pub type Locks = Vec<Lock>;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Observation {
    Locks(Locks),
}
impl datom_codec::Datomic for Observation {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Locks" => Ok(Self::Locks(datom_codec::Carrying::body(v)?)),
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
            Self::Locks(p0) => {
                datom_codec::Datom::Variant(
                    "Locks".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Lock(LockRequest),
    Release(LockId),
    Observe(ObserveSelection),
}
impl datom_codec::Datomic for Request {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Lock" => Ok(Self::Lock(datom_codec::Carrying::body(v)?)),
            "Release" => Ok(Self::Release(datom_codec::Carrying::body(v)?)),
            "Observe" => Ok(Self::Observe(datom_codec::Carrying::body(v)?)),
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
            Self::Lock(p0) => {
                datom_codec::Datom::Variant(
                    "Lock".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Release(p0) => {
                datom_codec::Datom::Variant(
                    "Release".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Observe(p0) => {
                datom_codec::Datom::Variant(
                    "Observe".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
        }
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
impl datom_codec::Datomic for Response {
    fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Locked" => Ok(Self::Locked(datom_codec::Carrying::body(v)?)),
            "LockRejected" => Ok(Self::LockRejected(datom_codec::Carrying::body(v)?)),
            "Released" => Ok(Self::Released(datom_codec::Carrying::body(v)?)),
            "ReleaseRejected" => {
                Ok(Self::ReleaseRejected(datom_codec::Carrying::body(v)?))
            }
            "Observed" => Ok(Self::Observed(datom_codec::Carrying::body(v)?)),
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
            Self::Locked(p0) => {
                datom_codec::Datom::Variant(
                    "Locked".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::LockRejected(p0) => {
                datom_codec::Datom::Variant(
                    "LockRejected".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Released(p0) => {
                datom_codec::Datom::Variant(
                    "Released".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::ReleaseRejected(p0) => {
                datom_codec::Datom::Variant(
                    "ReleaseRejected".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
            Self::Observed(p0) => {
                datom_codec::Datom::Variant(
                    "Observed".to_owned(),
                    Box::new(datom_codec::Datomic::conceive(p0)),
                )
            }
        }
    }
}
