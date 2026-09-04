use std::{env, fs, path::Path, process::ExitCode};

use protos::{Head, Protoform, Separator, Structural};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [] => {
            print!("{}", include_str!("../ethos-zero.ethos"));
            ExitCode::SUCCESS
        }
        [arg] if !arg.starts_with('-') => match run(arg) {
            Ok(output) => {
                println!("{output}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        },
        _ => {
            eprintln!(
                "GenerationFailure.{{ \u{201C}ethos-zero accepts one datom value and no flags\u{201D} }}"
            );
            ExitCode::FAILURE
        }
    }
}

fn run(arg: &str) -> Result<String, String> {
    let delineation = arg.to_owned().delineate().map_err(|f| {
        format!(
            "GenerationFailure.{{ \u{201C}delineation fault at {}..{}\u{201D} }}",
            f.extent.0, f.extent.1
        )
    })?;

    let [pf] = delineation.protoforms.as_slice() else {
        return Err("GenerationFailure.{ \u{201C}expected one datom value\u{201D} }".into());
    };

    let Protoform::Headed(Head::Bare(head), sep, body) = pf else {
        return Err(
            "GenerationFailure.{ \u{201C}expected Generate.{ path out-dir }\u{201D} }".into(),
        );
    };

    if head != "Generate" || *sep != Separator::Period {
        return Err(format!(
            "GenerationFailure.{{ \u{201C}unknown command: {head}\u{201D} }}"
        ));
    }

    let Protoform::Enclosed(protos::Enclosure::Braced, children) = body.as_ref() else {
        return Err("GenerationFailure.{ \u{201C}expected braced body\u{201D} }".into());
    };
    if children.len() < 2 {
        return Err(
            "GenerationFailure.{ \u{201C}expected Generate.{ file-path out-dir }\u{201D} }".into(),
        );
    }

    let file_path = rejoin_text(&children[0]);
    let out_dir = rejoin_text(&children[1]);

    let source = fs::read_to_string(&file_path).map_err(|e| {
        format!("GenerationFailure.{{ \u{201C}{file_path}\u{201D} \u{201C}{e}\u{201D} }}")
    })?;

    use ethos_zero::{Actualizing, Emitting};

    let concept = ethos_zero::Potential::from(source.as_str())
        .actualize()
        .map_err(|f| {
            format!("GenerationFailure.{{ \u{201C}{file_path}\u{201D} \u{201C}{f}\u{201D} }}")
        })?;

    let rust = concept.emit().map_err(|f| {
        format!("GenerationFailure.{{ \u{201C}{file_path}\u{201D} \u{201C}{f}\u{201D} }}")
    })?;

    let formatted = format_rust(&rust).unwrap_or(rust);

    let out_name = match &concept {
        ethos_zero::Concept::Library(_) => {
            Path::new(&file_path)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
                + ".rs"
        }
        ethos_zero::Concept::Signal(_) => "signal.rs".to_owned(),
    };

    let out_path = Path::new(&out_dir).join(&out_name);
    fs::create_dir_all(&out_dir).map_err(|e| {
        format!("GenerationFailure.{{ \u{201C}{out_dir}\u{201D} \u{201C}{e}\u{201D} }}")
    })?;
    fs::write(&out_path, &formatted).map_err(|e| {
        format!(
            "GenerationFailure.{{ \u{201C}{}\u{201D} \u{201C}{e}\u{201D} }}",
            out_path.display()
        )
    })?;

    Ok(format!(
        "Generated.[ \u{201C}{}\u{201D} ]",
        out_path.display()
    ))
}

fn rejoin_text(pf: &Protoform) -> String {
    match pf {
        Protoform::Bare(s) => s.clone(),
        Protoform::Headed(Head::Bare(head), sep, body) => {
            format!("{}{}{}", head, sep.glyph(), rejoin_text(body))
        }
        Protoform::Headed(Head::Qualified(head, _), sep, body) => {
            format!("{}{}{}", head, sep.glyph(), rejoin_text(body))
        }
        _ => String::new(),
    }
}

fn format_rust(source: &str) -> Option<String> {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut child = Command::new("rustfmt")
        .arg("--edition=2024")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    child.stdin.take()?.write_all(source.as_bytes()).ok()?;
    let output = child.wait_with_output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8(output.stdout).ok())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_flags() {
        assert!(run("--help").is_err());
    }

    #[test]
    fn rejects_unknown_command() {
        let result = run("Unknown.{ /tmp/x /tmp/y }");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown command"));
    }

    #[test]
    fn rejoin_recovers_dotted_path() {
        let delineation = "/abs/file.ethos".to_owned().delineate().unwrap();
        let rejoined = rejoin_text(&delineation.protoforms[0]);
        assert_eq!(rejoined, "/abs/file.ethos");
    }

    #[test]
    fn rejoin_recovers_plain_path() {
        let delineation = "/abs/out-dir".to_owned().delineate().unwrap();
        let rejoined = rejoin_text(&delineation.protoforms[0]);
        assert_eq!(rejoined, "/abs/out-dir");
    }
}
