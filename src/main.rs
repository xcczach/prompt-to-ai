use clap::{Parser, Subcommand};
use prompt_to_ai::{add_commit_push, clip_commit_prompt};

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
    Push {
        message: String,
    },
}
fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Commit { use_english } => {
            clip_commit_prompt(!use_english).unwrap();
            println!("Commit prompt copied to clipboard.");
        }
        Command::Push { message } => {
            let message = message.trim_end().to_owned();
            println!("Committing and pushing changes...");
            add_commit_push(message).unwrap();
            println!("Changes committed and pushed.");
        }
    }
}
