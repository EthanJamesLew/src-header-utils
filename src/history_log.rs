use chrono::{DateTime, Utc, TimeZone};
use git2::{BlameOptions, Repository};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct Line {
    line_no: usize,
    line: String,
}

pub struct Message {
    author_email: String,
    date: DateTime<Utc>,
    commit_id: String,
    message: String,
    lines: Vec<Line>,
}

pub struct HistoryLog {
    entries: HashMap<(String, String), Vec<Message>>,
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

    pub fn from_git_blame(
        repo_path: &str,
        file_path: &str,
        branch_name: &str,
    ) -> Result<Self, git2::Error> {
        let repo = Repository::open(repo_path)?;
        let reference = format!("refs/heads/{}", branch_name);
        let commit = repo.find_reference(&reference)?.peel_to_commit()?;
        let path = Path::new(file_path);
        let mut blame_options = BlameOptions::new();
        let full_path = Path::new(repo_path).join(path);
        let file_contents = fs::read_to_string(full_path).expect("Unable to read string");
        let file_lines: Vec<String> = file_contents.lines().map(String::from).collect();

        blame_options.newest_commit(commit.id());
        let blame = repo.blame_file(path, Some(&mut blame_options))?;

        let mut log = HistoryLog::new();

        for hunk in blame.iter() {
            let commit_id = hunk.final_commit_id();
            let commit = repo.find_commit(commit_id)?;
            let author = commit.author();
            let time = commit.time();
            let date =
                Utc.from_utc_datetime(&DateTime::from_timestamp(time.seconds(), 0).unwrap().naive_utc());
            let line_start = hunk.final_start_line();
            let line_count = hunk.lines_in_hunk();
            let lines = (line_start..line_start + line_count)
                .map(|i| Line {
                    line_no: i,
                    line: file_lines
                        .get(i)
                        .unwrap_or(&String::from("<error>"))
                        .to_string(),
                })
                .collect();

            let message = Message {
                author_email: author.email().unwrap_or("<UNKNOWN EMAIL>").to_string(),
                date,
                commit_id: commit_id.to_string(),
                message: commit
                    .summary()
                    .unwrap_or("<NO COMMIT MESSAGE>")
                    .to_string(),
                lines: lines,
            };
            log.add_entry(message);
        }

        Ok(log)
    }

    pub fn format_history(&self) -> String {
        let mut result = String::from("HISTORY\n");
        for ((date, author), messages) in &self.entries {
            result.push_str(&format!("{} - {}\n", date, author));
            for message in messages {
                result.push_str(&format!(
                    "    -- {} ({})\n",
                    message.message, message.commit_id
                ));
                for line in &message.lines {
                    result.push_str(&format!("        {} {}\n", line.line_no, line.line));
                }
            }
        }
        result
    }

    pub fn prompt(&self) -> String {
        let history_string = self.format_history();
        format!("Take the history log: {}\nWrite a timeline change that summarizes the history. Use a bullet list formatted as: * {{date}} - {{author}} - {{summary}}", history_string)
    }
}
