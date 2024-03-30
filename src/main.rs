use clap::Parser;
use confique::Config;
use gen::cli::{Cli, Subcommands};
use gen::config::Conf;
use gen::edit::change_file;
use gen::github::GitHub;
use gen::process::{gh, git, try_git};
use rand::Rng;
use regex::Regex;
use serde_json::to_string_pretty;
use serde_json::Value;
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
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

fn housekeeping(config: &Conf) {
    for _ in 0..3 {
        let json_str = gh(&["pr", "list", "--json", "number,mergeable,comments"]);
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
                        GitHub::close(&pr);
                        println!("closed pr: {} (had merge conflicts)", &pr);
                    }
                    "MERGEABLE" => {
                        if !requeued.contains(&pr)
                            && comments.contains("removed from the merge queue")
                        {
                            enqueue(&pr, config);
                            println!("requeued pr: {}", &pr);
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

fn configure_git(config: &Conf) {
    git(&["config", "user.email", &config.git.email]);
    git(&["config", "user.name", &config.git.name]);
}

fn enqueue(pr: &str, config: &Conf) {
    if !config.merge.comment.is_empty() {
        GitHub::comment(pr, &config.merge.comment);
    }
    if !config.merge.labels.is_empty() {
        let labels: Vec<&str> = config.merge.labels.split(',').map(|s| s.trim()).collect();
        for lbl in &labels {
            GitHub::add_label(pr, lbl);
        }
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
    println!("Flake rate: {}", config.test.flake_rate);

    random_float > config.test.flake_rate
}

fn create_pull_request(words: &[String], config: &Conf) -> Result<String, String> {
    let branch_name = format!("change/{}", words.join("-"));
    git(&["checkout", "-t", "-b", &branch_name]);

    let commit_msg = format!("Moving words {}", words.join(", "));
    git(&["commit", "-am", &commit_msg]);
    let result = try_git(&["push", "--set-upstream", "origin", "HEAD"]);
    if result.is_err() {
        git(&["checkout", "main"]);
        git(&["pull"]);
        return Err("could not push to origin".to_owned());
    }

    let j_words = words.join(", ");
    let mut args: Vec<&str> = vec![
        "pr",
        "create",
        "--title",
        &j_words,
        "--body",
        &config.pullrequest.body,
    ];

    for lbl in config.pullrequest.labels.split(',') {
        args.push("--label");
        args.push(lbl.trim());
    }

    let pr_url = gh(args.as_slice());

    let re = Regex::new(r"(.*)/pull/(\d+)$").unwrap();
    let caps = re.captures(pr_url.trim()).unwrap();
    let pr_number = caps.get(2).map_or("", |m| m.as_str());

    git(&["checkout", "main"]);
    git(&["pull"]);

    Ok(pr_number.to_string())
}

fn run() -> anyhow::Result<()> {
    let cli: Cli = Cli::parse();

    if let Some(Subcommands::Genconfig {}) = &cli.subcommand {
        Conf::print_default();
        return Ok(());
    }

    let config = Conf::builder()
        .env()
        .file("mq.toml")
        .file(".config/mq.toml")
        .load()
        .unwrap_or_else(|err| {
            eprintln!("Generator cannot run: {}", err);
            std::process::exit(1);
        });

    if let Some(Subcommands::Housekeeping {}) = &cli.subcommand {
        housekeeping(&config);
        return Ok(());
    }

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

    if config.pullrequest.requests_per_hour == 0 {
        println!("generator is disabled pull requests per hour is set to 0");
        return Ok(());
    }

    configure_git(&config);

    // divide by 6 since we run once every 10 minutes
    let pull_requests_to_make = (config.pullrequest.requests_per_hour as f32 / 6.0).ceil() as usize;

    let mut prs: Vec<String> = Vec::new();

    for _ in 0..pull_requests_to_make {
        let start = Instant::now();
        let files = get_txt_files()?;
        let mut filenames: Vec<String> = files
            .into_iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect();

        filenames.sort();
        let filenames: Vec<String> = filenames
            .into_iter()
            .take(config.pullrequest.max_deps)
            .collect();

        let max_impacted_deps = config.pullrequest.max_impacted_deps as u32; // Convert usize to u32
        let words = change_file(&filenames, max_impacted_deps); // Use the converted value

        let pr_result = create_pull_request(&words, &config);
        if pr_result.is_err() {
            println!("problem created pr for {:?}", words);
            continue;
        }
        let duration = start.elapsed();
        let pr = pr_result.unwrap();
        println!("created pr: {} in {:?}", pr, duration);
        prs.push(pr);
    }

    for pr in &prs {
        enqueue(pr, &config)
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
