use chrono::{DateTime, TimeZone, Utc};
use git2::{BlameOptions, Repository};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Clone, PartialEq)]
pub struct Line {
    line_no: usize,
    line: String,
}

#[derive(Clone)]
pub struct Message {
    author_email: String,
    date: DateTime<Utc>,
    commit_id: String,
    message: String,
    lines: Vec<Line>,
}

pub struct HistoryLog {
    entries: HashMap<String, Message>,
}

impl HistoryLog {
    pub fn new() -> Self {
        HistoryLog {
            entries: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, message: Message) {
        let entry = self.entries.entry(message.commit_id.clone()).or_insert_with(|| message.clone());
        if entry.lines != message.lines {
            entry.lines.extend(message.lines.iter().cloned());
        }
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
            let date = Utc.from_utc_datetime(
                &DateTime::from_timestamp(time.seconds(), 0)
                    .unwrap()
                    .naive_utc(),
            );
            let line_start = hunk.final_start_line();
            let line_count = hunk.lines_in_hunk();
            let lines = (line_start..line_start + line_count)
                .map(|i| Line {
                    line_no: i,
                    line: file_lines
                        .get(i - 1) // adjust index
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
                lines,
            };
            log.add_entry(message);
        }

        Ok(log)
    }

    pub fn format_history(&self) -> String {
        let mut result = String::from("HISTORY\n");
        
        // Collect and sort entries by date
        let mut sorted_entries: Vec<&Message> = self.entries.values().collect();
        sorted_entries.sort_by_key(|message| message.date);

        // Format sorted entries into result string
        for message in sorted_entries {
            result.push_str(&format!(
                "{} - {} ({}):\n    -- {}\n",
                message.date.format("%m/%d/%Y"),
                message.author_email,
                message.commit_id,
                message.message
            ));
            for line in &message.lines {
                result.push_str(&format!("        {}: {}\n", line.line_no, line.line));
            }
        }

        result
    }

    pub fn prompt(&self) -> String {
        let history_string = self.format_history();
        format!("Generate a changelog summary based on the following git and source file changes. {}\nThe changelog should be formatted as a timeline, be concise yet detailed enough to capture all significant modifications, and adhere to typical documentation style. Each entry should include the date, file name, type of change (e.g., feature, fix, refactor), and a brief description of the change.", history_string)
    }
}

