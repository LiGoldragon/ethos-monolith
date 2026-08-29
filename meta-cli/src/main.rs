use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    process::ExitCode,
};

use meta_signal_ethos_zero as signal;
use signal::SignalFrameCodec as _;

fn frame(body: signal::FrameBody) -> signal::Frame {
    signal::Frame {
        channel_contract_id: signal::CHANNEL_CONTRACT_ID,
        channel_wire_revision: signal::CHANNEL_WIRE_REVISION,
        protocol_version: signal::PROTOCOL_VERSION,
        body,
    }
}
fn request(input: &str) -> Result<signal::Request, &'static str> {
    match input {
        "Observe.Configuration" => Ok(signal::Request::Observe(
            signal::MetaObservationSelection::Configuration,
        )),
        "Observe.Sources" => Ok(signal::Request::Observe(
            signal::MetaObservationSelection::Sources,
        )),
        _ if input.starts_with("Configure.{") && input.ends_with('}') => {
            let values = input[11..input.len() - 1].split(' ').collect::<Vec<_>>();
            let [ordinary, meta, manifest] = values.as_slice() else {
                return Err("Configure needs three paths");
            };
            Ok(signal::Request::Configure(signal::Configuration {
                ordinary_socket_path: signal::OrdinarySocketPath((*ordinary).into()),
                meta_socket_path: signal::MetaSocketPath((*meta).into()),
                source_manifest_path: signal::SourceManifestPath((*manifest).into()),
            }))
        }
        _ => Err("expected one Datomic meta request"),
    }
}
fn render(reply: signal::Reply) -> String {
    match reply {
        signal::Reply::Configured(_) => "Configured".into(),
        signal::Reply::Observed(signal::MetaObservation::Configuration(_)) => {
            "Observed.Configuration".into()
        }
        signal::Reply::Observed(signal::MetaObservation::Sources(_)) => "Observed.Sources".into(),
        signal::Reply::ConfigurationRejected(_) => "ConfigurationRejected".into(),
    }
}
fn main() -> ExitCode {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments.len() != 1 || arguments[0].starts_with('-') {
        eprintln!("ethos-zero-meta takes exactly one inline Datomic object");
        return ExitCode::from(2);
    }
    let Ok(request) = request(&arguments[0]) else {
        eprintln!("invalid Datomic request");
        return ExitCode::from(2);
    };
    let Ok(mut stream) = UnixStream::connect(std::env::temp_dir().join("ethos-zero-meta.sock"))
    else {
        eprintln!("Ethos-zero Nexus meta socket unavailable");
        return ExitCode::FAILURE;
    };
    let encoded = frame(signal::FrameBody::Request(request))
        .encode_length_prefixed()
        .expect("valid request");
    if stream.write_all(&encoded).is_err() {
        return ExitCode::FAILURE;
    }
    let mut prefix = [0; 4];
    if stream.read_exact(&mut prefix).is_err() {
        return ExitCode::FAILURE;
    }
    let mut bytes = prefix.to_vec();
    bytes.resize(4 + u32::from_le_bytes(prefix) as usize, 0);
    if stream.read_exact(&mut bytes[4..]).is_err() {
        return ExitCode::FAILURE;
    }
    match signal::Frame::decode_length_prefixed(&bytes) {
        Ok(signal::Frame {
            body: signal::FrameBody::Reply(reply),
            ..
        }) => {
            println!("{}", render(reply));
            ExitCode::SUCCESS
        }
        _ => ExitCode::FAILURE,
    }
}
