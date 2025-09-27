use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "pai")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Diff,
}
fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Diff => {
            println!("Committing changes...");
            // Add your commit logic here
        }
    }
}
