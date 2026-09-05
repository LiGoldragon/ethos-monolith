#![allow(dead_code)]
pub type LockId = protos::Integer;
pub type LockName = protos::Text;
pub type FlowId = protos::Text;
pub type LockPath = protos::Text;
pub type LockPaths = std::vec::Vec<LockPath>;
pub type LockReason = protos::Text;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockRequest(pub LockName, pub FlowId, pub LockPaths, pub LockReason);
impl datom_codec::Datomic for LockRequest {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 4)?;
        let p0: LockName = datom_codec::Positional::position(&mut p)?;
        let p1: FlowId = datom_codec::Positional::position(&mut p)?;
        let p2: LockPaths = datom_codec::Positional::position(&mut p)?;
        let p3: LockReason = datom_codec::Positional::position(&mut p)?;
        std::result::Result::Ok(Self(p0, p1, p2, p3))
    }
}
impl protos::Conceivable<datom_codec::Datom> for LockRequest {
    type Fault = std::convert::Infallible;
    fn conceive(
        &self,
    ) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                datom_codec::Datom::Struct(
                    vec![
                        protos::Conceivable::conceive(& self.0)
                        .expect("infallible datom ascent").1,
                        protos::Conceivable::conceive(& self.1)
                        .expect("infallible datom ascent").1,
                        protos::Conceivable::conceive(& self.2)
                        .expect("infallible datom ascent").1,
                        protos::Conceivable::conceive(& self.3)
                        .expect("infallible datom ascent").1
                    ],
                ),
            ),
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lock(pub LockId, pub LockName, pub FlowId, pub LockPaths, pub LockReason);
impl datom_codec::Datomic for Lock {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 5)?;
        let p0: LockId = datom_codec::Positional::position(&mut p)?;
        let p1: LockName = datom_codec::Positional::position(&mut p)?;
        let p2: FlowId = datom_codec::Positional::position(&mut p)?;
        let p3: LockPaths = datom_codec::Positional::position(&mut p)?;
        let p4: LockReason = datom_codec::Positional::position(&mut p)?;
        std::result::Result::Ok(Self(p0, p1, p2, p3, p4))
    }
}
impl protos::Conceivable<datom_codec::Datom> for Lock {
    type Fault = std::convert::Infallible;
    fn conceive(
        &self,
    ) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                datom_codec::Datom::Struct(
                    vec![
                        protos::Conceivable::conceive(& self.0)
                        .expect("infallible datom ascent").1,
                        protos::Conceivable::conceive(& self.1)
                        .expect("infallible datom ascent").1,
                        protos::Conceivable::conceive(& self.2)
                        .expect("infallible datom ascent").1,
                        protos::Conceivable::conceive(& self.3)
                        .expect("infallible datom ascent").1,
                        protos::Conceivable::conceive(& self.4)
                        .expect("infallible datom ascent").1
                    ],
                ),
            ),
        )
    }
}
pub type DuplicateName = Lock;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockOverlap(pub LockPath, pub Lock);
impl datom_codec::Datomic for LockOverlap {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 2)?;
        let p0: LockPath = datom_codec::Positional::position(&mut p)?;
        let p1: Lock = datom_codec::Positional::position(&mut p)?;
        std::result::Result::Ok(Self(p0, p1))
    }
}
impl protos::Conceivable<datom_codec::Datom> for LockOverlap {
    type Fault = std::convert::Infallible;
    fn conceive(
        &self,
    ) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                datom_codec::Datom::Struct(
                    vec![
                        protos::Conceivable::conceive(& self.0)
                        .expect("infallible datom ascent").1,
                        protos::Conceivable::conceive(& self.1)
                        .expect("infallible datom ascent").1
                    ],
                ),
            ),
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LockRejection {
    DuplicateName(Lock),
    PathOverlap(LockOverlap),
}
impl datom_codec::Datomic for LockRejection {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "DuplicateName" => {
                std::result::Result::Ok(
                    Self::DuplicateName(datom_codec::Carrying::body(v)?),
                )
            }
            "PathOverlap" => {
                std::result::Result::Ok(
                    Self::PathOverlap(datom_codec::Carrying::body(v)?),
                )
            }
            _ => {
                std::result::Result::Err(
                    datom_codec::Headed::reject(
                        &v,
                        datom_codec::Problem::UnknownVariant(
                            protos::Word::try_from(v.name).expect("variant name"),
                        ),
                    ),
                )
            }
        }
    }
}
impl protos::Conceivable<datom_codec::Datom> for LockRejection {
    type Fault = std::convert::Infallible;
    fn conceive(
        &self,
    ) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                match self {
                    Self::DuplicateName(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("DuplicateName")
                                .expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::PathOverlap(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("PathOverlap")
                                .expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                },
            ),
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReleaseRejection {
    UnknownLockId,
}
impl datom_codec::Datomic for ReleaseRejection {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "UnknownLockId" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::UnknownLockId)
            }
            _ => {
                std::result::Result::Err(
                    datom_codec::Headed::reject(
                        &v,
                        datom_codec::Problem::UnknownVariant(
                            protos::Word::try_from(v.name).expect("variant name"),
                        ),
                    ),
                )
            }
        }
    }
}
impl protos::Conceivable<datom_codec::Datom> for ReleaseRejection {
    type Fault = std::convert::Infallible;
    fn conceive(
        &self,
    ) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                match self {
                    Self::UnknownLockId => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("UnknownLockId")
                                        .expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                },
            ),
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserveSelection {
    Locks,
}
impl datom_codec::Datomic for ObserveSelection {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Locks" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::Locks)
            }
            _ => {
                std::result::Result::Err(
                    datom_codec::Headed::reject(
                        &v,
                        datom_codec::Problem::UnknownVariant(
                            protos::Word::try_from(v.name).expect("variant name"),
                        ),
                    ),
                )
            }
        }
    }
}
impl protos::Conceivable<datom_codec::Datom> for ObserveSelection {
    type Fault = std::convert::Infallible;
    fn conceive(
        &self,
    ) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                match self {
                    Self::Locks => {
                        datom_codec::Datom::Word(
                            datom_codec::DatomWord::try_from(
                                    protos::Word::try_from("Locks").expect("static variant"),
                                )
                                .expect("stable variant"),
                        )
                    }
                },
            ),
        )
    }
}
pub type Locks = std::vec::Vec<Lock>;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Observation {
    Locks(Locks),
}
impl datom_codec::Datomic for Observation {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Locks" => {
                std::result::Result::Ok(Self::Locks(datom_codec::Carrying::body(v)?))
            }
            _ => {
                std::result::Result::Err(
                    datom_codec::Headed::reject(
                        &v,
                        datom_codec::Problem::UnknownVariant(
                            protos::Word::try_from(v.name).expect("variant name"),
                        ),
                    ),
                )
            }
        }
    }
}
impl protos::Conceivable<datom_codec::Datom> for Observation {
    type Fault = std::convert::Infallible;
    fn conceive(
        &self,
    ) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                match self {
                    Self::Locks(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Locks").expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                },
            ),
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Lock(LockRequest),
    Release(LockId),
    Observe(ObserveSelection),
}
impl datom_codec::Datomic for Request {
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Lock" => {
                std::result::Result::Ok(Self::Lock(datom_codec::Carrying::body(v)?))
            }
            "Release" => {
                std::result::Result::Ok(Self::Release(datom_codec::Carrying::body(v)?))
            }
            "Observe" => {
                std::result::Result::Ok(Self::Observe(datom_codec::Carrying::body(v)?))
            }
            _ => {
                std::result::Result::Err(
                    datom_codec::Headed::reject(
                        &v,
                        datom_codec::Problem::UnknownVariant(
                            protos::Word::try_from(v.name).expect("variant name"),
                        ),
                    ),
                )
            }
        }
    }
}
impl protos::Conceivable<datom_codec::Datom> for Request {
    type Fault = std::convert::Infallible;
    fn conceive(
        &self,
    ) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                match self {
                    Self::Lock(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Lock").expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Release(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Release").expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Observe(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Observe").expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                },
            ),
        )
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
    fn incorporate(
        site: datom_codec::Site<'_>,
    ) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Locked" => {
                std::result::Result::Ok(Self::Locked(datom_codec::Carrying::body(v)?))
            }
            "LockRejected" => {
                std::result::Result::Ok(
                    Self::LockRejected(datom_codec::Carrying::body(v)?),
                )
            }
            "Released" => {
                std::result::Result::Ok(Self::Released(datom_codec::Carrying::body(v)?))
            }
            "ReleaseRejected" => {
                std::result::Result::Ok(
                    Self::ReleaseRejected(datom_codec::Carrying::body(v)?),
                )
            }
            "Observed" => {
                std::result::Result::Ok(Self::Observed(datom_codec::Carrying::body(v)?))
            }
            _ => {
                std::result::Result::Err(
                    datom_codec::Headed::reject(
                        &v,
                        datom_codec::Problem::UnknownVariant(
                            protos::Word::try_from(v.name).expect("variant name"),
                        ),
                    ),
                )
            }
        }
    }
}
impl protos::Conceivable<datom_codec::Datom> for Response {
    type Fault = std::convert::Infallible;
    fn conceive(
        &self,
    ) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(
            protos::Situated(
                protos::Situation {
                    extent: protos::Extent(0, 0),
                    children: vec![],
                },
                match self {
                    Self::Locked(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Locked").expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::LockRejected(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("LockRejected")
                                .expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Released(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Released")
                                .expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::ReleaseRejected(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("ReleaseRejected")
                                .expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                    Self::Observed(p0) => {
                        datom_codec::Datom::Variant(
                            protos::Symbol::try_from("Observed")
                                .expect("static variant"),
                            std::boxed::Box::new(
                                protos::Conceivable::conceive(p0)
                                    .expect("infallible datom ascent")
                                    .1,
                            ),
                        )
                    }
                },
            ),
        )
    }
}
