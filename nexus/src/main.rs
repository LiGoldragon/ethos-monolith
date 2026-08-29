//! The Ethos-zero Nexus.  Its two Unix-socket edges carry only signal frames.

use std::{
    fs,
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};

use ethos_zero::{FileReader, Manifest, RustEmitter};
use meta::SignalFrameCodec as _;
use meta_signal_ethos_zero as meta;
use ordinary::SignalFrameCodec as _;
use signal_ethos_zero as ordinary;

#[derive(Clone, Debug)]
struct Configuration {
    ordinary_socket: PathBuf,
    meta_socket: PathBuf,
    source_manifest: PathBuf,
}

impl Default for Configuration {
    fn default() -> Self {
        let directory = std::env::temp_dir();
        Self {
            ordinary_socket: directory.join("ethos-zero.sock"),
            meta_socket: directory.join("ethos-zero-meta.sock"),
            source_manifest: PathBuf::from("sources.datom"),
        }
    }
}

struct EmptyManifest;

impl Manifest for EmptyManifest {
    fn resolve(&self, _: &str) -> Option<ethos_zero::FileLocation> {
        None
    }
}

struct NexusCore {
    configuration: Mutex<Configuration>,
    assemblies: Mutex<Vec<ordinary::AssemblySummary>>,
}

impl NexusCore {
    fn new(configuration: Configuration) -> Self {
        Self {
            configuration: Mutex::new(configuration),
            assemblies: Mutex::new(Vec::new()),
        }
    }

    fn ordinary(&self, request: ordinary::Request) -> ordinary::Reply {
        match request {
            ordinary::Request::Observe(ordinary::ObservationSelection::Assemblies) => {
                ordinary::Reply::Observed(ordinary::Observation::Assemblies(
                    ordinary::AssemblySnapshot {
                        assemblies: ordinary::Assemblies(
                            self.assemblies.lock().expect("assemblies lock").clone(),
                        ),
                    },
                ))
            }
            ordinary::Request::Generate(request) => self.generate(request),
        }
    }

    fn generate(&self, request: ordinary::GenerationRequest) -> ordinary::Reply {
        let path = Path::new(&request.file.relative_path.0);
        let Ok(source) = fs::read_to_string(path) else {
            return ordinary::Reply::GenerationRejected(ordinary::GenerationRefusal::FileAbsent(
                request.file,
            ));
        };
        let reader = FileReader::new(&EmptyManifest);
        let Ok(file) = reader.read(&source) else {
            return ordinary::Reply::GenerationRejected(ordinary::GenerationRefusal::InvalidEthos(
                syntax_fault("Ethos source is not embodied"),
            ));
        };
        let Ok(_) = RustEmitter::new().emit(&file) else {
            return ordinary::Reply::GenerationRejected(
                ordinary::GenerationRefusal::RustProjectionRejected(ordinary::ProjectionFault {
                    reason: ordinary::ProjectionFaultReason("Rust projection rejected".into()),
                }),
            );
        };
        let generation = ordinary::Generation {
            file: request.file,
            artifact: ordinary::ArtifactPath("src/generated/signal.rs".into()),
        };
        self.assemblies
            .lock()
            .expect("assemblies lock")
            .push(ordinary::AssemblySummary {
                file: generation.file.clone(),
                artifact: generation.artifact.clone(),
            });
        ordinary::Reply::Generated(generation)
    }

    fn meta(&self, request: meta::Request) -> meta::Reply {
        match request {
            meta::Request::Configure(next) => {
                if next.ordinary_socket_path.0.is_empty() {
                    return meta::Reply::ConfigurationRejected(
                        meta::ConfigurationRefusal::InvalidOrdinarySocketPath,
                    );
                }
                if next.meta_socket_path.0.is_empty() {
                    return meta::Reply::ConfigurationRejected(
                        meta::ConfigurationRefusal::InvalidMetaSocketPath,
                    );
                }
                if next.source_manifest_path.0.is_empty() {
                    return meta::Reply::ConfigurationRejected(
                        meta::ConfigurationRefusal::InvalidSourceManifestPath,
                    );
                }
                *self.configuration.lock().expect("configuration lock") = Configuration {
                    ordinary_socket: next.ordinary_socket_path.0.clone().into(),
                    meta_socket: next.meta_socket_path.0.clone().into(),
                    source_manifest: next.source_manifest_path.0.clone().into(),
                };
                meta::Reply::Configured(next)
            }
            meta::Request::Observe(meta::MetaObservationSelection::Configuration) => {
                let configuration = self.configuration.lock().expect("configuration lock");
                meta::Reply::Observed(meta::MetaObservation::Configuration(meta::Configuration {
                    ordinary_socket_path: meta::OrdinarySocketPath(
                        configuration.ordinary_socket.display().to_string(),
                    ),
                    meta_socket_path: meta::MetaSocketPath(
                        configuration.meta_socket.display().to_string(),
                    ),
                    source_manifest_path: meta::SourceManifestPath(
                        configuration.source_manifest.display().to_string(),
                    ),
                }))
            }
            meta::Request::Observe(meta::MetaObservationSelection::Sources) => {
                meta::Reply::Observed(meta::MetaObservation::Sources(meta::SourceIndex {
                    sources: meta::Sources(Vec::new()),
                }))
            }
        }
    }
}

fn syntax_fault(reason: &str) -> ordinary::SyntaxFault {
    ordinary::SyntaxFault {
        extent: ordinary::SourceExtent {
            extent_start: ordinary::ExtentStart(0),
            extent_end: ordinary::ExtentEnd(0),
        },
        reason: ordinary::SyntaxFaultReason(reason.into()),
    }
}

fn ordinary_frame(body: ordinary::FrameBody) -> ordinary::Frame {
    ordinary::Frame {
        channel_contract_id: ordinary::CHANNEL_CONTRACT_ID,
        channel_wire_revision: ordinary::CHANNEL_WIRE_REVISION,
        protocol_version: ordinary::PROTOCOL_VERSION,
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
    let length = u32::from_le_bytes(prefix) as usize;
    let mut frame = vec![0; 4 + length];
    frame[..4].copy_from_slice(&prefix);
    stream.read_exact(&mut frame[4..])?;
    Ok(frame)
}

fn serve_ordinary(listener: UnixListener, core: Arc<NexusCore>) {
    for stream in listener.incoming().flatten() {
        let core = Arc::clone(&core);
        thread::spawn(move || {
            let mut stream = stream;
            let reply = match read_frame(&mut stream)
                .ok()
                .and_then(|bytes| ordinary::Frame::decode_length_prefixed(&bytes).ok())
            {
                Some(ordinary::Frame {
                    body: ordinary::FrameBody::Request(request),
                    ..
                }) => core.ordinary(request),
                _ => {
                    ordinary::Reply::GenerationRejected(ordinary::GenerationRefusal::InvalidEthos(
                        syntax_fault("invalid ordinary signal frame"),
                    ))
                }
            };
            let _ = stream.write_all(
                &ordinary_frame(ordinary::FrameBody::Reply(reply))
                    .encode_length_prefixed()
                    .expect("constant valid ordinary response"),
            );
        });
    }
}

fn serve_meta(listener: UnixListener, core: Arc<NexusCore>) {
    for stream in listener.incoming().flatten() {
        let core = Arc::clone(&core);
        thread::spawn(move || {
            let mut stream = stream;
            let reply = match read_frame(&mut stream)
                .ok()
                .and_then(|bytes| meta::Frame::decode_length_prefixed(&bytes).ok())
            {
                Some(meta::Frame {
                    body: meta::FrameBody::Request(request),
                    ..
                }) => core.meta(request),
                _ => meta::Reply::ConfigurationRejected(
                    meta::ConfigurationRefusal::InvalidSourceManifestPath,
                ),
            };
            let _ = stream.write_all(
                &meta_frame(meta::FrameBody::Reply(reply))
                    .encode_length_prefixed()
                    .expect("constant valid meta response"),
            );
        });
    }
}

fn bind(path: &Path) -> std::io::Result<UnixListener> {
    if path.exists() {
        fs::remove_file(path)?;
    }
    UnixListener::bind(path)
}

fn main() -> std::io::Result<()> {
    let configuration = Configuration::default();
    let ordinary = bind(&configuration.ordinary_socket)?;
    let meta = bind(&configuration.meta_socket)?;
    let core = Arc::new(NexusCore::new(configuration));
    let meta_core = Arc::clone(&core);
    thread::spawn(move || serve_meta(meta, meta_core));
    serve_ordinary(ordinary, core);
    Ok(())
}
