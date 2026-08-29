//! Datomic edge client for the ordinary Ethos-zero Nexus socket.

use std::{
    env,
    io::{Read, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    process::ExitCode,
};

use datomic::{Datomic, Text, TextEdge};
use signal::SignalFrameCodec as _;
use signal_ethos_zero as signal;

fn socket_path() -> Result<PathBuf, String> {
    env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .or_else(|| env::var_os("XDG_STATE_HOME").map(PathBuf::from))
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state")))
        .map(|root| root.join("ethos-zero-nexus/ethos-zero.sock"))
        .ok_or_else(|| "XDG_RUNTIME_DIR, XDG_STATE_HOME, or HOME is required".to_owned())
}
fn frame(body: signal::FrameBody) -> signal::Frame {
    signal::Frame {
        channel_contract_id: signal::CHANNEL_CONTRACT_ID,
        channel_wire_revision: signal::CHANNEL_WIRE_REVISION,
        protocol_version: signal::PROTOCOL_VERSION,
        body,
    }
}
fn parse_arguments(values: &[String]) -> Result<String, String> {
    match values {
        [value] if !value.starts_with('-') => Ok(value.clone()),
        _ => Err("accepts exactly one Datomic object and no flags".into()),
    }
}
fn single_argument() -> Result<String, String> {
    parse_arguments(&env::args().skip(1).collect::<Vec<_>>())
}
fn run() -> Result<(), String> {
    let request = Text::<signal::Request>::from(single_argument()?)
        .embody()
        .map_err(|error| format!("Datomic request: {error:?}"))?;
    let mut stream = UnixStream::connect(socket_path()?).map_err(|error| error.to_string())?;
    stream
        .write_all(
            &frame(signal::FrameBody::Request(request))
                .encode_length_prefixed()
                .map_err(|error| format!("{error:?}"))?,
        )
        .map_err(|error| error.to_string())?;
    let mut prefix = [0; 4];
    stream
        .read_exact(&mut prefix)
        .map_err(|error| error.to_string())?;
    let mut bytes = prefix.to_vec();
    bytes.resize(4 + u32::from_le_bytes(prefix) as usize, 0);
    stream
        .read_exact(&mut bytes[4..])
        .map_err(|error| error.to_string())?;
    match signal::Frame::decode_length_prefixed(&bytes)
        .map_err(|error| format!("{error:?}"))?
        .body
    {
        signal::FrameBody::Reply(reply) => println!("{}", reply.textualize().as_ref()),
        signal::FrameBody::Refusal(refusal) => println!("{}", refusal.textualize().as_ref()),
        _ => return Err("Nexus returned a non-reply frame".into()),
    }
    Ok(())
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ethos-zero: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_exactly_one_non_flag_datom() {
        assert!(parse_arguments(&["Observe.Assemblies".into()]).is_ok());
        assert!(parse_arguments(&[]).is_err());
        assert!(parse_arguments(&["--help".into()]).is_err());
        assert!(parse_arguments(&["Observe.Assemblies".into(), "extra".into()]).is_err());
    }
}
