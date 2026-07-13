use std::{fmt::Write as FmtWrite, path::Path};
use crate::schema;

pub fn run(schema_path: &Path, output: &Path) -> anyhow::Result<()> {
    let schema = schema::load(schema_path)?;
    let mut out = String::new();

    let mut sections: Vec<(&String, &schema::Section)> = schema.iter().collect();
    sections.sort_by_key(|(k, _)| k.as_str());

    for (section_name, vars) in &sections {
        let bar = "─".repeat(78_usize.saturating_sub(section_name.len() + 6));
        writeln!(out, "# ── {section_name} {bar}")?;
        writeln!(out)?;

        let mut keys: Vec<&String> = vars.keys().collect();
        keys.sort();

        for key in keys {
            let spec = &vars[key];

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

            let mut comment = spec.description.clone().unwrap_or_default();
            if !meta.is_empty() {
                let tag = format!("[{}]", meta.join(", "));
                comment = if comment.is_empty() { tag } else { format!("{comment} {tag}") };
            }
            if !comment.is_empty() {
                writeln!(out, "# {comment}")?;
            }

            let example = spec.example.as_deref().unwrap_or("");
            if spec.is_required() {
                writeln!(out, "{key}={example}")?;
            } else {
                let val = spec.default.as_deref().unwrap_or(example);
                writeln!(out, "# {key}={val}")?;
            }
            writeln!(out)?;
        }
    }

    std::fs::write(output, &out)?;
    println!("generated {}", output.display());
    Ok(())
}
