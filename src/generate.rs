use std::{fmt::Write as FmtWrite, path::Path};
use crate::schema;

pub fn run(schema_path: &Path, output: &Path, cli_env: Option<&str>) -> anyhow::Result<()> {
    let schema = schema::load(schema_path)?;
    let mut out = String::new();

    let mut sections: Vec<(&String, &schema::Section)> = schema.iter().collect();
    sections.sort_by_key(|(k, _)| k.as_str());

    for (section_name, vars) in &sections {
        let mut keys: Vec<&String> = vars.keys().collect();
        keys.sort();

        let visible: Vec<&&String> = keys.iter()
            .filter(|k| vars[**k].applies_to_env(cli_env))
            .collect();
        if visible.is_empty() {
            continue;
        }

        let bar = "─".repeat(78_usize.saturating_sub(section_name.len() + 6));
        writeln!(out, "# ── {section_name} {bar}")?;
        writeln!(out)?;

        for key in visible {
            let spec = &vars[*key];

            let mut meta: Vec<String> = Vec::new();
            if let Some(t) = &spec.var_type {
                if t != "string" {
                    meta.push(format!("type:{t}"));
                }
            }
            if spec.is_required() {
                meta.push("required".into());
            } else if let Some(d) = &spec.default {
                meta.push(format!("default:{d}"));
            }
            if let Some(min) = spec.min_length {
                meta.push(format!("min_length:{min}"));
            }
            if let Some(choices) = &spec.choices {
                meta.push(format!("choices:{}", choices.join("|")));
            }
            if let Some(g) = &spec.generate {
                meta.push(format!("generated:{g}"));
            }

            let mut comment = spec.description.clone().unwrap_or_default();
            if !meta.is_empty() {
                let tag = format!("[{}]", meta.join(", "));
                comment = if comment.is_empty() { tag } else { format!("{comment} {tag}") };
            }
            if !comment.is_empty() {
                writeln!(out, "# {comment}")?;
            }

            let placeholder = resolve_placeholder(spec, cli_env)?;
            if spec.is_required() || spec.generate.is_some() {
                writeln!(out, "{key}={placeholder}")?;
            } else {
                let val = spec.default.as_deref().unwrap_or(&placeholder);
                writeln!(out, "# {key}={val}")?;
            }
            writeln!(out)?;
        }
    }

    std::fs::write(output, &out)?;
    println!("generated {}", output.display());
    Ok(())
}

fn resolve_placeholder(spec: &schema::VarSpec, cli_env: Option<&str>) -> anyhow::Result<String> {
    if let Some(gen) = &spec.generate {
        return generate_secret(gen);
    }
    if let Some(env) = cli_env {
        if let Some(values) = &spec.values {
            if let Some(v) = values.get(env) {
                return Ok(v.clone());
            }
        }
    }
    Ok(spec.example.clone().unwrap_or_default())
}

fn generate_secret(kind: &str) -> anyhow::Result<String> {
    use rand::RngCore;
    let mut rng = rand::rng();

    match kind {
        "hex32" => {
            let mut b = [0u8; 32];
            rng.fill_bytes(&mut b);
            Ok(b.iter().map(|x| format!("{x:02x}")).collect())
        }
        "hex64" => {
            let mut b = [0u8; 64];
            rng.fill_bytes(&mut b);
            Ok(b.iter().map(|x| format!("{x:02x}")).collect())
        }
        "base64_32" => {
            let mut b = [0u8; 32];
            rng.fill_bytes(&mut b);
            use base64::{Engine, engine::general_purpose::STANDARD};
            Ok(STANDARD.encode(b))
        }
        "uuid" => {
            let mut b = [0u8; 16];
            rng.fill_bytes(&mut b);
            b[6] = (b[6] & 0x0f) | 0x40;
            b[8] = (b[8] & 0x3f) | 0x80;
            let h: String = b.iter().map(|x| format!("{x:02x}")).collect();
            Ok(format!("{}-{}-{}-{}-{}", &h[0..8], &h[8..12], &h[12..16], &h[16..20], &h[20..32]))
        }
        other => Err(anyhow::anyhow!(
            "unknown generate type {other:?} — valid: hex32, hex64, base64_32, uuid"
        )),
    }
}
