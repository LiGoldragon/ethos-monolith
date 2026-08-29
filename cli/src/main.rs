use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    process::ExitCode,
};

use signal::SignalFrameCodec as _;
use signal_ethos_zero as signal;

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
        "Observe.Assemblies" => Ok(signal::Request::Observe(
            signal::ObservationSelection::Assemblies,
        )),
        _ if input.starts_with("Generate.{") && input.ends_with('}') => {
            let fields = input[10..input.len() - 1].split(' ').collect::<Vec<_>>();
            let [source, relative_path] = fields.as_slice() else {
                return Err("Generate needs source and relative path");
            };
            Ok(signal::Request::Generate(signal::GenerationRequest {
                file: signal::FileLocation {
                    source_name: signal::SourceName((*source).into()),
                    relative_path: signal::RelativePath((*relative_path).into()),
                },
            }))
        }
        _ => Err("expected one Datomic ordinary request"),
    }
}

fn render(reply: signal::Reply) -> String {
    match reply {
        signal::Reply::Generated(value) => format!(
            "Generated.{{{} {}}}",
            value.file.source_name.0, value.artifact.0
        ),
        signal::Reply::Observed(signal::Observation::Assemblies(value)) => {
            format!("Observed.Assemblies.[{}]", value.assemblies.0.len())
        }
        signal::Reply::GenerationRejected(_) => "GenerationRejected".into(),
    }
}

fn main() -> ExitCode {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments.len() != 1 || arguments[0].starts_with('-') {
        eprintln!("ethos-zero takes exactly one inline Datomic object");
        return ExitCode::from(2);
    }
    let Ok(request) = request(&arguments[0]) else {
        eprintln!("invalid Datomic request");
        return ExitCode::from(2);
    };
    let Ok(mut stream) = UnixStream::connect(std::env::temp_dir().join("ethos-zero.sock")) else {
        eprintln!("Ethos-zero Nexus ordinary socket unavailable");
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
