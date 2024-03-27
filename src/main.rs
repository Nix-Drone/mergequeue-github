use clap::Parser;
use confique::Config;
use gen::cli::{Cli, Subcommands};
use gen::config::Conf;
use gen::edit::change_file;
use serde_json::to_string_pretty;
use std::path::PathBuf;
use walkdir::WalkDir;

fn get_txt_files() -> std::io::Result<Vec<PathBuf>> {
    let mut path = std::env::current_dir()?;
    path.push("bazel/");
    let mut paths = Vec::new();
    for entry in WalkDir::new(&path) {
        let entry = entry?;
        if entry.file_type().is_file()
            && entry.path().extension().and_then(std::ffi::OsStr::to_str) == Some("txt")
        {
            paths.push(entry.path().to_path_buf());
        }
    }
    Ok(paths)
}

fn run() -> anyhow::Result<()> {
    let cli: Cli = Cli::parse();

    if let Some(Subcommands::Genconfig {}) = &cli.subcommand {
        Conf::print_default();
        return Ok(());
    }
    let config = Conf::builder()
        .env()
        .file("demo.toml")
        .file(".config/demo.toml")
        .load()
        .unwrap_or_else(|err| {
            eprintln!("Generator cannot run: {}", err);
            std::process::exit(1);
        });

    if let Some(Subcommands::Config {}) = &cli.subcommand {
        let config_json = to_string_pretty(&config).expect("Failed to serialize config to JSON");
        println!("{}", config_json);
        return Ok(());
    }

    println!("{:?}", config.mode);

    println!("{:?}", cli.gh_token);

    let files = get_txt_files()?;
    let mut filenames: Vec<String> = files
        .into_iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();

    filenames.sort();
    let filenames: Vec<String> = filenames.into_iter().take(config.max_deps).collect();

    let max_impacted_deps = config.max_impacted_deps as u32; // Convert usize to u32
    let words = change_file(&filenames, max_impacted_deps); // Use the converted value

    println!("::set-output name=words::{}", words.join(", "));
    println!("::set-output name=words-in-one::{}", words.join("-"));

    Ok(())
}

fn main() {
    env_logger::init();

    match run() {
        Ok(_) => (),
        Err(err) => {
            log::error!("{}", err);
            std::process::exit(1);
        }
    }
}
