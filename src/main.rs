use clap::{App, Arg};
mod history_log;
use history_log::HistoryLog;

#[tokio::main]
async fn main() {
    let matches = App::new("Header Comment Utility")
        .version("1.0")
        .author("Ethan Lew <ethanlew16@gmail.com>")
        .about("Automation for generating tedious boilerplate comments.")
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

    match HistoryLog::from_git_blame(repo_path, file_path, branch_name) {
        Ok(log) => {
            log.pretty_print();
        },
        Err(e) => eprintln!("Error processing git blame: {}", e),
    }
}

