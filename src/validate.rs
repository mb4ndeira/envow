use std::path::Path;
use colored::Colorize;
use crate::schema::{self, VarSpec};
use crate::Format;

pub enum Error {
    Schema(anyhow::Error),
    Failures,
}

struct Failure {
    section: String,
    key: String,
    msg: String,
    spec: VarSpec,
}

pub fn run(schema_path: &Path, only: &[String], cli_env: Option<&str>, format: &Format) -> Result<(), Error> {
    let schema = schema::load(schema_path).map_err(Error::Schema)?;
    let mut failures: Vec<Failure> = Vec::new();

    let mut sections: Vec<(&String, &schema::Section)> = schema.iter().collect();
    sections.sort_by_key(|(k, _)| k.as_str());

    for (section, vars) in &sections {
        if !only.is_empty() && !only.contains(section) {
            continue;
        }
        let mut keys: Vec<&String> = vars.keys().collect();
        keys.sort();
        for key in keys {
            let spec = &vars[key];
            if !spec.applies_to_env(cli_env) {
                continue;
            }
            let value = std::env::var(key).ok();
            if let Err(msg) = check(spec, value.as_deref()) {
                failures.push(Failure { section: section.to_string(), key: key.clone(), msg, spec: spec.clone() });
            }
        }
    }

    if failures.is_empty() {
        match format {
            Format::Plain => println!("{}", "✓ all env vars valid".green()),
            Format::Json => println!("{}", serde_json::json!({ "ok": true, "failures": [] })),
        }
        return Ok(());
    }

    match format {
        Format::Json => {
            let payload = serde_json::json!({
                "ok": false,
                "failures": failures.iter().map(|f| serde_json::json!({
                    "section": f.section,
                    "key": f.key,
                    "message": f.msg,
                    "description": f.spec.description,
                    "example": f.spec.example,
                })).collect::<Vec<_>>()
            });
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
        }
        Format::Plain => {
            let n = failures.len();
            eprintln!(
                "\n{}\n",
                format!("✗ {n} env var{} missing or invalid:", if n == 1 { "" } else { "s" })
                    .red()
                    .bold()
            );
            for f in &failures {
                eprintln!("  {}  {}", f.key.yellow().bold(), f.msg.red());
                if let Some(desc) = &f.spec.description {
                    eprintln!("    → {}", desc.dimmed());
                }
                if let Some(ex) = &f.spec.example {
                    eprintln!("    example: {}", ex.dimmed());
                }
                eprintln!();
            }
        }
    }

    Err(Error::Failures)
}

fn check(spec: &VarSpec, value: Option<&str>) -> Result<(), String> {
    let v = match value {
        None | Some("") => {
            return if spec.is_required() {
                Err("[required] not set".into())
            } else {
                Ok(())
            };
        }
        Some(v) => v,
    };

    match spec.var_type.as_deref().unwrap_or("string") {
        "string" => {
            if let Some(min) = spec.min_length {
                if v.len() < min {
                    return Err(format!("[string] too short — min {min} chars, got {}", v.len()));
                }
            }
            if let Some(max) = spec.max_length {
                if v.len() > max {
                    return Err(format!("[string] too long — max {max} chars, got {}", v.len()));
                }
            }
            if let Some(choices) = &spec.choices {
                if !choices.iter().any(|c| c == v) {
                    return Err(format!("[string] must be one of: {}", choices.join(", ")));
                }
            }
        }
        "url" => {
            if !v.starts_with("http://") && !v.starts_with("https://") {
                return Err("[url] must start with http:// or https://".into());
            }
        }
        "port" => {
            if v.parse::<u16>().is_err() {
                return Err(format!("[port] {v:?} is not a valid port (1–65535)"));
            }
        }
        "int" => {
            if v.parse::<i64>().is_err() {
                return Err(format!("[int] {v:?} is not a valid integer"));
            }
        }
        "float" => {
            if v.parse::<f64>().is_err() {
                return Err(format!("[float] {v:?} is not a valid float"));
            }
        }
        "bool" => {
            match v.to_lowercase().as_str() {
                "true" | "false" | "1" | "0" | "yes" | "no" => {}
                _ => return Err(format!("[bool] {v:?} — valid values: true/false/1/0/yes/no")),
            }
        }
        "email" => {
            if !v.contains('@') {
                return Err(format!("[email] {v:?} does not look like an email address"));
            }
        }
        other => {
            return Err(format!(
                "unknown type {other:?} — valid: string, url, port, int, float, bool, email"
            ));
        }
    }

    Ok(())
}
