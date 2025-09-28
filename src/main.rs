use clap::{Parser, Subcommand};
use prompt_to_ai::{add_commit, clip_commit_prompt};

#[derive(Parser)]
#[command(name = "pai")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Commit {
        #[arg(short = 'e', long = "english", default_value_t = false)]
        use_english: bool,
    },
    Ls,
}
fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Commit { use_english } => {
            clip_commit_prompt(!use_english).unwrap();
            println!("Commit prompt copied to clipboard.");
            // Prompt user to enter commit message
            println!("Please enter commit message:");
            let mut commit_msg = String::new();
            std::io::stdin().read_line(&mut commit_msg).unwrap();
            println!("Committing with message: {}", commit_msg.trim());
            add_commit(commit_msg.trim().to_owned()).unwrap();
            println!("Committed successfully.");
        }
        Command::Ls => {
            println!("Listing files is not implemented yet.");
        }
    }
}
