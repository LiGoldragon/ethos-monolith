//! The durable, two-socket Ethos-zero Nexus runtime.
//!
//! The only owner of `sema_engine::Engine` is the store actor thread. Socket
//! workers and the two CLIs communicate with it through generated signal
//! values; they never obtain a storage handle.

use std::{
    collections::BTreeMap,
    env, fs,
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::{Component, Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    thread,
};

use datomic::{DatomicString, Text, TextEdge};
use ethos_zero::{FileLocation as EthosFileLocation, FileReader, Manifest, RustEmitter};
use meta::SignalFrameCodec as _;
use meta_signal_ethos_zero as meta;
use ordinary_signal::SignalFrameCodec as _;
use sema_engine::{
    Engine, EngineOpen, EngineRecord, FamilyName, QueryPlan, RecordKey, SchemaHash, SchemaVersion,
    TableDescriptor, TableName, TableReference,
};
use signal_ethos_zero as ordinary_signal;
use thiserror::Error;

const SCHEMA_VERSION: SchemaVersion = SchemaVersion::new(1);
const CONFIGURATION_TABLE: TableName = TableName::new("ethos_zero_nexus_configuration");
const ASSEMBLIES_TABLE: TableName = TableName::new("ethos_zero_nexus_assemblies");
const CONFIGURATION_KEY: &str = "configuration";
const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;
static NEXT_SESSION: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paths {
    pub state_path: PathBuf,
    pub ordinary_socket: PathBuf,
    pub meta_socket: PathBuf,
    pub source_manifest: PathBuf,
}

impl Paths {
    pub fn from_environment() -> Result<Self, RuntimeError> {
        let state_home = env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state")))
            .ok_or(RuntimeError::MissingHome)?;
        let runtime_root = env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| state_home.clone());
        Ok(Self::from_roots(state_home, runtime_root))
    }

    fn from_roots(state_home: PathBuf, runtime_root: PathBuf) -> Self {
        let state_root = state_home.join("ethos-zero-nexus");
        Self {
            state_path: state_root.join("ethos-zero-nexus.sema"),
            ordinary_socket: runtime_root.join("ethos-zero-nexus/ethos-zero.sock"),
            meta_socket: runtime_root.join("ethos-zero-nexus/meta-ethos-zero.sock"),
            source_manifest: state_root.join("sources.datom"),
        }
    }

    pub fn configuration(&self) -> Result<meta::Configuration, RuntimeError> {
        Ok(meta::Configuration {
            ordinary_socket_path: self.ordinary_socket.display().to_string().try_into()?,
            meta_socket_path: self.meta_socket.display().to_string().try_into()?,
            source_manifest_path: self.source_manifest.display().to_string().try_into()?,
        })
    }
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("HOME is required when XDG_STATE_HOME is unset")]
    MissingHome,
    #[error("filesystem: {0}")]
    Filesystem(#[from] std::io::Error),
    #[error("sema engine: {0}")]
    Engine(#[from] sema_engine::Error),
    #[error("a runtime string is not representable by Datomic")]
    String,
    #[error("the durable configuration singleton has {0} rows")]
    ConfigurationInvariant(usize),
    #[error("the actor mailbox closed")]
    ActorClosed,
}
impl From<datomic::UnrepresentableString> for RuntimeError {
    fn from(_: datomic::UnrepresentableString) -> Self {
        Self::String
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone)]
struct StoredConfiguration {
    configuration: meta::Configuration,
}
impl EngineRecord for StoredConfiguration {
    fn record_key(&self) -> RecordKey {
        RecordKey::new(CONFIGURATION_KEY)
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone)]
struct StoredAssembly {
    generation: ordinary_signal::Generation,
}
impl EngineRecord for StoredAssembly {
    fn record_key(&self) -> RecordKey {
        RecordKey::new(format!(
            "{}:{}",
            self.generation.file.source_name.as_ref(),
            self.generation.file.relative_path.as_ref()
        ))
    }
}

struct NexusStore {
    engine: Engine,
    configuration_table: TableReference<StoredConfiguration>,
    assemblies_table: TableReference<StoredAssembly>,
    configuration: meta::Configuration,
}

impl NexusStore {
    fn open(path: &Path, defaults: meta::Configuration) -> Result<Self, RuntimeError> {
        let parent = path.parent().expect("state path has a parent");
        fs::create_dir_all(parent)?;
        let mut engine = Engine::open(EngineOpen::new(path, SCHEMA_VERSION))?;
        let configuration_table = engine.register_table(TableDescriptor::new(
            CONFIGURATION_TABLE,
            FamilyName::new("ethos-zero-nexus-configuration"),
            SchemaHash::for_label("ethos-zero-nexus-configuration-v1"),
        ))?;
        let assemblies_table = engine.register_table(TableDescriptor::new(
            ASSEMBLIES_TABLE,
            FamilyName::new("ethos-zero-nexus-assembly-state"),
            SchemaHash::for_label("ethos-zero-nexus-assembly-state-v1"),
        ))?;
        let configuration = match engine
            .match_records(QueryPlan::all(configuration_table))?
            .records()
        {
            [] => {
                // Initial state is one atomic durable assertion, before this
                // Nexus accepts either socket.
                engine.commit_atomic(engine.begin_atomic_commit().assert(
                    configuration_table,
                    StoredConfiguration {
                        configuration: defaults.clone(),
                    },
                ))?;
                defaults
            }
            [stored] => stored.configuration.clone(),
            rows => return Err(RuntimeError::ConfigurationInvariant(rows.len())),
        };
        Ok(Self {
            engine,
            configuration_table,
            assemblies_table,
            configuration,
        })
    }

    fn assemblies(&self) -> Result<Vec<ordinary_signal::AssemblySummary>, RuntimeError> {
        let mut rows: Vec<_> = self
            .engine
            .match_records(QueryPlan::all(self.assemblies_table))?
            .records()
            .iter()
            .map(|stored| ordinary_signal::AssemblySummary {
                file: stored.generation.file.clone(),
                artifact: stored.generation.artifact.clone(),
            })
            .collect();
        rows.sort_by(|left, right| {
            left.file
                .source_name
                .as_ref()
                .cmp(right.file.source_name.as_ref())
                .then_with(|| {
                    left.file
                        .relative_path
                        .as_ref()
                        .cmp(right.file.relative_path.as_ref())
                })
        });
        Ok(rows)
    }

    fn ordinary_observation(&self) -> Result<ordinary_signal::Observation, RuntimeError> {
        Ok(ordinary_signal::Observation::Assemblies(
            ordinary_signal::AssemblySnapshot {
                assemblies: ordinary_signal::Assemblies(self.assemblies()?),
            },
        ))
    }

    fn source_map(&self) -> Result<SourceMap, RuntimeError> {
        let text = fs::read_to_string(self.configuration.source_manifest_path.as_ref())?;
        SourceMap::from_datomic(&text).map_err(|_| {
            RuntimeError::Filesystem(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "source manifest is not a Datomic map",
            ))
        })
    }

    fn source_index(&self) -> Result<meta::SourceIndex, RuntimeError> {
        let sources = self
            .source_map()?
            .roots
            .into_iter()
            .map(|(name, path)| -> Result<meta::Source, RuntimeError> {
                Ok(meta::Source {
                    source_name: name.try_into()?,
                    relative_path: path.display().to_string().try_into()?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(meta::SourceIndex {
            sources: meta::Sources(sources),
        })
    }

    fn configure(&mut self, value: meta::Configuration) -> Result<meta::Reply, RuntimeError> {
        if value.ordinary_socket_path.as_ref().is_empty() {
            return Ok(meta::Reply::ConfigurationRejected(
                meta::ConfigurationRefusal::InvalidOrdinarySocketPath,
            ));
        }
        if value.meta_socket_path.as_ref().is_empty() {
            return Ok(meta::Reply::ConfigurationRejected(
                meta::ConfigurationRefusal::InvalidMetaSocketPath,
            ));
        }
        if value.source_manifest_path.as_ref().is_empty() {
            return Ok(meta::Reply::ConfigurationRejected(
                meta::ConfigurationRefusal::InvalidSourceManifestPath,
            ));
        }
        // Listener ownership lives outside the store actor. Until listeners
        // can be replaced atomically, accepting a changed path would persist
        // a lie: the running Nexus would still serve the old socket.
        if value.ordinary_socket_path != self.configuration.ordinary_socket_path {
            return Ok(meta::Reply::ConfigurationRejected(
                meta::ConfigurationRefusal::InvalidOrdinarySocketPath,
            ));
        }
        if value.meta_socket_path != self.configuration.meta_socket_path {
            return Ok(meta::Reply::ConfigurationRejected(
                meta::ConfigurationRefusal::InvalidMetaSocketPath,
            ));
        }
        self.engine
            .commit_atomic(self.engine.begin_atomic_commit().mutate(
                self.configuration_table,
                StoredConfiguration {
                    configuration: value.clone(),
                },
            ))?;
        self.configuration = value.clone();
        Ok(meta::Reply::Configured(value))
    }

    fn generate(
        &mut self,
        request: ordinary_signal::GenerationRequest,
    ) -> Result<ordinary_signal::Reply, RuntimeError> {
        let relative = request.file.relative_path.as_ref();
        let Some(relative_path) = contained_relative(relative) else {
            return Ok(ordinary_signal::Reply::GenerationRejected(
                ordinary_signal::GenerationRefusal::InvalidRelativePath(request.file.relative_path),
            ));
        };
        let sources = match self.source_map() {
            Ok(value) => value,
            Err(_) => {
                return Ok(ordinary_signal::Reply::GenerationRejected(
                    ordinary_signal::GenerationRefusal::ImportUnresolved(request.file),
                ));
            }
        };
        let Some(root) = sources.roots.get(request.file.source_name.as_ref()) else {
            return Ok(ordinary_signal::Reply::GenerationRejected(
                ordinary_signal::GenerationRefusal::UnknownSource(request.file.source_name),
            ));
        };
        let Some(source_path) = contained_existing(root, &relative_path) else {
            return Ok(ordinary_signal::Reply::GenerationRejected(
                ordinary_signal::GenerationRefusal::InvalidRelativePath(request.file.relative_path),
            ));
        };
        let source = match fs::read_to_string(&source_path) {
            Ok(source) => source,
            Err(_) => {
                return Ok(ordinary_signal::Reply::GenerationRejected(
                    ordinary_signal::GenerationRefusal::FileAbsent(request.file),
                ));
            }
        };
        let reader = FileReader::new(&sources);
        let file = match reader.read(&source) {
            Ok(file) => file,
            Err(error) => {
                return Ok(ordinary_signal::Reply::GenerationRejected(
                    ordinary_signal::GenerationRefusal::InvalidEthos(syntax_fault(&format!(
                        "{error}"
                    ))?),
                ));
            }
        };
        let generated = match RustEmitter::wire_contract().emit(&file) {
            Ok(generated) => generated,
            Err(error) => {
                return Ok(ordinary_signal::Reply::GenerationRejected(
                    ordinary_signal::GenerationRefusal::RustProjectionRejected(projection_fault(
                        &format!("{error}"),
                    )?),
                ));
            }
        };
        let artifact_path = relative_path.with_extension("rs");
        let Some(destination) = contained_destination(root, &artifact_path) else {
            return Ok(ordinary_signal::Reply::GenerationRejected(
                ordinary_signal::GenerationRefusal::InvalidRelativePath(request.file.relative_path),
            ));
        };
        // The source artifact reaches the filesystem before its durable state
        // transition. A process crash can therefore be repaired by a later
        // Generate; it can never claim a stored generation whose artifact was
        // not first written.
        fs::write(&destination, generated)?;
        let artifact: ordinary_signal::ArtifactPath =
            artifact_path.display().to_string().try_into()?;
        let generation = ordinary_signal::Generation {
            file: request.file,
            artifact,
        };
        let stored = StoredAssembly {
            generation: generation.clone(),
        };
        let exists = !self
            .engine
            .match_records(QueryPlan::key(self.assemblies_table, stored.record_key()))?
            .records()
            .is_empty();
        let commit = if exists {
            self.engine
                .begin_atomic_commit()
                .mutate(self.assemblies_table, stored)
        } else {
            self.engine
                .begin_atomic_commit()
                .assert(self.assemblies_table, stored)
        };
        self.engine.commit_atomic(commit)?;
        Ok(ordinary_signal::Reply::Generated(generation))
    }
}

struct SourceMap {
    roots: BTreeMap<String, PathBuf>,
}
impl SourceMap {
    fn from_datomic(text: &str) -> Result<Self, datomic::Fault> {
        let encoded = Text::<BTreeMap<DatomicString, DatomicString>>::from(text);
        let map = encoded.embody()?;
        Ok(Self {
            roots: map
                .into_iter()
                .map(|(name, path)| (name.as_ref().to_owned(), PathBuf::from(path.as_ref())))
                .collect(),
        })
    }
}
impl Manifest for SourceMap {
    fn resolve(&self, source: &str) -> Option<EthosFileLocation> {
        let root = self.roots.get(source)?;
        Some(EthosFileLocation {
            directory: root.display().to_string(),
            file: String::new(),
        })
    }
}

fn contained_relative(value: &str) -> Option<PathBuf> {
    let path = Path::new(value);
    if path.is_absolute() || path.as_os_str().is_empty() {
        return None;
    }
    path.components()
        .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
        .then(|| path.to_path_buf())
}

fn contained_existing(root: &Path, relative: &Path) -> Option<PathBuf> {
    let root = fs::canonicalize(root).ok()?;
    let candidate = fs::canonicalize(root.join(relative)).ok()?;
    candidate.starts_with(&root).then_some(candidate)
}

fn contained_destination(root: &Path, relative: &Path) -> Option<PathBuf> {
    let root = fs::canonicalize(root).ok()?;
    let mut destination = root.clone();
    let mut components = relative.components().peekable();
    while let Some(component) = components.next() {
        let Component::Normal(name) = component else {
            return None;
        };
        let next = destination.join(name);
        if components.peek().is_none() {
            if next.exists() {
                let canonical = fs::canonicalize(&next).ok()?;
                return canonical.starts_with(&root).then_some(canonical);
            }
            return Some(next);
        }
        match fs::symlink_metadata(&next) {
            Ok(metadata) if metadata.file_type().is_symlink() => return None,
            Ok(metadata) if metadata.is_dir() => destination = fs::canonicalize(next).ok()?,
            Ok(_) => return None,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&next).ok()?;
                destination = next;
            }
            Err(_) => return None,
        }
        if !destination.starts_with(&root) {
            return None;
        }
    }
    None
}

fn syntax_fault(reason: &str) -> Result<ordinary_signal::SyntaxFault, RuntimeError> {
    Ok(ordinary_signal::SyntaxFault {
        extent: ordinary_signal::SourceExtent {
            extent_start: ordinary_signal::ExtentStart(0),
            extent_end: ordinary_signal::ExtentEnd(0),
        },
        reason: reason.to_owned().try_into()?,
    })
}
fn projection_fault(reason: &str) -> Result<ordinary_signal::ProjectionFault, RuntimeError> {
    Ok(ordinary_signal::ProjectionFault {
        reason: reason.to_owned().try_into()?,
    })
}

enum Command {
    Ordinary {
        session: u64,
        request: ordinary_signal::Request,
        events: Option<mpsc::Sender<ordinary_signal::Stream>>,
        reply: mpsc::Sender<Result<OrdinaryAnswer, RuntimeError>>,
    },
    Meta {
        session: u64,
        request: meta::Request,
        events: Option<mpsc::Sender<meta::Stream>>,
        reply: mpsc::Sender<Result<meta::Reply, RuntimeError>>,
    },
}

enum OrdinaryAnswer {
    Reply(ordinary_signal::Reply),
    Refusal(ordinary_signal::Refusal),
}

#[derive(Clone)]
struct Actor {
    sender: mpsc::Sender<Command>,
}
impl Actor {
    fn start(paths: &Paths) -> Result<Self, RuntimeError> {
        let defaults = paths.configuration()?;
        let mut store = NexusStore::open(&paths.state_path, defaults)?;
        let (sender, receiver) = mpsc::channel::<Command>();
        thread::spawn(move || {
            let mut ordinary_subscribers: Vec<(
                u64,
                ordinary_signal::SubscriptionRequest,
                mpsc::Sender<ordinary_signal::Stream>,
            )> = Vec::new();
            let mut meta_subscribers: Vec<(
                u64,
                meta::MetaSubscriptionRequest,
                mpsc::Sender<meta::Stream>,
            )> = Vec::new();
            while let Ok(command) = receiver.recv() {
                match command {
                    Command::Ordinary {
                        session,
                        request,
                        events,
                        reply,
                    } => {
                        let result = match request {
                            ordinary_signal::Request::Observe(_) => store
                                .ordinary_observation()
                                .map(ordinary_signal::Reply::Observed)
                                .map(OrdinaryAnswer::Reply),
                            ordinary_signal::Request::Subscribe(subscription) => {
                                if let Some(events) = events {
                                    ordinary_subscribers.push((session, subscription, events));
                                }
                                store
                                    .ordinary_observation()
                                    .map(ordinary_signal::Reply::Observed)
                                    .map(OrdinaryAnswer::Reply)
                            }
                            ordinary_signal::Request::Unsubscribe(subscription) => {
                                ordinary_subscribers.retain(|(owner, current, _)| {
                                    *owner != session || current != &subscription
                                });
                                store
                                    .ordinary_observation()
                                    .map(ordinary_signal::Reply::Observed)
                                    .map(OrdinaryAnswer::Reply)
                            }
                            ordinary_signal::Request::Generate(request) => {
                                let requested_file = request.file.clone();
                                if contained_relative(request.file.relative_path.as_ref()).is_none()
                                {
                                    let refusal = ordinary_signal::Refusal::InvalidRelativePath(
                                        request.file.relative_path,
                                    );
                                    let _ = reply.send(Ok(OrdinaryAnswer::Refusal(refusal)));
                                    continue;
                                }
                                let started =
                                    ordinary_signal::Stream::GenerationStarted(request.clone());
                                ordinary_subscribers.retain(|(_, current, sender)| {
                                    current.file != requested_file
                                        || sender.send(started.clone()).is_ok()
                                });
                                let result = store.generate(request).map(OrdinaryAnswer::Reply);
                                if let Ok(OrdinaryAnswer::Reply(reply_value)) = &result {
                                    let event = match reply_value {
                                        ordinary_signal::Reply::Generated(generation) => {
                                            ordinary_signal::Stream::GenerationCompleted(
                                                generation.clone(),
                                            )
                                        }
                                        ordinary_signal::Reply::GenerationRejected(refusal) => {
                                            ordinary_signal::Stream::GenerationRefused(
                                                refusal.clone(),
                                            )
                                        }
                                        ordinary_signal::Reply::Observed(_) => {
                                            unreachable!("Generate has a generation reply")
                                        }
                                    };
                                    ordinary_subscribers.retain(|(_, current, sender)| {
                                        current.file != requested_file
                                            || sender.send(event.clone()).is_ok()
                                    });
                                }
                                result
                            }
                        };
                        let _ = reply.send(result);
                    }
                    Command::Meta {
                        session,
                        request,
                        events,
                        reply,
                    } => {
                        let result = match request {
                            meta::Request::Configure(configuration) => {
                                let result = store.configure(configuration);
                                if let Ok(meta::Reply::Configured(configuration)) = &result {
                                    let event =
                                        meta::Stream::ConfigurationChanged(configuration.clone());
                                    meta_subscribers.retain(|(_, subscription, sender)| {
                                        !matches!(
                                            subscription.selection,
                                            meta::MetaObservationSelection::Configuration
                                        ) || sender.send(event.clone()).is_ok()
                                    });
                                    if let Ok(index) = store.source_index() {
                                        let event = meta::Stream::SourcesChanged(index);
                                        meta_subscribers.retain(|(_, subscription, sender)| {
                                            !matches!(
                                                subscription.selection,
                                                meta::MetaObservationSelection::Sources
                                            ) || sender.send(event.clone()).is_ok()
                                        });
                                    }
                                }
                                result
                            }
                            meta::Request::Observe(selection) => match selection {
                                meta::MetaObservationSelection::Configuration => {
                                    Ok(meta::Reply::Observed(meta::MetaObservation::Configuration(
                                        store.configuration.clone(),
                                    )))
                                }
                                meta::MetaObservationSelection::Sources => {
                                    match store.source_index() {
                                        Ok(index) => Ok(meta::Reply::Observed(
                                            meta::MetaObservation::Sources(index),
                                        )),
                                        Err(_) => Ok(meta::Reply::ConfigurationRejected(
                                            meta::ConfigurationRefusal::UnreadableSourceManifest,
                                        )),
                                    }
                                }
                            },
                            meta::Request::Subscribe(subscription) => {
                                let selection = subscription.selection.clone();
                                if let Some(events) = events {
                                    meta_subscribers.push((session, subscription, events));
                                }
                                match selection {
                                    meta::MetaObservationSelection::Configuration => {
                                        Ok(meta::Reply::Observed(
                                            meta::MetaObservation::Configuration(
                                                store.configuration.clone(),
                                            ),
                                        ))
                                    }
                                    meta::MetaObservationSelection::Sources => match store
                                        .source_index()
                                    {
                                        Ok(index) => Ok(meta::Reply::Observed(
                                            meta::MetaObservation::Sources(index),
                                        )),
                                        Err(_) => Ok(meta::Reply::ConfigurationRejected(
                                            meta::ConfigurationRefusal::UnreadableSourceManifest,
                                        )),
                                    },
                                }
                            }
                            meta::Request::Unsubscribe(subscription) => {
                                meta_subscribers.retain(|(owner, current, _)| {
                                    *owner != session || current != &subscription
                                });
                                Ok(meta::Reply::Observed(meta::MetaObservation::Configuration(
                                    store.configuration.clone(),
                                )))
                            }
                        };
                        let _ = reply.send(result);
                    }
                }
            }
        });
        Ok(Self { sender })
    }

    #[cfg(test)]
    fn ordinary(
        &self,
        request: ordinary_signal::Request,
        events: Option<mpsc::Sender<ordinary_signal::Stream>>,
    ) -> Result<OrdinaryAnswer, RuntimeError> {
        self.ordinary_session(0, request, events)
    }
    fn ordinary_session(
        &self,
        session: u64,
        request: ordinary_signal::Request,
        events: Option<mpsc::Sender<ordinary_signal::Stream>>,
    ) -> Result<OrdinaryAnswer, RuntimeError> {
        let (reply, receive) = mpsc::channel();
        self.sender
            .send(Command::Ordinary {
                session,
                request,
                events,
                reply,
            })
            .map_err(|_| RuntimeError::ActorClosed)?;
        receive.recv().map_err(|_| RuntimeError::ActorClosed)?
    }
    #[cfg(test)]
    fn meta(
        &self,
        request: meta::Request,
        events: Option<mpsc::Sender<meta::Stream>>,
    ) -> Result<meta::Reply, RuntimeError> {
        self.meta_session(0, request, events)
    }
    fn meta_session(
        &self,
        session: u64,
        request: meta::Request,
        events: Option<mpsc::Sender<meta::Stream>>,
    ) -> Result<meta::Reply, RuntimeError> {
        let (reply, receive) = mpsc::channel();
        self.sender
            .send(Command::Meta {
                session,
                request,
                events,
                reply,
            })
            .map_err(|_| RuntimeError::ActorClosed)?;
        receive.recv().map_err(|_| RuntimeError::ActorClosed)?
    }
}

fn ordinary_frame(body: ordinary_signal::FrameBody) -> ordinary_signal::Frame {
    ordinary_signal::Frame {
        channel_contract_id: ordinary_signal::CHANNEL_CONTRACT_ID,
        channel_wire_revision: ordinary_signal::CHANNEL_WIRE_REVISION,
        protocol_version: ordinary_signal::PROTOCOL_VERSION,
        body,
    }
}
fn meta_frame(body: meta::FrameBody) -> meta::Frame {
    meta::Frame {
        channel_contract_id: meta::CHANNEL_CONTRACT_ID,
        channel_wire_revision: meta::CHANNEL_WIRE_REVISION,
        protocol_version: meta::PROTOCOL_VERSION,
        body,
    }
}

fn read_frame(stream: &mut UnixStream) -> std::io::Result<Vec<u8>> {
    let mut prefix = [0; 4];
    stream.read_exact(&mut prefix)?;
    let length = frame_length(prefix)?;
    let mut bytes = vec![0; length + 4];
    bytes[..4].copy_from_slice(&prefix);
    stream.read_exact(&mut bytes[4..])?;
    Ok(bytes)
}

fn frame_length(prefix: [u8; 4]) -> std::io::Result<usize> {
    let length = u32::from_le_bytes(prefix) as usize;
    (length <= MAX_FRAME_BYTES)
        .then_some(length)
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "frame exceeds maximum length",
            )
        })
}

fn bind(path: &Path) -> Result<UnixListener, RuntimeError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(UnixListener::bind(path)?)
}

fn ordinary_socket(mut stream: UnixStream, actor: Actor) {
    let session = NEXT_SESSION.fetch_add(1, Ordering::Relaxed);
    let request = read_frame(&mut stream)
        .ok()
        .and_then(|bytes| ordinary_signal::Frame::decode_length_prefixed(&bytes).ok())
        .and_then(|frame| match frame.body {
            ordinary_signal::FrameBody::Request(value) => Some(value),
            _ => None,
        });
    let Some(request) = request else {
        return;
    };
    let subscribe = matches!(request, ordinary_signal::Request::Subscribe(_));
    let (events, receiver) = mpsc::channel();
    let reply = actor.ordinary_session(session, request, subscribe.then_some(events));
    let body = match reply {
        Ok(OrdinaryAnswer::Reply(value)) => ordinary_signal::FrameBody::Reply(value),
        Ok(OrdinaryAnswer::Refusal(value)) => ordinary_signal::FrameBody::Refusal(value),
        Err(_) => return,
    };
    let Ok(bytes) = ordinary_frame(body).encode_length_prefixed() else {
        return;
    };
    if stream.write_all(&bytes).is_err() {
        return;
    }
    if subscribe {
        for event in receiver {
            let Ok(bytes) =
                ordinary_frame(ordinary_signal::FrameBody::Event(event)).encode_length_prefixed()
            else {
                break;
            };
            if stream.write_all(&bytes).is_err() {
                break;
            }
        }
    }
}
fn meta_socket(mut stream: UnixStream, actor: Actor) {
    let session = NEXT_SESSION.fetch_add(1, Ordering::Relaxed);
    let request = read_frame(&mut stream)
        .ok()
        .and_then(|bytes| meta::Frame::decode_length_prefixed(&bytes).ok())
        .and_then(|frame| match frame.body {
            meta::FrameBody::Request(value) => Some(value),
            _ => None,
        });
    let Some(request) = request else {
        return;
    };
    let subscribe = matches!(request, meta::Request::Subscribe(_));
    let (events, receiver) = mpsc::channel();
    let reply = actor.meta_session(session, request, subscribe.then_some(events));
    let body = match reply {
        Ok(value) => meta::FrameBody::Reply(value),
        Err(_) => return,
    };
    let Ok(bytes) = meta_frame(body).encode_length_prefixed() else {
        return;
    };
    if stream.write_all(&bytes).is_err() {
        return;
    }
    if subscribe {
        for event in receiver {
            let Ok(bytes) = meta_frame(meta::FrameBody::Event(event)).encode_length_prefixed()
            else {
                break;
            };
            if stream.write_all(&bytes).is_err() {
                break;
            }
        }
    }
}

pub fn run(paths: Paths) -> Result<(), RuntimeError> {
    let actor = Actor::start(&paths)?;
    let ordinary = bind(&paths.ordinary_socket)?;
    let meta = bind(&paths.meta_socket)?;
    let meta_actor = actor.clone();
    thread::spawn(move || {
        for stream in meta.incoming().flatten() {
            let actor = meta_actor.clone();
            thread::spawn(move || meta_socket(stream, actor));
        }
    });
    for stream in ordinary.incoming().flatten() {
        let actor = actor.clone();
        thread::spawn(move || ordinary_socket(stream, actor));
    }
    Ok(())
}

pub fn run_default() -> Result<(), RuntimeError> {
    run(Paths::from_environment()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use datomic::Datomic;

    fn paths(root: &Path) -> Paths {
        Paths {
            state_path: root.join("state/nexus.sema"),
            ordinary_socket: root.join("runtime/ethos-zero.sock"),
            meta_socket: root.join("runtime/meta-ethos-zero.sock"),
            source_manifest: root.join("state/sources.datom"),
        }
    }

    fn request(path: &str) -> ordinary_signal::GenerationRequest {
        ordinary_signal::GenerationRequest {
            file: ordinary_signal::FileLocation {
                source_name: "workspace".try_into().expect("representable source"),
                relative_path: path.try_into().expect("representable relative path"),
            },
        }
    }

    fn write_manifest(paths: &Paths, source_root: &Path) {
        let mut sources = BTreeMap::new();
        sources.insert(
            DatomicString::try_from("workspace".to_owned()).expect("source key"),
            DatomicString::try_from(source_root.display().to_string()).expect("source root"),
        );
        fs::create_dir_all(paths.source_manifest.parent().expect("state parent"))
            .expect("state directory");
        fs::write(&paths.source_manifest, sources.textualize().as_ref()).expect("source manifest");
    }

    fn write_interface(source_root: &Path) {
        fs::write(
            source_root.join("example.ethos"),
            "Interface.{0 3 0}\nChannel.{Example 1 3}\n[]\n{[Observe.Selection][Observed.Snapshot][][][Selection.[State] Value.String Snapshot.{Value}]}",
        ).expect("interface fixture");
    }

    #[test]
    fn configuration_reopens_from_the_singleton_family() {
        let directory = tempfile::tempdir().expect("temporary root");
        let paths = paths(directory.path());
        let defaults = paths.configuration().expect("defaults");
        let mut store = NexusStore::open(&paths.state_path, defaults.clone()).expect("open store");
        let changed = meta::Configuration {
            ordinary_socket_path: defaults.ordinary_socket_path.clone(),
            meta_socket_path: defaults.meta_socket_path.clone(),
            source_manifest_path: directory
                .path()
                .join("other-sources.datom")
                .display()
                .to_string()
                .try_into()
                .expect("manifest path"),
        };
        assert!(matches!(
            store.configure(changed.clone()).expect("configure"),
            meta::Reply::Configured(_)
        ));
        drop(store);
        let reopened = NexusStore::open(&paths.state_path, defaults).expect("reopen store");
        assert_eq!(reopened.configuration, changed);
    }

    #[test]
    fn state_fallback_uses_one_nexus_directory_and_frames_are_bounded() {
        let paths = Paths::from_roots(PathBuf::from("/state"), PathBuf::from("/state"));
        assert_eq!(
            paths.ordinary_socket,
            PathBuf::from("/state/ethos-zero-nexus/ethos-zero.sock")
        );
        assert!(frame_length((MAX_FRAME_BYTES as u32).to_le_bytes()).is_ok());
        assert!(frame_length(((MAX_FRAME_BYTES as u32) + 1).to_le_bytes()).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_source_and_artifact_paths_are_typed_containment_refusals() {
        use std::os::unix::fs::symlink;
        let directory = tempfile::tempdir().expect("temporary root");
        let paths = paths(directory.path());
        let source = directory.path().join("source");
        let outside = directory.path().join("outside");
        fs::create_dir_all(&source).expect("source root");
        fs::create_dir_all(&outside).expect("outside root");
        fs::write(
            outside.join("escape.ethos"),
            "Interface.{0 1 0} Channel.{Example 1 1} [] {[][][][][]}",
        )
        .expect("outside source");
        symlink(outside.join("escape.ethos"), source.join("escape.ethos")).expect("source link");
        write_manifest(&paths, &source);
        let actor = Actor::start(&paths).expect("actor");
        assert!(matches!(
            actor
                .ordinary(
                    ordinary_signal::Request::Generate(request("escape.ethos")),
                    None
                )
                .expect("refusal"),
            OrdinaryAnswer::Reply(ordinary_signal::Reply::GenerationRejected(
                ordinary_signal::GenerationRefusal::InvalidRelativePath(_)
            ))
        ));
    }

    #[test]
    fn actor_serializes_requests_and_refuses_an_escaping_relative_path() {
        let directory = tempfile::tempdir().expect("temporary root");
        let paths = paths(directory.path());
        write_manifest(&paths, directory.path());
        let actor = Actor::start(&paths).expect("actor");
        let first = actor.clone();
        let second = actor.clone();
        let first = thread::spawn(move || {
            first.ordinary(
                ordinary_signal::Request::Observe(
                    ordinary_signal::ObservationSelection::Assemblies,
                ),
                None,
            )
        });
        let second = thread::spawn(move || {
            second.ordinary(
                ordinary_signal::Request::Observe(
                    ordinary_signal::ObservationSelection::Assemblies,
                ),
                None,
            )
        });
        assert!(matches!(
            first
                .join()
                .expect("first actor call")
                .expect("first reply"),
            OrdinaryAnswer::Reply(ordinary_signal::Reply::Observed(_))
        ));
        assert!(matches!(
            second
                .join()
                .expect("second actor call")
                .expect("second reply"),
            OrdinaryAnswer::Reply(ordinary_signal::Reply::Observed(_))
        ));
        let reply = actor
            .ordinary(
                ordinary_signal::Request::Generate(request("../outside.ethos")),
                None,
            )
            .expect("typed refusal reply");
        assert!(matches!(
            reply,
            OrdinaryAnswer::Refusal(ordinary_signal::Refusal::InvalidRelativePath(_))
        ));
    }

    #[test]
    fn subscription_receives_started_then_completed_after_file_write_and_store_mutation() {
        let directory = tempfile::tempdir().expect("temporary root");
        let paths = paths(directory.path());
        write_manifest(&paths, directory.path());
        write_interface(directory.path());
        let actor = Actor::start(&paths).expect("actor");
        let generation = request("example.ethos");
        let subscription = ordinary_signal::SubscriptionRequest {
            file: generation.file.clone(),
        };
        let (events, received) = mpsc::channel();
        assert!(matches!(
            actor
                .ordinary(
                    ordinary_signal::Request::Subscribe(subscription),
                    Some(events)
                )
                .expect("subscribe"),
            OrdinaryAnswer::Reply(ordinary_signal::Reply::Observed(_))
        ));
        assert!(matches!(
            actor
                .ordinary(ordinary_signal::Request::Generate(generation), None)
                .expect("generate"),
            OrdinaryAnswer::Reply(ordinary_signal::Reply::Generated(_))
        ));
        assert!(matches!(
            received.recv().expect("started event"),
            ordinary_signal::Stream::GenerationStarted(_)
        ));
        assert!(matches!(
            received.recv().expect("completed event"),
            ordinary_signal::Stream::GenerationCompleted(_)
        ));
        assert!(
            directory.path().join("example.rs").is_file(),
            "generation writes before it is recorded"
        );
    }

    #[test]
    fn subscriptions_filter_by_file_and_unsubscribe_only_their_session() {
        let directory = tempfile::tempdir().expect("temporary root");
        let paths = paths(directory.path());
        write_manifest(&paths, directory.path());
        write_interface(directory.path());
        let actor = Actor::start(&paths).expect("actor");
        let matching = ordinary_signal::SubscriptionRequest {
            file: request("example.ethos").file,
        };
        let other = ordinary_signal::SubscriptionRequest {
            file: request("other.ethos").file,
        };
        let (first_send, first_events) = mpsc::channel();
        let (other_send, other_events) = mpsc::channel();
        actor
            .ordinary_session(
                1,
                ordinary_signal::Request::Subscribe(matching.clone()),
                Some(first_send),
            )
            .expect("first subscribe");
        actor
            .ordinary_session(
                2,
                ordinary_signal::Request::Subscribe(other),
                Some(other_send),
            )
            .expect("other subscribe");
        actor
            .ordinary_session(
                3,
                ordinary_signal::Request::Unsubscribe(matching.clone()),
                None,
            )
            .expect("foreign unsubscribe");
        actor
            .ordinary(
                ordinary_signal::Request::Generate(request("example.ethos")),
                None,
            )
            .expect("generate");
        assert!(matches!(
            first_events.recv().expect("matching start"),
            ordinary_signal::Stream::GenerationStarted(_)
        ));
        assert!(matches!(
            first_events.recv().expect("matching completion"),
            ordinary_signal::Stream::GenerationCompleted(_)
        ));
        assert!(matches!(
            other_events.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
        actor
            .ordinary_session(1, ordinary_signal::Request::Unsubscribe(matching), None)
            .expect("owned unsubscribe");
        actor
            .ordinary(
                ordinary_signal::Request::Generate(request("example.ethos")),
                None,
            )
            .expect("regenerate");
        assert!(matches!(
            first_events.try_recv(),
            Err(mpsc::TryRecvError::Disconnected)
        ));
    }

    #[test]
    fn meta_subscriptions_are_owned_by_their_sessions() {
        let directory = tempfile::tempdir().expect("temporary root");
        let paths = paths(directory.path());
        write_manifest(&paths, directory.path());
        let actor = Actor::start(&paths).expect("actor");
        let subscription = meta::MetaSubscriptionRequest {
            selection: meta::MetaObservationSelection::Configuration,
        };
        let (first_send, first_events) = mpsc::channel();
        let (second_send, second_events) = mpsc::channel();
        actor
            .meta_session(
                1,
                meta::Request::Subscribe(subscription.clone()),
                Some(first_send),
            )
            .expect("first subscription");
        actor
            .meta_session(
                2,
                meta::Request::Subscribe(subscription.clone()),
                Some(second_send),
            )
            .expect("second subscription");
        actor
            .meta_session(3, meta::Request::Unsubscribe(subscription.clone()), None)
            .expect("foreign unsubscribe");
        let configuration = paths.configuration().expect("defaults");
        actor
            .meta(meta::Request::Configure(configuration.clone()), None)
            .expect("configuration");
        assert!(matches!(
            first_events.recv().expect("first event"),
            meta::Stream::ConfigurationChanged(value) if value == configuration
        ));
        assert!(matches!(
            second_events.recv().expect("second event"),
            meta::Stream::ConfigurationChanged(value) if value == configuration
        ));
        actor
            .meta_session(1, meta::Request::Unsubscribe(subscription), None)
            .expect("owned unsubscribe");
        actor
            .meta(meta::Request::Configure(configuration.clone()), None)
            .expect("configuration again");
        assert!(matches!(
            first_events.try_recv(),
            Err(mpsc::TryRecvError::Disconnected)
        ));
        assert!(matches!(
            second_events.recv().expect("second retained event"),
            meta::Stream::ConfigurationChanged(value) if value == configuration
        ));
    }

    #[test]
    fn both_sockets_accept_only_their_own_generated_frames() {
        let directory = tempfile::tempdir().expect("temporary root");
        let paths = paths(directory.path());
        write_manifest(&paths, directory.path());
        let actor = Actor::start(&paths).expect("actor");
        let ordinary_listener = bind(&paths.ordinary_socket).expect("ordinary listener");
        let meta_listener = bind(&paths.meta_socket).expect("meta listener");
        let ordinary_actor = actor.clone();
        let ordinary_thread = thread::spawn(move || {
            ordinary_socket(
                ordinary_listener.accept().expect("ordinary client").0,
                ordinary_actor,
            )
        });
        let meta_thread = thread::spawn(move || {
            meta_socket(meta_listener.accept().expect("meta client").0, actor)
        });
        let mut ordinary_client =
            UnixStream::connect(&paths.ordinary_socket).expect("ordinary connect");
        ordinary_client
            .write_all(
                &ordinary_frame(ordinary_signal::FrameBody::Request(
                    ordinary_signal::Request::Observe(
                        ordinary_signal::ObservationSelection::Assemblies,
                    ),
                ))
                .encode_length_prefixed()
                .expect("ordinary frame"),
            )
            .expect("ordinary write");
        let ordinary_reply = read_frame(&mut ordinary_client).expect("ordinary reply bytes");
        assert!(matches!(
            ordinary_signal::Frame::decode_length_prefixed(&ordinary_reply)
                .expect("ordinary reply frame")
                .body,
            ordinary_signal::FrameBody::Reply(ordinary_signal::Reply::Observed(_))
        ));
        let mut meta_client = UnixStream::connect(&paths.meta_socket).expect("meta connect");
        meta_client
            .write_all(
                &meta_frame(meta::FrameBody::Request(meta::Request::Observe(
                    meta::MetaObservationSelection::Configuration,
                )))
                .encode_length_prefixed()
                .expect("meta frame"),
            )
            .expect("meta write");
        let meta_reply = read_frame(&mut meta_client).expect("meta reply bytes");
        assert!(matches!(
            meta::Frame::decode_length_prefixed(&meta_reply)
                .expect("meta reply frame")
                .body,
            meta::FrameBody::Reply(meta::Reply::Observed(meta::MetaObservation::Configuration(
                _
            )))
        ));
        ordinary_thread.join().expect("ordinary server");
        meta_thread.join().expect("meta server");
    }

    #[test]
    fn malformed_frame_is_refused_by_the_codec_before_actor_dispatch() {
        assert!(ordinary_signal::Frame::decode_length_prefixed(&[4, 0, 0, 0]).is_err());
        assert!(meta::Frame::decode_length_prefixed(&[4, 0, 0, 0]).is_err());
    }

    #[test]
    fn containment_refusal_uses_the_outer_refusal_root_on_the_socket() {
        let directory = tempfile::tempdir().expect("temporary root");
        let paths = paths(directory.path());
        write_manifest(&paths, directory.path());
        let actor = Actor::start(&paths).expect("actor");
        let listener = bind(&paths.ordinary_socket).expect("ordinary listener");
        let server =
            thread::spawn(move || ordinary_socket(listener.accept().expect("client").0, actor));
        let mut client = UnixStream::connect(&paths.ordinary_socket).expect("connect");
        client
            .write_all(
                &ordinary_frame(ordinary_signal::FrameBody::Request(
                    ordinary_signal::Request::Generate(request("../escape.ethos")),
                ))
                .encode_length_prefixed()
                .expect("request frame"),
            )
            .expect("request write");
        let reply = read_frame(&mut client).expect("reply bytes");
        assert!(matches!(
            ordinary_signal::Frame::decode_length_prefixed(&reply)
                .expect("reply frame")
                .body,
            ordinary_signal::FrameBody::Refusal(ordinary_signal::Refusal::InvalidRelativePath(_))
        ));
        server.join().expect("server");
    }

    #[test]
    fn meta_observe_subscribe_unsubscribe_persists_config_and_orders_events() {
        let directory = tempfile::tempdir().expect("temporary root");
        let paths = paths(directory.path());
        write_manifest(&paths, directory.path());
        let actor = Actor::start(&paths).expect("actor");
        assert!(matches!(
            actor
                .meta(
                    meta::Request::Observe(meta::MetaObservationSelection::Configuration),
                    None
                )
                .expect("meta observe"),
            meta::Reply::Observed(meta::MetaObservation::Configuration(_))
        ));
        let subscription = meta::MetaSubscriptionRequest {
            selection: meta::MetaObservationSelection::Configuration,
        };
        let (events, received) = mpsc::channel();
        assert!(matches!(
            actor
                .meta(meta::Request::Subscribe(subscription.clone()), Some(events))
                .expect("meta subscribe"),
            meta::Reply::Observed(meta::MetaObservation::Configuration(_))
        ));
        let changed = meta::Configuration {
            ordinary_socket_path: paths
                .configuration()
                .expect("defaults")
                .ordinary_socket_path,
            meta_socket_path: paths.configuration().expect("defaults").meta_socket_path,
            source_manifest_path: paths
                .source_manifest
                .display()
                .to_string()
                .try_into()
                .expect("source manifest path"),
        };
        assert!(matches!(
            actor
                .meta(meta::Request::Configure(changed.clone()), None)
                .expect("meta configure"),
            meta::Reply::Configured(_)
        ));
        assert!(matches!(
            received.recv().expect("configuration event"),
            meta::Stream::ConfigurationChanged(value) if value == changed
        ));
        assert!(matches!(
            actor
                .meta(meta::Request::Unsubscribe(subscription), None)
                .expect("meta unsubscribe"),
            meta::Reply::Observed(_)
        ));
    }
}
