use clap::{App, Arg};
use ollama_rs::Ollama;
use ollama_rs::generation::completion::{request::GenerationRequest, GenerationResponseStream};
use tokio::io::{stdout, AsyncWriteExt};
use tokio_stream::StreamExt;

mod history_log;
use history_log::HistoryLog;

async fn run_prompt(ollama_address: String, ollama_port: u16, ollama_model: String, prompt: String) {
    let ollama = Ollama::new(ollama_address, ollama_port);

    let model = "mistral:latest".to_string();

    let mut stream = ollama.generate_stream(GenerationRequest::new(model, prompt)).await.unwrap();

    let mut stdout = tokio::io::stdout();
    while let Some(res) = stream.next().await {
        let responses = res.unwrap();
        for resp in responses {
            stdout.write_all(resp.response.as_bytes()).await.unwrap();
            stdout.flush().await.unwrap();
        }
    }
}

#[tokio::main]
async fn main() {
    let matches = App::new("Header Comment Utility")
        .version("1.0")
        .author("Ethan Lew <ethanlew16@gmail.com>")
        .about("Automation for generating tedious boilerplate comments.")
        .arg(
            Arg::new("repo")
                .short('r')
                .long("repo")
                .takes_value(true)
                .required(true)
                .help("Directory to Git Repo"),
        )
        .arg(
            Arg::new("file_path")
                .short('f')
                .long("file")
                .takes_value(true)
                .required(true)
                .help("File path within the GitHub repository"),
        )
        .arg(
            Arg::new("branch")
                .short('b')
                .long("branch")
                .takes_value(true)
                .required(true)
                .help("Branch or commit"),
        )
        .arg(
            Arg::new("ollama_port")
                .short('p')
                .long("port")
                .takes_value(true)
                .default_value("11434")
                .help("Port number for the Ollama service"),
        )
        .arg(
            Arg::new("ollama_model")
                .short('m')
                .long("model")
                .takes_value(true)
                .default_value("mistral:latest")
                .help("LLM Model for the Ollama service"),
        )
        .arg(
            Arg::new("ollama_address")
                .short('a')
                .long("address")
                .takes_value(true)
                .default_value("http://localhost")
                .help("Address for the Ollama service"),
        )
        .get_matches();

    let repo_path = matches.value_of("repo").unwrap();
    let file_path = matches.value_of("file_path").unwrap();
    let branch_name = matches.value_of("branch").unwrap();
    let ollama_port: u16 = matches.value_of_t("ollama_port").unwrap(); 
    let ollama_model = matches.value_of("ollama_model").unwrap().to_string();
    let ollama_address = matches.value_of("ollama_address").unwrap().to_string();

    match HistoryLog::from_git_blame(repo_path, file_path, branch_name) {
        Ok(log) => {
            run_prompt(ollama_address, ollama_port, ollama_model, log.prompt()).await;
        }
        Err(e) => {
            eprintln!("Error processing git blame: {}", e);
        }
    }
}

