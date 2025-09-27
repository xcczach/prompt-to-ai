use clap::{Parser, Subcommand};
use prompt_to_ai::clip_commit_prompt;

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
}
fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Commit { use_english } => {
            clip_commit_prompt(!use_english).unwrap();
            println!("Commit prompt copied to clipboard.");
        }
    }
}
