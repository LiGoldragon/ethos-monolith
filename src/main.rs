//! The ethos-zero CLI: it speaks datom, through its own contract.
//!
//! One inline datom value, no flags. `Generate.{ /abs/file.ethos
//! /abs/out-dir }` reads the file, generates its Rust module and
//! writes `/abs/out-dir/<stem>.rs`. Every reply is a value of the
//! contract's `Response`, textualized. With no argument the CLI prints
//! its own ethos.

use std::path::Path;
use std::process::ExitCode;

use datomic::{Datom, Datomic};
use ethos_zero::{File, Generating};
use protos::{Actualizable, Potential, Text};

/// The crate's contract, declared in ethos-zero.ethos and generated into contract.rs.
#[rustfmt::skip]
mod contract;

use contract::{Generation, Request, Response};

/// The crate's own ethos, printed when nothing is asked.
const ETHOS: &str = include_str!("../ethos-zero.ethos");

// ---------------------------------------------------------------------------
// Kinds
// ---------------------------------------------------------------------------

/// The kind whose capability serves a request, yielding the response.
trait Serving {
    fn serve(&self) -> Response;
}

/// The kind whose capability yields the process exit code a response ends with.
trait Exiting {
    fn exit(&self) -> ExitCode;
}

/// The kind whose capability answers the command line as a whole.
trait Invoking {
    fn invoke(&self) -> ExitCode;
}

// ---------------------------------------------------------------------------
// Interactions
// ---------------------------------------------------------------------------

impl Serving for Generation {
    fn serve(&self) -> Response {
        let Generation(source, directory) = self;
        let text = match std::fs::read_to_string(source) {
            Ok(text) => text,
            Err(error) => return Response::Unreadable(source.clone(), error.to_string()),
        };
        let file = match Potential::<File>::from(text).actualize() {
            Ok(file) => file,
            Err(fault) => return Response::Faulty(source.clone(), fault),
        };
        let stem = Path::new(source)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let target = Path::new(directory).join(format!("{stem}.rs"));
        let written = target.to_string_lossy().into_owned();
        if let Err(error) = std::fs::create_dir_all(directory) {
            return Response::Unwritable(directory.clone(), error.to_string());
        }
        match std::fs::write(&target, file.generate()) {
            Ok(()) => Response::Generated(vec![written]),
            Err(error) => Response::Unwritable(written, error.to_string()),
        }
    }
}

impl Serving for Request {
    fn serve(&self) -> Response {
        match self {
            Request::Generate(generation) => generation.serve(),
        }
    }
}

impl Serving for Potential<Request, Datom> {
    fn serve(&self) -> Response {
        match self.actualize() {
            Ok(request) => request.serve(),
            Err(fault) => Response::Malformed(fault),
        }
    }
}

impl Exiting for Response {
    fn exit(&self) -> ExitCode {
        match self {
            Response::Generated(_) => ExitCode::SUCCESS,
            Response::Arguments(_)
            | Response::Malformed(_)
            | Response::Unreadable(_, _)
            | Response::Faulty(_, _)
            | Response::Unwritable(_, _) => ExitCode::FAILURE,
        }
    }
}

impl Invoking for [Text] {
    fn invoke(&self) -> ExitCode {
        let response = match self {
            [] => {
                print!("{ETHOS}");
                if !ETHOS.ends_with('\n') {
                    println!();
                }
                return ExitCode::SUCCESS;
            }
            [argument] => Potential::<Request, Datom>::from(argument.as_str()).serve(),
            many => Response::Arguments(many.len() as protos::Integer),
        };
        println!("{}", response.textualize());
        response.exit()
    }
}

fn main() -> ExitCode {
    let arguments: Vec<Text> = std::env::args().skip(1).collect();
    arguments.invoke()
}
