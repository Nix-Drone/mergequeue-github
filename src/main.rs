use clap::Parser;
use confique::Config;
use gen::cli::{Cli, Subcommands};
use gen::config::Conf;
use gen::edit::change_file;
use rand::Rng;
use regex::Regex;
use serde_json::to_string_pretty;
use serde_json::Value;
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;
use std::time::Instant;
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

fn queue_to_merge(pr: &String) {
    let output = Command::new("gh")
        .arg("pr")
        .arg("comment")
        .arg(pr) // Fix: Pass the expression directly as an argument
        .arg("--body")
        .arg("/trunk merge")
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Call to comment on PR {} on GitHub failed", pr);
    }
}

fn close_pr(pr: &str) {
    let output = Command::new("gh")
        .arg("pr")
        .arg("close")
        .arg(pr) // Fix: Pass the expression directly as an argument
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        eprintln!("could not close pr {}", pr);
    }
}

fn housekeeping() {
    for _ in 0..3 {
        let output = Command::new("gh")
            .arg("pr")
            .arg("list")
            .arg("--json") // Fix: Pass the expression directly as an argument
            .arg("number,mergeable,comments")
            .output()
            .expect("Failed to execute command");

        if !output.status.success() {
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("could not get list of prs");
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let v: Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");

        let mut has_unknown = false;
        let mut requeued: HashSet<String> = HashSet::new();
        if let Some(array) = v.as_array() {
            for item in array {
                let mergeable = item["mergeable"].as_str().unwrap_or("");
                let pr = item["number"].as_i64().unwrap_or(0).to_string();
                let comments = item["comments"].to_string();
                match mergeable {
                    "UNKNOWN" => {
                        has_unknown = true;
                    }
                    "CONFLICTING" => {
                        close_pr(&pr);
                    }
                    "MERGEABLE" => {
                        if !requeued.contains(&pr)
                            && comments.contains("removed from the merge queue")
                        {
                            queue_to_merge(&pr);
                            requeued.insert(pr);
                        }
                    }
                    _ => {
                        // handle other states
                    }
                }
            }

            if has_unknown {
                thread::sleep(Duration::from_secs(10));
            } else {
                return;
            }
        } else {
            return;
        }
    }
}

fn configure_git() {
    let output = Command::new("git")
        .arg("config")
        .arg("user.email")
        .arg("bot@trunk.io")
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Failed to run git config email");
    }

    let output = Command::new("git")
        .arg("config")
        .arg("user.name")
        .arg("trunk bot")
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Failed to run git config name");
    }
}

fn test_with_flakes(config: &Conf) -> bool {
    let is_merge_str = env::var("IS_MERGE").unwrap_or_else(|_| String::from("false"));
    let is_merge = is_merge_str.to_lowercase() == "true";

    if !is_merge {
        println!("no flake or sleep when running on pr branch");
        return true;
    }

    println!("sleeping for {} seconds", config.sleep_duration().as_secs());
    thread::sleep(config.sleep_duration());

    let mut rng = rand::thread_rng();
    let random_float = rng.gen_range(0.0..1.0);

    println!("Random float: {}", random_float);
    println!("Flake rate: {}", config.flake_rate);

    random_float > config.flake_rate
}

fn create_pull_request(words: &[String]) -> String {
    let branch_name = format!("change/{}", words.join("-"));

    let output = Command::new("git")
        .arg("checkout")
        .arg("-t")
        .arg("-b")
        .arg(&branch_name)
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        panic!("Command executed with failing error code");
    }

    let output = Command::new("git")
        .arg("commit")
        .arg("-am")
        .arg(format!("Moving words {}", words.join(", ")))
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Failed to run git commit");
    }

    let output = Command::new("git")
        .arg("push")
        .arg("--set-upstream")
        .arg("origin")
        .arg("HEAD")
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Failed to push the current branch to the remote repository");
    }

    let output = Command::new("gh")
        .arg("pr")
        .arg("create")
        .arg("--title")
        .arg(words.join(", ")) // Fix: Pass the expression directly as an argument
        .arg("--body")
        .arg("This PR was generated by the trunk-pr-generator tool.")
        .arg("--label")
        .arg("bot-pr")
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Call to create PR on GitHub failed");
    }

    let pr_url = String::from_utf8_lossy(&output.stdout);
    let re = Regex::new(r"(.*)/pull/(\d+)$").unwrap();
    let caps = re.captures(pr_url.trim()).unwrap();
    let pr_number = caps.get(2).map_or("", |m| m.as_str());

    let output = Command::new("git")
        .arg("checkout")
        .arg("main")
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        panic!("Command executed with failing error code");
    }

    pr_number.to_string()
}

fn run() -> anyhow::Result<()> {
    let cli: Cli = Cli::parse();

    if let Some(Subcommands::Genconfig {}) = &cli.subcommand {
        Conf::print_default();
        return Ok(());
    }

    if let Some(Subcommands::Housekeeping {}) = &cli.subcommand {
        housekeeping();
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

    if let Some(Subcommands::TestSim {}) = &cli.subcommand {
        if !test_with_flakes(&config) {
            std::process::exit(1);
        }
        return Ok(());
    }

    if let Some(Subcommands::Config {}) = &cli.subcommand {
        let config_json = to_string_pretty(&config).expect("Failed to serialize config to JSON");
        println!("{}", config_json);
        return Ok(());
    }

    if config.pull_requests_per_hour == 0 {
        println!("generator is disabled pull requests per hour is set to 0");
        return Ok(());
    }

    configure_git();

    // divide by 6 since we run once every 10 minutes
    let pull_requests_to_make = (config.pull_requests_per_hour as f32 / 6.0).ceil() as usize;

    let mut prs: Vec<String> = Vec::new();

    for _ in 0..pull_requests_to_make {
        let start = Instant::now();
        let files = get_txt_files()?;
        let mut filenames: Vec<String> = files
            .into_iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect();

        filenames.sort();
        let filenames: Vec<String> = filenames.into_iter().take(config.max_deps).collect();

        let max_impacted_deps = config.max_impacted_deps as u32; // Convert usize to u32
        let words = change_file(&filenames, max_impacted_deps); // Use the converted value

        let pr = create_pull_request(&words);
        let duration = start.elapsed();
        println!("created pr: {} in {:?}", pr, duration);
        prs.push(pr);
    }

    for pr in &prs {
        queue_to_merge(pr);
    }

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
