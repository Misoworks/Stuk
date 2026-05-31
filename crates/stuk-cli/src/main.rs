mod build;
mod doctor;
mod project;
mod runtime;
mod templates;

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use build::{BuildOptions, run_build};
use clap::{Parser, Subcommand};
use doctor::run_doctor;
use project::{CreateProjectOptions, DevOptions, create_project, run_cargo_command, run_dev};
use runtime::RuntimeCommand;
use stuk_devtools::{BundlePlan, BundleTarget, ManifestInspection, PreviewDescriptor};
use stuk_manifest::{Diagnostic, DiagnosticLevel, parse_file, validate_with_base_dir};

#[derive(Debug, Parser)]
#[command(name = "stuk", version, about = "Native UI tooling for Stuk apps")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    New {
        name: String,
        #[arg(long, default_value = "basic")]
        template: String,
    },
    Dev {
        #[arg(long)]
        once: bool,
        #[arg(long, default_value_t = 750)]
        poll_ms: u64,
    },
    Run,
    Build {
        #[arg(long)]
        release: bool,
        #[arg(long)]
        target: Option<String>,
    },
    Validate {
        #[arg(long)]
        json: bool,
        #[arg(default_value = "Stuk.toml")]
        manifest: PathBuf,
    },
    Doctor {
        #[arg(long)]
        json: bool,
    },
    Inspect {
        #[arg(long)]
        json: bool,
        #[arg(default_value = "Stuk.toml")]
        manifest: PathBuf,
    },
    Preview {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        theme: Option<String>,
        #[arg(long)]
        density: Option<String>,
        #[arg(default_value = "Stuk.toml")]
        manifest: PathBuf,
    },
    Fmt,
    Check,
    Bundle {
        #[arg(long, default_value = "staccato")]
        target: String,
        #[arg(long)]
        json: bool,
        #[arg(default_value = "Stuk.toml")]
        manifest: PathBuf,
    },
    Runtime {
        #[command(subcommand)]
        command: RuntimeSubcommand,
    },
}

#[derive(Debug, Subcommand)]
enum RuntimeSubcommand {
    List {
        #[arg(long)]
        json: bool,
    },
    Install {
        engine: String,
    },
    Remove {
        engine: String,
        version: Option<String>,
    },
    Doctor {
        #[arg(long)]
        json: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli.command.unwrap_or(Command::Doctor { json: false })) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn run(command: Command) -> Result<ExitCode, String> {
    match command {
        Command::New { name, template } => {
            create_project(CreateProjectOptions { name, template })?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Dev { once, poll_ms } => run_dev(DevOptions { once, poll_ms }),
        Command::Run => run_cargo_command("run", &["run"]),
        Command::Build { release, target } => run_build(BuildOptions { release, target }),
        Command::Validate { json, manifest } => validate_manifest(json, manifest),
        Command::Doctor { json } => Ok(run_doctor(json)),
        Command::Inspect { json, manifest } => inspect_manifest(json, manifest),
        Command::Preview {
            json,
            theme,
            density,
            manifest,
        } => preview_manifest(json, theme, density, manifest),
        Command::Fmt => run_cargo_command("fmt", &["fmt"]),
        Command::Check => run_cargo_command("check", &["check"]),
        Command::Bundle {
            target,
            json,
            manifest,
        } => bundle_manifest(target, json, manifest),
        Command::Runtime { command } => Ok(runtime::run_runtime(match command {
            RuntimeSubcommand::List { json } => RuntimeCommand::List { json },
            RuntimeSubcommand::Install { engine } => RuntimeCommand::Install { engine },
            RuntimeSubcommand::Remove { engine, version } => {
                RuntimeCommand::Remove { engine, version }
            }
            RuntimeSubcommand::Doctor { json } => RuntimeCommand::Doctor { json },
        })),
    }
}

fn validate_manifest(json: bool, manifest: PathBuf) -> Result<ExitCode, String> {
    let manifest_path = manifest;
    let manifest = match parse_file(&manifest_path) {
        Ok(manifest) => manifest,
        Err(error) => {
            if json {
                println!(
                    "{{\"ok\":false,\"diagnostics\":[{{\"level\":\"error\",\"path\":\"{}\",\"message\":\"{}\"}}]}}",
                    escape_json(&manifest_path.display().to_string()),
                    escape_json(&error.to_string())
                );
            } else {
                eprintln!("validation failed: {error}");
            }
            return Ok(ExitCode::from(1));
        }
    };

    let base_dir = manifest_base_dir(&manifest_path);
    let diagnostics = validate_with_base_dir(&manifest, base_dir);
    let has_errors = diagnostics
        .iter()
        .any(|diagnostic| diagnostic.level == DiagnosticLevel::Error);

    if json {
        println!(
            "{{\"ok\":{},\"diagnostics\":{}}}",
            !has_errors,
            diagnostics_json(&diagnostics)
        );
    } else if diagnostics.is_empty() {
        println!("{} is valid", manifest_path.display());
    } else {
        for diagnostic in &diagnostics {
            let level = match diagnostic.level {
                DiagnosticLevel::Error => "error",
                DiagnosticLevel::Warning => "warning",
            };
            println!("{level}: {}: {}", diagnostic.path, diagnostic.message);
        }
    }

    if has_errors {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn inspect_manifest(json: bool, manifest: PathBuf) -> Result<ExitCode, String> {
    let manifest_data = match parse_file(&manifest) {
        Ok(manifest_data) => manifest_data,
        Err(error) => {
            if json {
                println!(
                    "{{\"ok\":false,\"diagnostics\":[{{\"level\":\"error\",\"path\":\"{}\",\"message\":\"{}\"}}]}}",
                    escape_json(&manifest.display().to_string()),
                    escape_json(&error.to_string())
                );
            } else {
                eprintln!("inspect failed: {error}");
            }
            return Ok(ExitCode::from(1));
        }
    };
    let base_dir = manifest_base_dir(&manifest);
    let diagnostics = validate_with_base_dir(&manifest_data, base_dir);
    let inspection = ManifestInspection::from_manifest(&manifest_data, &diagnostics);

    if json {
        println!("{}", inspection.to_json());
    } else {
        print!("{}", inspection.to_text());
    }

    if inspection.ok {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}

fn preview_manifest(
    json: bool,
    theme: Option<String>,
    density: Option<String>,
    manifest: PathBuf,
) -> Result<ExitCode, String> {
    let manifest_data = match parse_file(&manifest) {
        Ok(manifest_data) => manifest_data,
        Err(error) => {
            if json {
                println!(
                    "{{\"ok\":false,\"previews\":[],\"diagnostics\":[{{\"level\":\"error\",\"path\":\"{}\",\"message\":\"{}\"}}]}}",
                    escape_json(&manifest.display().to_string()),
                    escape_json(&error.to_string())
                );
            } else {
                eprintln!("preview failed: {error}");
            }
            return Ok(ExitCode::from(1));
        }
    };
    let base_dir = manifest_base_dir(&manifest);
    let diagnostics = validate_with_base_dir(&manifest_data, base_dir);
    let inspection = ManifestInspection::from_manifest(&manifest_data, &diagnostics);
    let previews = inspection.preview_descriptors(theme.as_deref(), density.as_deref());

    if json {
        println!("{}", preview_json(&inspection, &previews));
    } else {
        print!("{}", preview_text(&inspection, &previews));
    }

    if inspection.ok {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}

fn preview_text(inspection: &ManifestInspection, previews: &[PreviewDescriptor]) -> String {
    let mut output = format!("Previews: {}\n", previews.len());
    for preview in previews {
        output.push_str(&format!(
            "  {}: {} ({}x{}){}\n",
            preview.id,
            preview.label,
            preview.width,
            preview.height,
            preview_options(preview)
        ));
    }
    output.push_str(&format!("Diagnostics: {}\n", inspection.diagnostics.len()));
    for diagnostic in &inspection.diagnostics {
        output.push_str(&format!(
            "  {}: {}: {}\n",
            diagnostic.level, diagnostic.path, diagnostic.message
        ));
    }
    output
}

fn preview_options(preview: &PreviewDescriptor) -> String {
    let mut options = Vec::new();
    if let Some(theme) = &preview.theme {
        options.push(format!("theme={theme}"));
    }
    if let Some(density) = &preview.density {
        options.push(format!("density={density}"));
    }
    if options.is_empty() {
        String::new()
    } else {
        format!(" {}", options.join(" "))
    }
}

fn preview_json(inspection: &ManifestInspection, previews: &[PreviewDescriptor]) -> String {
    let previews = previews
        .iter()
        .map(preview_descriptor_json)
        .collect::<Vec<_>>()
        .join(",");
    let diagnostics = inspection
        .diagnostics
        .iter()
        .map(|diagnostic| {
            format!(
                "{{\"level\":\"{}\",\"path\":\"{}\",\"message\":\"{}\"{}}}",
                escape_json(&diagnostic.level),
                escape_json(&diagnostic.path),
                escape_json(&diagnostic.message),
                diagnostic
                    .fix_hint
                    .as_deref()
                    .map(|hint| format!(",\"fix_hint\":\"{}\"", escape_json(hint)))
                    .unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{{\"ok\":{},\"previews\":[{}],\"diagnostics\":[{}]}}",
        inspection.ok, previews, diagnostics
    )
}

fn preview_descriptor_json(preview: &PreviewDescriptor) -> String {
    format!(
        "{{\"id\":\"{}\",\"label\":\"{}\",\"width\":{},\"height\":{},\"theme\":{},\"density\":{}}}",
        escape_json(&preview.id),
        escape_json(&preview.label),
        preview.width,
        preview.height,
        optional_json_string(preview.theme.as_deref()),
        optional_json_string(preview.density.as_deref())
    )
}

fn bundle_manifest(target: String, json: bool, manifest: PathBuf) -> Result<ExitCode, String> {
    let Some(target) = BundleTarget::parse(&target) else {
        return Err(
            "unknown bundle target; use staccato, flatpak, appimage, windows, or macos".to_string(),
        );
    };
    let manifest_data = match parse_file(&manifest) {
        Ok(manifest_data) => manifest_data,
        Err(error) => {
            if json {
                println!(
                    "{{\"ok\":false,\"diagnostics\":[{{\"level\":\"error\",\"path\":\"{}\",\"message\":\"{}\"}}]}}",
                    escape_json(&manifest.display().to_string()),
                    escape_json(&error.to_string())
                );
            } else {
                eprintln!("bundle failed: {error}");
            }
            return Ok(ExitCode::from(1));
        }
    };
    let base_dir = manifest_base_dir(&manifest);
    let diagnostics = validate_with_base_dir(&manifest_data, base_dir);
    let plan = BundlePlan::from_manifest(&manifest_data, &diagnostics, target, &manifest);

    if json {
        println!("{}", plan.to_json());
    } else {
        print!("{}", plan.to_text());
    }

    if plan.ok {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}

fn diagnostics_json(diagnostics: &[Diagnostic]) -> String {
    let mut output = String::from("[");
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"level\":\"{}\",\"path\":\"{}\",\"message\":\"{}\"",
            diagnostic_level(diagnostic),
            escape_json(&diagnostic.path),
            escape_json(&diagnostic.message)
        ));
        if let Some(fix_hint) = &diagnostic.fix_hint {
            output.push_str(&format!(",\"fix_hint\":\"{}\"", escape_json(fix_hint)));
        }
        output.push('}');
    }
    output.push(']');
    output
}

fn optional_json_string(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", escape_json(value)))
        .unwrap_or_else(|| "null".to_string())
}

fn diagnostic_level(diagnostic: &Diagnostic) -> &'static str {
    match diagnostic.level {
        DiagnosticLevel::Error => "error",
        DiagnosticLevel::Warning => "warning",
    }
}

fn manifest_base_dir(path: &Path) -> &Path {
    path.parent().unwrap_or_else(|| Path::new("."))
}

fn escape_json(value: &str) -> String {
    let mut output = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch.is_control() => output.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => output.push(ch),
        }
    }
    output
}
