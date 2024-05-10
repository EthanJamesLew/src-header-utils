use clap::{App, Arg};
use std::process;
use reqwest::Client;
use git2::{Repository, BlameOptions, Commit, Time};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use chrono;

fn git_blame(repo_path: &str, file_path: &str, branch_name: &str) -> Result<(), git2::Error> {
    let repo = Repository::open(repo_path)?;
    let reference = format!("refs/heads/{}", branch_name);
    let commit = repo.find_reference(&reference)?.peel_to_commit()?;
    let path = Path::new(file_path);
    let mut blame_options = BlameOptions::new();
    blame_options.newest_commit(commit.id());
    let blame = repo.blame_file(path, Some(&mut blame_options))?;

    let mut commits_by_author_date: HashMap<(String, String), Vec<String>> = HashMap::new();
    let mut seen = HashSet::new();

    for hunk in blame.iter() {
        let commit_id = hunk.final_commit_id();
        if seen.insert(commit_id) {
            let commit = repo.find_commit(commit_id)?;
            let author = commit.author();
            let time = commit.time();
            let date = time_to_string(time);
            let author_email = author.email().unwrap_or("no-email").to_string();
            let key = (date, author_email);

            let entry = commits_by_author_date.entry(key).or_insert_with(Vec::new);
            entry.push(commit.summary().unwrap_or("No commit message").to_string());
        }
    }

    // Print formatted commit information
    println!("HISTORY");
    for ((date, author), messages) in commits_by_author_date {
        println!("{} - {}", date, author);
        for message in messages {
            println!("    -- {}", message);
        }
    }

    Ok(())
}

fn time_to_string(time: Time) -> String {
    let datetime = chrono::NaiveDateTime::from_timestamp(time.seconds(), 0);
    let date = datetime.format("%m/%d/%Y").to_string();
    date
}


#[tokio::main]
async fn main() {
    let matches = App::new("GitHub History Tool")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("Analyzes file history in GitHub repositories")
        .arg(Arg::with_name("repo")
             .short('r')
             .long("repo")
             .takes_value(true)
             .required(true)
             .help("Directory to Git Repo"))
        .arg(Arg::with_name("file_path")
             .short('f')
             .long("file")
             .takes_value(true)
             .required(true)
             .help("File path within the GitHub repository"))
        .arg(Arg::with_name("branch")
             .short('b')
             .long("branch")
             .takes_value(true)
             .required(true)
             .help("branch or commit"))
        .get_matches();

    let file_path = matches.value_of("file_path").unwrap();
    let repo_path = matches.value_of("repo").unwrap();
    let branch_name = matches.value_of("branch").unwrap();

    if let Err(e) = run(file_path, repo_path, branch_name).await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

async fn run(file_path: &str, repo_path: &str, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Use the new git_blame function signature
    git_blame(repo_path, file_path, branch_name)?;

    Ok(())
}
