use chrono::{NaiveDateTime, DateTime, Utc};
use git2::{BlameOptions, Repository, Time};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct Message {
    author_email: String,
    date: DateTime<Utc>,
    commit_id: String,
    message: String,
    lines: Vec<String>,
}

pub struct HistoryLog {
    entries: HashMap<(String, String), Vec<Message>>, // (Date as YYYY-MM-DD, Author Email) -> Vec of messages
}

impl HistoryLog {
    pub fn new() -> Self {
        HistoryLog {
            entries: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, message: Message) {
        let date = message.date.format("%m/%d/%Y").to_string();
        let key = (date, message.author_email.clone());
        let entry = self.entries.entry(key).or_default();
        entry.push(message);
    }

    pub fn from_git_blame(repo_path: &str, file_path: &str, branch_name: &str) -> Result<Self, git2::Error> {
        let repo = Repository::open(repo_path)?;
        let reference = format!("refs/heads/{}", branch_name);
        let commit = repo.find_reference(&reference)?.peel_to_commit()?;
        let path = std::path::Path::new(file_path);
        let mut blame_options = BlameOptions::new();
        let full_path = Path::new(repo_path).join(path);
        let file_contents = fs::read_to_string(full_path).expect("unable to read string");

        blame_options.newest_commit(commit.id());
        let blame = repo.blame_file(path, Some(&mut blame_options))?;

        let mut log = HistoryLog::new();
        let mut seen = std::collections::HashSet::new();

        for hunk in blame.iter() {
            let commit_id = hunk.final_commit_id();
            
            if seen.insert(commit_id) {
                let commit = repo.find_commit(commit_id)?;
                let author = commit.author();
                let time = commit.time();
                let date = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(time.seconds(), 0), Utc);
                let line_start = hunk.final_start_line();
                let line_count = hunk.lines_in_hunk();
                let lines = file_contents.lines().skip(line_start as usize - 1).take(line_count as usize).map(String::from).collect();
                let message = Message {
                    author_email: author.email().unwrap_or("no-email").to_string(),
                    date,
                    commit_id: commit_id.to_string(),
                    message: commit.summary().unwrap_or("No commit message").to_string(),
                    lines: lines,
                };
                log.add_entry(message);
            }
        }

        Ok(log)
    }

    pub fn pretty_print(&self) {
        println!("HISTORY");
        for ((date, author), messages) in &self.entries {
            println!("{} - {}", date, author);
            for message in messages {
                println!("    -- {} ({})", message.message, message.commit_id);
                for line in &message.lines {
                    println!("        {}", line);
                }
            }
        }
    }
}

