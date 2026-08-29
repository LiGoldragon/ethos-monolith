use std::process::ExitCode;

fn main() -> ExitCode {
    match ethos_zero_nexus::run_default() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ethos-zero-nexus: {error}");
            ExitCode::FAILURE
        }
    }
}
