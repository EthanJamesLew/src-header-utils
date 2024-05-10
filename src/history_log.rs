use std::collections::HashMap;
use git2::{Repository, BlameOptions, Time, Commit};
use chrono::NaiveDateTime;

pub struct HistoryLog {
    entries: HashMap<(String, String), Vec<String>>, // (Date, Author Email) -> Vec of messages
}

impl HistoryLog {
    pub fn new() -> Self {
        HistoryLog {
            entries: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, date: String, author_email: String, message: String) {
        let key = (date, author_email);
        let entry = self.entries.entry(key).or_insert_with(Vec::new);
        entry.push(message);
    }

    pub fn from_git_blame(repo_path: &str, file_path: &str, branch_name: &str) -> Result<Self, git2::Error> {
        let repo = Repository::open(repo_path)?;
        let reference = format!("refs/heads/{}", branch_name);
        let commit = repo.find_reference(&reference)?.peel_to_commit()?;
        let path = std::path::Path::new(file_path);
        let mut blame_options = BlameOptions::new();
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
                let date = HistoryLog::time_to_string(time);
                let author_email = author.email().unwrap_or("no-email").to_string();

                log.add_entry(date, author_email, commit.summary().unwrap_or("No commit message").to_string());
            }
        }

        Ok(log)
    }

    fn time_to_string(time: Time) -> String {
        let datetime = NaiveDateTime::from_timestamp(time.seconds(), 0);
        datetime.format("%m/%d/%Y").to_string()
    }

    pub fn pretty_print(&self) {
        println!("HISTORY");
        for ((date, author), messages) in &self.entries {
            println!("{} - {}", date, author);
            for message in messages {
                println!("    -- {}", message);
            }
        }
    }
}

